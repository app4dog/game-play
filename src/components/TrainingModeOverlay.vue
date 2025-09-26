<template>
  <div class="training-mode-overlay">
    <div class="camera-surface">
      <canvas ref="cameraCanvas" class="camera-canvas"></canvas>
    </div>

    <div class="overlay">
      <div class="overlay-top">
        <div class="timer-display">{{ formattedElapsed }}</div>
      </div>
      <div class="overlay-bottom">
        <q-btn color="primary" label="Exit" class="exit-btn" @click="onExitClick" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useQuasar } from 'quasar'
import { cameraService, type CameraFrame } from '../services/CameraService'

const emit = defineEmits<{
  (event: 'exit', reason?: 'timeout' | 'manual' | 'error'): void
}>()

const $q = useQuasar()

const cameraCanvas = ref<HTMLCanvasElement | null>(null)
const elapsedSeconds = ref(0)

const formattedElapsed = computed(() => {
  const minutes = Math.floor(elapsedSeconds.value / 60)
  const seconds = elapsedSeconds.value % 60
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

const TRAINING_DURATION_SECONDS = 15

let intervalId: number | null = null
let trainingStartedAt = 0
let frameCleanup: (() => void) | null = null
let ctx: CanvasRenderingContext2D | null = null
let rawCanvas: HTMLCanvasElement | null = null
let rawCtx: CanvasRenderingContext2D | null = null
let rawImageData: ImageData | null = null
let pendingFrame: CameraFrame | null = null
let drawHandle = 0
let cameraActive = false
let dispatchedStartEvent = false
let exiting = false
let viewportWidth = 0
let viewportHeight = 0
let resizeHandler: (() => void) | null = null

function startTimer() {
  trainingStartedAt = Date.now()
  elapsedSeconds.value = 0
  intervalId = window.setInterval(() => {
    const diff = Math.floor((Date.now() - trainingStartedAt) / 1000)
    elapsedSeconds.value = diff
    if (diff >= TRAINING_DURATION_SECONDS) {
      void exitTraining('timeout')
    }
  }, 1000)
}

function stopTimer() {
  if (intervalId !== null) {
    window.clearInterval(intervalId)
    intervalId = null
  }
}

function handleFrame(frame: CameraFrame) {
  pendingFrame = frame
  if (drawHandle === 0) {
    drawHandle = window.requestAnimationFrame(drawFrame)
  }
}

function drawFrame() {
  drawHandle = 0
  const frame = pendingFrame
  pendingFrame = null
  if (!frame) return

  const canvas = cameraCanvas.value
  if (!canvas) return

  if (!ctx) {
    ctx = canvas.getContext('2d', { willReadFrequently: true })
    if (!ctx) return
    ctx.imageSmoothingEnabled = true
  }

  if (!rawCanvas) {
    rawCanvas = document.createElement('canvas')
    rawCtx = rawCanvas.getContext('2d', { willReadFrequently: true })
  }
  if (!rawCtx || !rawCanvas) return

  if (rawCanvas.width !== frame.width || rawCanvas.height !== frame.height) {
    rawCanvas.width = frame.width
    rawCanvas.height = frame.height
    rawImageData = null
  }

  if (!rawImageData || rawImageData.width !== frame.width || rawImageData.height !== frame.height) {
    rawImageData = rawCtx.createImageData(frame.width, frame.height)
  }

  const dest = rawImageData.data
  const src = frame.data
  for (let srcIdx = 0, destIdx = 0; srcIdx < src.length; srcIdx += 3, destIdx += 4) {
    dest[destIdx] = src[srcIdx] ?? 0
    dest[destIdx + 1] = src[srcIdx + 1] ?? 0
    dest[destIdx + 2] = src[srcIdx + 2] ?? 0
    dest[destIdx + 3] = 255
  }

  rawCtx.putImageData(rawImageData, 0, 0)

  if (viewportWidth === 0 || viewportHeight === 0) {
    updateCanvasSize()
  }

  const dpr = window.devicePixelRatio || 1
  const displayWidth = viewportWidth
  const displayHeight = viewportHeight
  ctx.save()
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, displayWidth, displayHeight)

  const scale = Math.max(displayWidth / rawCanvas.width, displayHeight / rawCanvas.height)
  const drawWidth = rawCanvas.width * scale
  const drawHeight = rawCanvas.height * scale
  const offsetX = (displayWidth - drawWidth) / 2
  const offsetY = (displayHeight - drawHeight) / 2

  ctx.drawImage(rawCanvas, offsetX, offsetY, drawWidth, drawHeight)
  ctx.restore()
}

function dispatchCameraStart(width: number, height: number) {
  if (typeof window === 'undefined') return
  const eventPayload = {
    type: 'CameraStart',
    request_id: `training-${Date.now()}`,
    width,
    height
  }
  window.dispatchEvent(new CustomEvent('bevy-to-js-event', { detail: JSON.stringify(eventPayload) }))
  dispatchedStartEvent = true
}

function dispatchCameraStop() {
  if (!dispatchedStartEvent || typeof window === 'undefined') return
  const eventPayload = {
    type: 'CameraStop',
    request_id: `training-${Date.now()}`
  }
  window.dispatchEvent(new CustomEvent('bevy-to-js-event', { detail: JSON.stringify(eventPayload) }))
  dispatchedStartEvent = false
}

function dispatchCameraPreviewDisable() {
  if (typeof window === 'undefined') return
  const wasm = window.__A4D_WASM__
  const payload = {
    type: 'CameraPreviewToggle',
    request_id: `training-preview-${Date.now()}`,
    enabled: false,
    scale: 0,
    anchor: 'TopRight',
    margin: 0,
    mirror_x: false,
  }
  try {
    wasm?.send_js_to_bevy_event?.(JSON.stringify(payload))
  } catch (error) {
    console.warn('Failed to disable camera preview', error)
  }
  try {
    window.dispatchEvent(new CustomEvent('bevy-to-js-event', { detail: JSON.stringify(payload) }))
  } catch (err) {
    console.warn('Failed to dispatch preview toggle event', err)
  }
}

function updateCanvasSize() {
  const canvas = cameraCanvas.value
  if (!canvas || typeof window === 'undefined') return
  viewportWidth = window.innerWidth
  viewportHeight = window.innerHeight
  const dpr = window.devicePixelRatio || 1
  canvas.style.width = `${viewportWidth}px`
  canvas.style.height = `${viewportHeight}px`
  canvas.width = Math.round(viewportWidth * dpr)
  canvas.height = Math.round(viewportHeight * dpr)
  if (ctx) {
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.imageSmoothingEnabled = true
  }
}

async function cleanupCamera() {
  if (drawHandle) {
    window.cancelAnimationFrame(drawHandle)
    drawHandle = 0
  }
  pendingFrame = null
  ctx = null
  rawImageData = null
  rawCanvas = null
  rawCtx = null

  if (frameCleanup) {
    frameCleanup()
    frameCleanup = null
  }

  if (cameraActive) {
    try {
      await cameraService.stop()
    } catch (error) {
      console.warn('Failed to stop camera service:', error)
    }
    cameraActive = false
  }

  dispatchCameraStop()
  dispatchCameraPreviewDisable()
}

async function exitTraining(reason: 'timeout' | 'manual' | 'error' = 'manual') {
  if (exiting) return
  exiting = true
  stopTimer()
  await cleanupCamera()
  emit('exit', reason)
}

function onExitClick() {
  void exitTraining('manual')
}

onMounted(async () => {
  const canvas = cameraCanvas.value
  if (!canvas) {
    console.error('Training mode canvas unavailable')
    emit('exit', 'error')
    return
  }

  updateCanvasSize()
  if (typeof window !== 'undefined') {
    resizeHandler = () => updateCanvasSize()
    window.addEventListener('resize', resizeHandler)
  }

  frameCleanup = cameraService.addFrameListener(handleFrame)

  try {
    await cameraService.start({ width: 640, height: 480 })
    cameraActive = true
    dispatchCameraStart(640, 480)
  } catch (error) {
    console.error('Failed to start training mode camera:', error)
    $q.notify({ type: 'negative', message: 'Unable to start camera for training mode', position: 'top' })
    await cleanupCamera()
    emit('exit', 'error')
    return
  }

  startTimer()
})

onBeforeUnmount(async () => {
  stopTimer()
  await cleanupCamera()
  if (resizeHandler && typeof window !== 'undefined') {
    window.removeEventListener('resize', resizeHandler)
    resizeHandler = null
  }
})
</script>

<style scoped>
.training-mode-overlay {
  position: fixed;
  inset: 0;
  background-color: #000;
  z-index: 2000;
  overflow: hidden;
}

.camera-surface {
  position: absolute;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

.camera-canvas {
  width: 100%;
  height: 100%;
  display: block;
  background: #000;
}

.overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  pointer-events: none;
}

.overlay-top {
  display: flex;
  justify-content: center;
  padding: 24px;
}

.timer-display {
  font-size: 1.5rem;
  font-weight: 600;
  color: #fff;
  background: rgba(0, 0, 0, 0.4);
  padding: 8px 16px;
  border-radius: 999px;
}

.overlay-bottom {
  display: flex;
  justify-content: flex-end;
  padding: 24px;
}

.exit-btn {
  pointer-events: auto;
}
</style>
