<template>
  <canvas ref="canvasEl" :width="size" :height="size" class="sprite-preview-canvas" />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watchEffect } from 'vue'

interface Props {
  name: string
  url: string
  frameWidth: number
  frameHeight: number
  frameCount: number
  frameRate: number
  size?: number
  frames?: Array<{ x: number; y: number }>
}

const props = defineProps<Props>()
const size = props.size ?? 80

const canvasEl = ref<HTMLCanvasElement | null>(null)
let rafId: number | null = null
let img: HTMLImageElement | null = null
let imgLoaded = false

function cancelLoop() {
  if (rafId != null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
}

function startLoop() {
  const canvas = canvasEl.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const start = performance.now()
  const draw = (t: number) => {
    const elapsed = (t - start) / 1000
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    if (imgLoaded && img) {
      const frame = Math.floor(elapsed * props.frameRate) % Math.max(1, props.frameCount)
      let sx = 0, sy = 0
  if (props.frames && props.frames.length > 0) {
        const f = props.frames[frame % props.frames.length]
        if (f) {
          sx = f.x
          sy = f.y
        }
      } else {
        // Row-major across the full image width
        const cols = Math.max(1, Math.floor(img.width / props.frameWidth))
        const col = frame % cols
        const row = Math.floor(frame / cols)
        sx = col * props.frameWidth
        sy = row * props.frameHeight
      }
      // Center the sprite in the canvas
      const dw = Math.min(props.frameWidth, canvas.width)
      const dh = Math.min(props.frameHeight, canvas.height)
      const dx = (canvas.width - dw) / 2
      const dy = (canvas.height - dh) / 2
      ctx.drawImage(img, sx, sy, props.frameWidth, props.frameHeight, dx, dy, dw, dh)
    } else {
      // Fallback: simple pulsing circle with name initial
      const r = (Math.sin(elapsed * 2) * 0.1 + 0.9) * (canvas.width / 2 - 4)
      ctx.fillStyle = '#ffffff'
      ctx.beginPath()
      ctx.arc(canvas.width / 2, canvas.height / 2, r, 0, Math.PI * 2)
      ctx.fill()
      ctx.fillStyle = 'rgba(0,0,0,0.7)'
      ctx.font = 'bold 18px sans-serif'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText(props.name.charAt(0), canvas.width / 2, canvas.height / 2)
    }

    rafId = requestAnimationFrame(draw)
  }

  cancelLoop()
  rafId = requestAnimationFrame(draw)
}

onMounted(() => {
  // Attempt to load the sprite image
  img = new Image()
  img.onload = () => { imgLoaded = true }
  img.onerror = () => { imgLoaded = false }
  img.src = props.url

  startLoop()
})

onUnmounted(() => {
  cancelLoop()
})

// Restart loop if size or url changes
watchEffect(() => {
  // Touch dependency to trigger rerun when props.url/size change
  void props.url; void size
})
</script>

<style scoped>
.sprite-preview-canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
