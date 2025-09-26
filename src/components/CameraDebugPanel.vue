<template>
  <div class="column q-gutter-sm">
    <div class="text-h6">📷 Camera Debug Panel</div>
    <div class="row q-gutter-sm">
      <q-btn color="primary" label="Start (JS)" @click="startJs" />
      <q-btn color="negative" label="Stop (JS)" @click="stopJs" />
      <q-btn color="secondary" label="Start (Bevy event)" @click="startBevy" />
      <q-btn color="grey-8" label="Stop (Bevy event)" @click="stopBevy" />
    </div>
    <div class="row q-gutter-sm items-center">
      <q-btn flat color="teal" label="Use Video File…" @click="pickFile" />
      <input ref="fileRef" type="file" accept="video/*" style="display:none" @change="onFile" />
      <div v-if="status" class="text-caption">{{ status }}</div>
    </div>
    <q-separator />
    <div class="text-subtitle2">📹 Camera Preview</div>
    
    <!-- Development Camera Preview Canvas -->
    <div class="column q-gutter-sm q-mb-md">
      <canvas 
        ref="previewCanvas" 
        :width="previewWidth" 
        :height="previewHeight"
        style="border: 2px solid #9c27b0; border-radius: 4px; max-width: 300px; background: #000;"
        :style="{ display: showPreview ? 'block' : 'none' }"
      ></canvas>
      <div v-if="!showPreview" class="text-caption text-grey-6">
        📷 Camera preview will appear here when camera is started
      </div>
    </div>
    
    <div class="row q-gutter-sm">
      <q-btn color="purple" label="Enable Preview" @click="enablePreview" />
      <q-btn color="grey" label="Disable Preview" @click="disablePreview" />
    </div>
    <div class="row q-gutter-sm items-center">
      <q-slider v-model="previewScale" :min="0.1" :max="2.0" :step="0.1" label color="purple" style="max-width: 200px" />
      <div class="text-caption">Scale: {{ previewScale.toFixed(1) }}</div>
    </div>
    <div class="row q-gutter-sm">
      <q-select v-model="previewAnchor" :options="anchorOptions" label="Position" dense style="min-width: 120px" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { cameraService } from '../services/CameraService'

const status = ref('')
const fileRef = ref<HTMLInputElement | null>(null)
const previewScale = ref(0.3)
const previewAnchor = ref('TopRight')
const anchorOptions = ['TopLeft', 'TopRight', 'BottomLeft', 'BottomRight']

// Development preview canvas
const previewCanvas = ref<HTMLCanvasElement | null>(null)
const showPreview = ref(false)
const previewWidth = ref(320)
const previewHeight = ref(240)

// Camera stream for HTML preview
let previewStream: MediaStream | null = null
let animationId: number | null = null

// Set up canvas preview from camera stream
async function setupCanvasPreview(stream: MediaStream) {
  console.log('🎬 Setting up canvas preview...')
  if (!previewCanvas.value) {
    console.error('🎬 Preview canvas not available!')
    return
  }
  console.log('🎬 Canvas element found:', previewCanvas.value)
  
  const video = document.createElement('video')
  video.srcObject = stream
  video.autoplay = true
  video.muted = true
  video.playsInline = true
  console.log('🎬 Created video element with stream')
  
  await new Promise<void>((resolve) => {
    video.addEventListener('loadedmetadata', () => {
      console.log('🎬 Video metadata loaded:', { 
        videoWidth: video.videoWidth, 
        videoHeight: video.videoHeight,
        readyState: video.readyState 
      })
      previewWidth.value = Math.min(video.videoWidth, 320)
      previewHeight.value = Math.min(video.videoHeight, 240)
      console.log('🎬 Set canvas dimensions:', { width: previewWidth.value, height: previewHeight.value })
      resolve()
    })
  })
  
  const canvas = previewCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    console.error('🎬 Failed to get canvas 2D context!')
    return
  }
  console.log('🎬 Got canvas 2D context')
  
  // Animation loop to draw video frames to canvas
  const drawFrame = () => {
    if (!video || !canvas || !ctx) {
      console.warn('🎬 Missing elements in drawFrame:', { video: !!video, canvas: !!canvas, ctx: !!ctx })
      return
    }
    
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height)
    animationId = requestAnimationFrame(drawFrame)
  }
  
  video.addEventListener('play', () => {
    console.log('🎬 Video started playing, beginning animation loop')
    drawFrame()
    showPreview.value = true
    console.log('🎬 Set showPreview to true')
  })
  
  // Try to trigger video play manually if it doesn't auto-play
  video.play().then(() => {
    console.log('🎬 Video.play() successful')
  }).catch((error) => {
    console.warn('🎬 Video.play() failed:', error)
  })
  
  previewStream = stream
  console.log('🎬 Canvas preview setup completed')
}

