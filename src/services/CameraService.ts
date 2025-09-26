// Cross-platform Camera service with Capacitor (Android) and Web fallbacks
import { Capacitor } from '@capacitor/core'
// - Prefers @capacitor-community/camera-preview when available
// - Falls back to getUserMedia + hidden <video>/<canvas>

export interface CameraFrame {
  data: Uint8Array
  width: number
  height: number
  ts: number
}

type CameraPreviewPlugin = {
  start: (opts: { position: 'rear' | 'front'; toBack: boolean; width?: number; height?: number }) => Promise<void>
  stop: () => Promise<void>
  captureSample: (opts: { quality?: number }) => Promise<{ value?: string; data?: string }>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any // Allow for any additional methods
}

// Runtime detection of Capacitor Camera Preview
async function loadCapacitorCameraPreview(): Promise<CameraPreviewPlugin | null> {
  // Skip on web; use getUserMedia instead
  const plat = Capacitor.getPlatform?.() ?? 'web'
  const isNative = plat === 'android' || plat === 'ios'
  console.log(`[CameraService] Platform: ${plat}, isNative: ${isNative}`)
  
  if (!isNative) {
    console.log('[CameraService] Not on native platform, skipping Capacitor plugin')
    return null
  }
  
  try {
    console.log('[CameraService] Attempting to load @capacitor-community/camera-preview...')
    // Dynamic import to avoid bundling when not present
    const { CameraPreview } = await import('@capacitor-community/camera-preview')
    console.log('[CameraService] Module loaded, plugin:', CameraPreview)
    
    // Log available methods - try multiple approaches for Proxy objects
    console.log('[CameraService] Available methods (getOwnPropertyNames):', Object.getOwnPropertyNames(CameraPreview))
    console.log('[CameraService] Available methods (keys):', Object.keys(CameraPreview))
    console.log('[CameraService] Has start:', 'start' in CameraPreview, typeof CameraPreview.start)
    console.log('[CameraService] Has stop:', 'stop' in CameraPreview, typeof CameraPreview.stop)
    console.log('[CameraService] Has captureSample:', 'captureSample' in CameraPreview, typeof CameraPreview.captureSample)
    console.log('[CameraService] Has capture:', 'capture' in CameraPreview, typeof CameraPreview.capture)
    
    if (CameraPreview && typeof CameraPreview.start === 'function') {
      console.log('[CameraService] Plugin validated successfully')
      return CameraPreview
    } else {
      console.warn('[CameraService] Plugin validation failed:', { 
        hasPlugin: !!CameraPreview, 
        hasStartMethod: CameraPreview && typeof CameraPreview.start === 'function' 
      })
    }
  } catch (error) {
    console.error('[CameraService] Failed to load Capacitor camera preview:', error)
    // Not available; fall through to web fallback
  }
  return null
}

export class CameraService {
  private plugin: CameraPreviewPlugin | null = null
  private running = false
  private width = 0
  private height = 0
  private captureHandle: number | null = null

  // Web fallback
  private videoEl: HTMLVideoElement | null = null
  private canvasEl: HTMLCanvasElement | null = null
  private ctx: CanvasRenderingContext2D | null = null
  private stream: MediaStream | null = null
  private pluginCanvas: HTMLCanvasElement | null = null
  private pluginCtx: CanvasRenderingContext2D | null = null

  async start(opts?: { width?: number; height?: number; rear?: boolean }): Promise<void> {
    if (this.running) {
      console.log('[CameraService] Already running, skipping start')
      return
    }
    
    console.log('[CameraService] Starting camera with options:', opts)
    this.plugin = await loadCapacitorCameraPreview()
    this.width = opts?.width || 640
    this.height = opts?.height || 480
    const rear = opts?.rear !== false
    
    console.log('[CameraService] Camera config:', { width: this.width, height: this.height, rear })

    if (this.plugin) {
      console.log('[CameraService] Using Capacitor plugin for camera')
      try {
        const position: 'rear' | 'front' = rear ? 'rear' : 'front'
        const startOptions = { position, toBack: true, width: this.width, height: this.height }
        console.log('[CameraService] Calling plugin.start with:', startOptions)
        
        await this.plugin.start(startOptions)
        console.log('[CameraService] Plugin.start() successful')
        
        this.running = true
        this.startCaptureLoopCapacitor()
        console.log('[CameraService] Capacitor camera started successfully')
        return
      } catch (err) {
        console.error('[CameraService] Capacitor preview failed, falling back to web:', err)
        this.plugin = null
      }
    } else {
      console.log('[CameraService] No Capacitor plugin available, using web fallback')
    }

    // Web fallback
    console.log('[CameraService] Setting up web stream fallback')
    await this.setupWebStream(this.width, this.height, rear)
    this.running = true
    this.startCaptureLoopWeb()
    console.log('[CameraService] Web camera started successfully')
  }

