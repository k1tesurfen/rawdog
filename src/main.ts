import './style.css'
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

interface ImageInfo {
  id: string;
  raw_path: string;
  preview_path: string | null;
  status: 'Pending' | 'Keep' | 'Reject' | 'Favorite';
}

let images: ImageInfo[] = [];
let currentIndex = 0;
let isFinishOverlayVisible = false;

const app = document.querySelector<HTMLDivElement>('#app')!;

app.innerHTML = `
  <div id="start-screen" class="start-screen">
    <h1>rawdog</h1>
    <button id="select-folder" class="select-button">Select Photo Folder</button>
  </div>
  <div class="controls-hint">
    [H/←] Prev | [L/→] Next | [J/↑] Keep | [K/↓] Reject | [F] Favorite | [U] Undo | [ESC] Finish
  </div>
  <div class="image-container">
    <img id="main-image" class="main-image" src="" />
    <div id="status-indicator" class="status-indicator"></div>
  </div>
  <div class="overlay">
    <div id="filename">No folder selected</div>
    <div id="counter">0 / 0</div>
  </div>
  <div id="finish-overlay" class="finish-overlay">
    <h2>Finish Culling?</h2>
    <p>Unkept images will be moved to the "unused" folder.</p>
    <div class="shortcut">Press ENTER to Confirm | ESC to Cancel</div>
  </div>
`;

const imgElement = document.getElementById('main-image') as HTMLImageElement;
const filenameElement = document.getElementById('filename') as HTMLDivElement;
const counterElement = document.getElementById('counter') as HTMLDivElement;
const statusIndicator = document.getElementById('status-indicator') as HTMLDivElement;
const finishOverlay = document.getElementById('finish-overlay') as HTMLDivElement;
const startScreen = document.getElementById('start-screen') as HTMLDivElement;
const selectButton = document.getElementById('select-folder') as HTMLButtonElement;

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Select Photo Folder"
  });
  if (selected) {
    startCulling(selected as string);
  }
}

async function startCulling(path: string) {
  startScreen.classList.add('hidden');
  images = await invoke<ImageInfo[]>('start_scanning', { path });
  currentIndex = 0;
  updateDisplay();
}

async function loadImages() {
  try {
    const newImages = await invoke<ImageInfo[]>('get_images');
    images = newImages;
    updateDisplay();
  } catch (error) {
    console.error("Failed to load images:", error);
  }
}

function updateDisplay() {
  if (images.length === 0) {
    return;
  }

  const img = images[currentIndex];
  if (img.preview_path) {
    const src = `rawdog://localhost${img.preview_path}`;
    imgElement.src = src;
  } else {
    imgElement.src = "";
    filenameElement.innerText = `Extracting preview for ${img.raw_path.split('/').pop()}...`;
  }

  filenameElement.innerText = img.raw_path.split('/').pop() || "";
  counterElement.innerText = `${currentIndex + 1} / ${images.length}`;
  
  if (currentIndex + 1 < images.length) {
    const nextImg = images[currentIndex + 1];
    if (nextImg.preview_path) {
      const preload = new Image();
      preload.src = `rawdog://localhost${nextImg.preview_path}`;
    }
  }
}

async function setStatus(status: 'Keep' | 'Reject' | 'Favorite' | 'Pending') {
  if (images.length === 0) return;
  const img = images[currentIndex];
  img.status = status;
  await invoke('update_status', { id: img.id, status });
  showStatusIndicator(status);
  
  if (status !== 'Pending') {
    if (currentIndex < images.length - 1) {
      currentIndex++;
      updateDisplay();
    } else {
      // Last image reached and rated
      showFinishOverlay();
    }
  }
}

function showStatusIndicator(status: string) {
  statusIndicator.innerText = status.toUpperCase();
  statusIndicator.className = `status-indicator visible status-${status.toUpperCase()}`;
  setTimeout(() => statusIndicator.classList.remove('visible'), 500);
}

function showFinishOverlay() {
  isFinishOverlayVisible = true;
  finishOverlay.classList.add('visible');
}

function toggleFinishOverlay() {
  isFinishOverlayVisible = !isFinishOverlayVisible;
  finishOverlay.classList.toggle('visible', isFinishOverlayVisible);
}

window.addEventListener('keydown', (e) => {
  if (isFinishOverlayVisible) {
    if (e.key === 'Enter') invoke('finish_culling');
    else if (e.key === 'Escape') toggleFinishOverlay();
    return;
  }
  if (startScreen.classList.contains('hidden')) {
    switch (e.key.toLowerCase()) {
      case 'escape': toggleFinishOverlay(); break;
      case 'h':
      case 'arrowleft':
        if (currentIndex > 0) { currentIndex--; updateDisplay(); }
        break;
      case 'l':
      case 'arrowright':
        if (currentIndex < images.length - 1) { currentIndex++; updateDisplay(); }
        break;
      case 'j':
      case 'arrowup':
        setStatus('Keep');
        break;
      case 'k':
      case 'arrowdown':
        setStatus('Reject');
        break;
      case 'f':
        setStatus('Favorite');
        break;
      case 'u':
        if (currentIndex > 0) { currentIndex--; setStatus('Pending'); }
        break;
    }
  }
});

selectButton.addEventListener('click', selectFolder);

listen('auto-start', (event) => {
  startCulling(event.payload as string);
});

listen('preview-updated', () => loadImages());
listen('previews-ready', () => loadImages());
setInterval(loadImages, 1000);