// Stop canvas preview
function stopCanvasPreview() {
  if (animationId) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
  
  if (previewStream) {
    previewStream.getTracks().forEach(track => track.stop())
    previewStream = null
  }
  
  showPreview.value = false
  
  // Clear canvas
  if (previewCanvas.value) {
    const ctx = previewCanvas.value.getContext('2d')
    ctx?.clearRect(0, 0, previewCanvas.value.width, previewCanvas.value.height)
  }
}

onUnmounted(() => {
  stopCanvasPreview()
})

async function startJs() {
  status.value = 'Starting camera (JS)…'
  console.log('🎬 Starting camera service...')
  await cameraService.start({ width: 640, height: 480, rear: true, zoom: 0.25 })
  
  // Also start HTML canvas preview for development
  try {
    console.log('🎬 Requesting HTML preview stream...')
    const stream = await navigator.mediaDevices.getUserMedia({ 
      video: { width: 640, height: 480, facingMode: 'environment' } 
    })
    console.log('🎬 Got camera stream for HTML preview:', stream)
    console.log('🎬 Stream tracks:', stream.getTracks().map(t => ({ kind: t.kind, enabled: t.enabled, readyState: t.readyState })))
    await setupCanvasPreview(stream)
    console.log('🎬 HTML canvas preview setup completed')
  } catch (error) {
    console.warn('Could not start HTML preview:', error)
    console.error('HTML preview error details:', error)
  }
  
  status.value = 'Camera running (JS)'
}

async function stopJs() {
  await cameraService.stop()
  stopCanvasPreview()
  status.value = 'Camera stopped (JS)'
}

function startBevy() {
  const wasm = window.__A4D_WASM__
  if (!wasm?.send_event_to_bevy && !wasm?.send_js_to_bevy_event) {
    status.value = 'WASM bridge not available'
    return
  }
  const ev = {
    type: 'CameraStart',
    request_id: `cam-${Date.now()}`,
    width: 640,
    height: 480,
  }
  const s = JSON.stringify(ev)
  // For debug: directly dispatch a Bevy-to-JS event to trigger the TS bridge
  window.dispatchEvent(new CustomEvent('bevy-to-js-event', { detail: s }))
  status.value = 'Sent CameraStart to Bevy'
}

function stopBevy() {
  const wasm = window.__A4D_WASM__
  if (!wasm?.send_event_to_bevy && !wasm?.send_js_to_bevy_event) {
    status.value = 'WASM bridge not available'
    return
  }
  const ev = {
    type: 'CameraStop',
    request_id: `cam-${Date.now()}`,
  }
  const s = JSON.stringify(ev)
  window.dispatchEvent(new CustomEvent('bevy-to-js-event', { detail: s }))
  status.value = 'Sent CameraStop to Bevy'
}

function enablePreview() {
  const gameEngine = window.__A4D_WASM__?.game_engine
  if (!gameEngine?.enable_camera_preview) {
    status.value = 'WASM camera preview not available'
    return
  }
  const requestId = gameEngine.enable_camera_preview(previewScale.value, previewAnchor.value)
  status.value = `Enabled camera preview (${requestId})`
}

function disablePreview() {
  const gameEngine = window.__A4D_WASM__?.game_engine
  if (!gameEngine?.disable_camera_preview) {
    status.value = 'WASM camera preview not available'
    return
  }
  const requestId = gameEngine.disable_camera_preview()
  status.value = `Disabled camera preview (${requestId})`
}

function pickFile() { fileRef.value?.click() }
async function onFile(e: Event) {
  const input = e.target as HTMLInputElement
  const f = input.files?.[0]
  if (!f) return
  status.value = `Starting from file: ${f.name}`
  await cameraService.startWithVideoFile(f, { width: 640, height: 480 })
  status.value = 'Streaming from file (JS)'
}
</script>

<style scoped>
</style>