  async stop(): Promise<void> {
    this.running = false
    if (this.captureHandle != null) {
      cancelAnimationFrame(this.captureHandle)
      this.captureHandle = null
    }
    if (this.plugin) {
      try { await this.plugin.stop() } catch { /* noop */ }
    }
    if (this.stream) {
      this.stream.getTracks().forEach(t => t.stop())
    }
    this.stream = null
    // Clean up DOM nodes quietly
    if (this.videoEl?.parentElement) this.videoEl.parentElement.removeChild(this.videoEl)
    if (this.canvasEl?.parentElement) this.canvasEl.parentElement.removeChild(this.canvasEl)
    this.videoEl = null
    this.canvasEl = null
    this.ctx = null
  }

  private startCaptureLoopCapacitor() {
    console.log('[CameraService] Starting Capacitor capture loop')
    let frameCount = 0
    const tick = () => {
      const plugin = this.plugin
      if (!this.running || !plugin) {
        console.log('[CameraService] Capture loop stopped:', { running: this.running, hasPlugin: !!plugin })
        return
      }
      ;(async () => {
        try {
          const res = await plugin.captureSample({ quality: 50 })
          frameCount++
          
          // Log every 60th frame to avoid spam
          if (frameCount % 60 === 1) {
            console.log('[CameraService] Capture sample result:', { 
              frameCount,
              hasValue: !!res.value, 
              hasData: !!res.data,
              valueLength: res.value?.length,
              dataLength: res.data?.length 
            })
          }
          
          const b64 = (res.value || res.data || '').trim()
          if (b64) {
            const dataUrl = b64.startsWith('data:') ? b64 : `data:image/jpeg;base64,${b64}`
            const rgb = await this.decodeImageToRGB(dataUrl, this.width, this.height)
            this.emitFrame({ data: rgb, width: this.width, height: this.height, ts: performance.now() })
          } else if (frameCount % 60 === 1) {
            console.warn('[CameraService] Empty capture sample result')
          }
        } catch (e) {
          console.error('[CameraService] CameraPreview.captureSample failed:', e)
        }
      })().catch(() => {})
      this.captureHandle = requestAnimationFrame(tick)
    }
    this.captureHandle = requestAnimationFrame(tick)
  }

  private async setupWebStream(width: number, height: number, rear: boolean) {
    // Create hidden elements
    const container = document.createElement('div')
    container.style.position = 'fixed'
    container.style.left = '-10000px'
    container.style.top = '-10000px'
    container.style.width = '1px'
    container.style.height = '1px'
    container.style.overflow = 'hidden'
    document.body.appendChild(container)

    const video = document.createElement('video')
    video.autoplay = true
    video.playsInline = true
    video.muted = true
    container.appendChild(video)

    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    container.appendChild(canvas)

    const ctx = canvas.getContext('2d', { willReadFrequently: true })
    if (!ctx) throw new Error('Failed to create canvas 2d context')

    const constraints: MediaStreamConstraints = {
      video: {
        width: { ideal: width },
        height: { ideal: height },
        facingMode: rear ? { ideal: 'environment' } : { ideal: 'user' },
      },
      audio: false,
    }
    const stream = await navigator.mediaDevices.getUserMedia(constraints)
    video.srcObject = stream
    await new Promise<void>((resolve) => {
      video.onloadedmetadata = () => resolve()
    })

    this.videoEl = video
    this.canvasEl = canvas
    this.ctx = ctx
    this.stream = stream
  }

