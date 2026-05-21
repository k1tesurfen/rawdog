use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::process::Command;
use walkdir::WalkDir;
use uuid::Uuid;
use tauri::{Emitter, State, AppHandle, Manager};
use tauri::http::{Response, HeaderValue};
use std::io::Read;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CullingStatus {
    Pending,
    Keep,
    Reject,
    Favorite,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageInfo {
    pub id: String,
    pub raw_path: PathBuf,
    pub preview_path: Option<PathBuf>,
    pub status: CullingStatus,
}

pub struct AppState {
    pub target_dir: Mutex<Option<PathBuf>>,
    pub cache_dir: PathBuf,
    pub images: Mutex<Vec<ImageInfo>>,
}

#[tauri::command]
fn get_images(state: State<'_, AppState>) -> Vec<ImageInfo> {
    state.images.lock().unwrap().clone()
}

#[tauri::command]
fn update_status(id: String, status: CullingStatus, state: State<'_, AppState>) {
    let mut images = state.images.lock().unwrap();
    if let Some(img) = images.iter_mut().find(|i| i.id == id) {
        img.status = status;
    }
}

#[tauri::command]
fn finish_culling(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let images = state.images.lock().unwrap();
    let target_dir = state.target_dir.lock().unwrap();
    let target_dir = target_dir.as_ref().ok_or("No target directory set")?;
    
    // Create 'unused' directory
    let unused_dir = target_dir.join("unused");
    
    let mut moved_any = false;
    for img in images.iter() {
        if img.status != CullingStatus::Keep && img.status != CullingStatus::Favorite {
            if !moved_any {
                std::fs::create_dir_all(&unused_dir).map_err(|e| e.to_string())?;
                moved_any = true;
            }
            let dest = unused_dir.join(img.raw_path.file_name().unwrap());
            if let Err(e) = std::fs::rename(&img.raw_path, dest) {
                eprintln!("Failed to move {:?}: {}", img.raw_path, e);
            }
        }
    }

    let _ = std::fs::remove_dir_all(&state.cache_dir);
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn start_scanning(path: String, state: State<'_, AppState>, handle: AppHandle) -> Result<Vec<ImageInfo>, String> {
    let target_dir = PathBuf::from(path);
    {
        let mut dir = state.target_dir.lock().unwrap();
        *dir = Some(target_dir.clone());
    }

    let mut images = Vec::new();
    let extensions = ["arw", "cr2", "nef", "dng", "orf", "raf"];
    for entry in WalkDir::new(&target_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    images.push(ImageInfo {
                        id: Uuid::new_v4().to_string(),
                        raw_path: path.to_path_buf(),
                        preview_path: None,
                        status: CullingStatus::Pending,
                    });
                }
            }
        }
    }

    let images_clone = images.clone();
    {
        let mut state_images = state.images.lock().unwrap();
        *state_images = images;
    }

    let cache_dir = state.cache_dir.clone();
    std::thread::spawn(move || {
        let state = handle.state::<AppState>();
        let images_to_process = images_clone;

        for mut img in images_to_process {
            img.preview_path = extract_preview(&img.raw_path, &cache_dir);
            {
                let mut images = state.images.lock().unwrap();
                if let Some(i) = images.iter_mut().find(|i| i.id == img.id) {
                    i.preview_path = img.preview_path.clone();
                }
            }
            let _ = handle.emit("preview-updated", img.id);
        }
        let _ = handle.emit("previews-ready", ());
    });

    Ok(state.images.lock().unwrap().clone())
}

fn extract_preview(raw_path: &Path, cache_dir: &Path) -> Option<PathBuf> {
    let filename = raw_path.file_name()?.to_str()?;
    let output_path = cache_dir.join(format!("{}.jpg", filename));
    if output_path.exists() {
        return Some(output_path);
    }
    let status = Command::new("sips")
        .arg("-s").arg("format").arg("jpeg")
        .arg("-Z").arg("2048")
        .arg(raw_path)
        .arg("--out").arg(&output_path)
        .status().ok()?;
    if status.success() { Some(output_path) } else { None }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let mut initial_dir = None;
    if args.len() > 1 && !args[1].starts_with('-') {
        initial_dir = Some(PathBuf::from(&args[1]));
    }

    let cache_dir = std::env::temp_dir().join("rawdog").join(Uuid::new_v4().to_string());
    std::fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("rawdog", move |_app, request| {
            let path = request.uri().path();
            let decoded_path = percent_encoding::percent_decode_str(path).decode_utf8_lossy().into_owned();
            let path_buf = PathBuf::from(&decoded_path);
            match std::fs::File::open(&path_buf) {
                Ok(mut file) => {
                    let mut data = Vec::new();
                    if let Ok(_) = file.read_to_end(&mut data) {
                        Response::builder().header("Content-Type", "image/jpeg").body(data).unwrap()
                    } else { Response::builder().status(500).body(Vec::new()).unwrap() }
                }
                Err(_) => Response::builder().status(404).body(Vec::new()).unwrap(),
            }
        })
        .manage(AppState {
            target_dir: Mutex::new(initial_dir.clone()),
            cache_dir: cache_dir.clone(),
            images: Mutex::new(Vec::new()),
        })
        .setup(move |app| {
            if let Some(dir) = initial_dir {
                let handle = app.handle().clone();
                let dir_str = dir.to_string_lossy().to_string();
                std::thread::spawn(move || {
                    // Wait a bit for frontend to be ready
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = handle.emit("auto-start", dir_str);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_images, update_status, finish_culling, start_scanning])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