  private startCaptureLoopWeb() {
    const loop = () => {
      if (!this.running || !this.ctx || !this.canvasEl || !this.videoEl) return
      try {
        const { ctx, canvasEl, videoEl } = this
        ctx.drawImage(videoEl, 0, 0, canvasEl.width, canvasEl.height)
        const id = ctx.getImageData(0, 0, canvasEl.width, canvasEl.height)
        // Convert RGBA -> RGB to reduce size and match expected format
        const rgba = id.data
        const rgb = new Uint8Array((rgba.length / 4) * 3)
        for (let i = 0, j = 0; i < rgba.length; i += 4) {
          rgb[j++] = rgba[i] ?? 0
          rgb[j++] = rgba[i + 1] ?? 0
          rgb[j++] = rgba[i + 2] ?? 0
        }
        this.emitFrame({ data: rgb, width: canvasEl.width, height: canvasEl.height, ts: performance.now() })
      } catch (e) {
        console.warn('Web camera capture failed:', e)
      }
      this.captureHandle = requestAnimationFrame(loop)
    }
    this.captureHandle = requestAnimationFrame(loop)
  }

  // Decode base64/URL image to RGB bytes using offscreen canvas
  private async decodeImageToRGB(url: string, width: number, height: number): Promise<Uint8Array> {
    if (!this.pluginCanvas) {
      this.pluginCanvas = document.createElement('canvas')
      this.pluginCanvas.width = width
      this.pluginCanvas.height = height
      this.pluginCtx = this.pluginCanvas.getContext('2d', { willReadFrequently: true })
    }
    const ctx = this.pluginCtx
    if (!ctx || !this.pluginCanvas) throw new Error('No decode canvas')

    const img = await new Promise<HTMLImageElement>((resolve, reject) => {
      const im = new Image()
      im.onload = () => resolve(im)
      im.onerror = () => reject(new Error('image decode failed'))
      im.src = url
    })
    ctx.drawImage(img, 0, 0, width, height)
    const rgba = ctx.getImageData(0, 0, width, height).data
    const rgb = new Uint8Array((rgba.length / 4) * 3)
    for (let i = 0, j = 0; i < rgba.length; i += 4) {
      rgb[j++] = rgba[i] ?? 0
      rgb[j++] = rgba[i + 1] ?? 0
      rgb[j++] = rgba[i + 2] ?? 0
    }
    return rgb
  }

  // Optional: start from a user-provided video file for offline testing
  async startWithVideoFile(file: File, opts?: { width?: number; height?: number }): Promise<void> {
    if (this.running) await this.stop()
    this.width = opts?.width || 640
    this.height = opts?.height || 480

    const container = document.createElement('div')
    container.style.position = 'fixed'
    container.style.left = '-10000px'
    container.style.top = '-10000px'
    container.style.width = '1px'
    container.style.height = '1px'
    container.style.overflow = 'hidden'
    document.body.appendChild(container)

    const video = document.createElement('video')
    video.autoplay = true
    video.playsInline = true
    video.muted = true
    video.loop = true
    container.appendChild(video)

    const canvas = document.createElement('canvas')
    canvas.width = this.width
    canvas.height = this.height
    container.appendChild(canvas)
    const ctx = canvas.getContext('2d', { willReadFrequently: true })
    if (!ctx) throw new Error('Failed to create canvas 2d context')

    const url = URL.createObjectURL(file)
    video.src = url
    await new Promise<void>((resolve) => { video.onloadedmetadata = () => resolve() })
    await video.play().catch(() => {})

    this.videoEl = video
    this.canvasEl = canvas
    this.ctx = ctx
    this.stream = null
    this.running = true
    this.startCaptureLoopWeb()
  }

  private emitFrame(frame: CameraFrame) {
    const wasm = window.__A4D_WASM__
    const submit = wasm?.submit_camera_frame
    if (!submit) return
    try {
      submit(frame.width, frame.height, frame.data, frame.ts)
    } catch (e) {
      console.error('Failed to submit camera frame to WASM:', e)
    }
  }
}

export const cameraService = new CameraService()
