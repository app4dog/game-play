<template>
  <div class="game-canvas-container">
    <canvas
      id="game-canvas"
      ref="gameCanvas"
      class="game-canvas"
      @touchstart="handleTouch"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
      @mousedown="handleMouse"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseEnd"
    />
    
    <!-- Game overlay UI -->
    <div class="game-overlay">
      <div class="score-display">
        {{ gameState.score }} pts | Level {{ gameState.level }}
      </div>
      
      <q-btn
        v-if="gameState.isPaused"
        color="primary"
        label="Resume"
        @click="resumeGame"
        class="resume-btn"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { Ref } from 'vue'
// 🤓 Import WASM types from clean re-export
import type { GameEngine as WasmGameEngine } from '../types/wasm-types'

// GameEngineApi now imported from unified types

interface GameState {
  score: number
  level: number
  isPaused: boolean
}

// Props and emits
const emit = defineEmits<{
  gameReady: []
  gameError: [error: string]
  scoreChanged: [score: number]
  audioReady: []
  loadingProgress: [progress: { loaded: number, total: number, phase: string }]
}>()

// Expose game engine for other components
const getGameEngine = () => gameEngine

// Audio context management for browser autoplay policy
const audioContextInitialized = ref(false)
let webAudioCtx: AudioContext | null = null
let bgmSource: AudioBufferSourceNode | null = null
let bgmGain: GainNode | null = null

// Function to initialize AudioContext after user gesture
const initializeAudioContext = () => {
  if (audioContextInitialized.value || !wasmModule) return Promise.resolve()
  
  // Check for global audio disable flag
  if (window.__A4D_DISABLE_AUDIO__ === true) {
    console.debug('🔇 Audio globally disabled via __A4D_DISABLE_AUDIO__')
    audioContextInitialized.value = true // Mark as initialized to prevent retries
    return Promise.resolve()
  }
  
  return new Promise<void>((resolve) => {
    try {
      // Only create AudioContext if we don't have one, and only during actual user interaction
      const Ctx = window.AudioContext || window.webkitAudioContext
      if (Ctx && !webAudioCtx) {
        // Create a simple context that will be suspended initially
        webAudioCtx = new Ctx()
        
        // Ensure the context starts in a suspended state to comply with autoplay policy
        if (webAudioCtx.state === 'running') {
          console.debug('🎵 AudioContext created in running state, suspending to comply with autoplay policy')
          void webAudioCtx.suspend()
        }
      }
      
      // Resume the context - this should only succeed if called from a user gesture
      if (webAudioCtx && webAudioCtx.state === 'suspended') {
        webAudioCtx.resume().then(() => {
          console.debug('🎵 AudioContext successfully resumed after user gesture')
        }).catch((error) => {
          console.warn('⚠️ AudioContext resume failed (expected if no user gesture):', error)
        })
      }
      
      // Check if the WASM module has an audio context resume function
      const wasm = window.__A4D_WASM__
      if (wasm?.send_js_to_bevy_event) {
        // Send a user gesture event to Bevy to initialize AudioContext
        const userGestureEvent = {
          type: 'UserGesture',
          request_id: `user-gesture-${Date.now()}`,
          timestamp: Date.now()
        }
        wasm.send_js_to_bevy_event(JSON.stringify(userGestureEvent))
      }
      
      audioContextInitialized.value = true
      console.log('🎵 AudioContext initialized after user gesture')
      emit('audioReady')
      resolve()
    } catch (error) {
      console.warn('⚠️ Failed to initialize AudioContext:', error)
      resolve() // Don't reject, just log and continue
    }
  })
}

// Export for parent component access (consolidated)

// Reactive state
const gameCanvas: Ref<HTMLCanvasElement | null> = ref(null)
const gameState = ref<GameState>({
  score: 0,
  level: 1,
  isPaused: false
})

// Simple sprite-sheet loading & fallback rendering (if WASM is unavailable)
type SpriteSheet = {
  name: string
  url: string
  frameWidth: number
  frameHeight: number
  frameCount: number
  frameRate: number // frames per second
  image?: HTMLImageElement
}

const spriteConfig: SpriteSheet[] = [
  {
    name: 'Chirpy',
    url: `${import.meta.env.BASE_URL}assets/sprites/bird-animation.png`,
    frameWidth: 1000,
    frameHeight: 1000,
    frameCount: 6,
    frameRate: 10,
  },
  {
    name: 'Bouncy',
    url: `${import.meta.env.BASE_URL}assets/sprites/bunny-sprite-sheet.png`,
    frameWidth: 128,
    frameHeight: 128,
    frameCount: 2,
    frameRate: 2,
  },
]

const spriteLoadInfo = ref<{ total: number; loaded: number; failed: string[] }>(
  { total: spriteConfig.length, loaded: 0, failed: [] }
)
// Keep a local RAF id for potential future cancellation (intentionally unused)
// eslint-disable-next-line @typescript-eslint/no-unused-vars
let _rafIdLocal: number | null = null

// 🤓 Use direct import for clean type reference
let gameEngine: WasmGameEngine | null = null
let wasmModule: UnifiedWasmModule | null = null

// Touch/mouse interaction state
let isInteracting = false
let lastInteractionPos = { x: 0, y: 0 }

// Initialize game engine
onMounted(async () => {
  try {
    // Load WASM module (will be built from Rust)
    // Prefer global preload via index.html; fallback to injecting a module script dynamically
    // Prefer game-engine/ in prod; fallback to wasm/ in dev
    const base = import.meta.env.BASE_URL
    const candidates = [
      `${base}game-engine/app4dog_game_engine.js`,
      `${base}wasm/app4dog_game_engine.js`
    ]
    let moduleUrl = candidates[0]!
    console.debug('[A4D][WASM] glue candidates', candidates)

    // Guard: verify public/game-engine assets exist before initializing
    const assetExists = async (url: string): Promise<{ ok: boolean; status?: number }> => {
      try {
        const res = await fetch(url, { method: 'HEAD' })
        return { ok: res.ok, status: res.status }
      } catch {
        return { ok: false }
      }
    }
    // Try each candidate until both JS + WASM are present
    let selected: { jsOk: boolean; wasmOk: boolean; jsUrl: string; wasmUrl: string } | null = null
    for (const cand of candidates) {
      const wasmUrl = cand.replace('app4dog_game_engine.js', 'app4dog_game_engine_bg.wasm')
      const [jsHead, wasmHead] = await Promise.all([assetExists(cand), assetExists(wasmUrl)])
      console.debug('[A4D][WASM] HEAD', { js: { url: cand, ...jsHead }, wasm: { url: wasmUrl, ...wasmHead } })
      if (jsHead.ok && wasmHead.ok) {
        selected = { jsOk: jsHead.ok, wasmOk: wasmHead.ok, jsUrl: cand, wasmUrl }
        break
      }
    }
    if (!selected) {
      const friendly = 'WASM assets missing. Build with ./scripts/build-wasm.sh. Expected in public/game-engine/ or public/wasm/.'
      console.error(friendly, { candidates })
      emit('gameError', friendly)
      return
    }
    moduleUrl = selected.jsUrl

    // Obtain the module exports
    let mod = window.__A4D_WASM__
    if (!mod) {
      // Dynamically inject a module script that imports and exposes the glue
      console.debug('[A4D][WASM] injecting module script for glue')
      const script = document.createElement('script')
      script.type = 'module'
      script.textContent = `import * as m from '${moduleUrl}'; window.__A4D_WASM__ = m;`
      document.head.appendChild(script)
      // Wait briefly for the module to load
      mod = await new Promise<UnifiedWasmModule | undefined>((resolve) => {
        const start = performance.now()
        const check = () => {
          const m = window.__A4D_WASM__
          if (m) resolve(m)
          else if (performance.now() - start > 3000) resolve(undefined)
          else setTimeout(check, 50)
        }
        check()
      })
      if (!mod) {
        const msg = 'WASM JS glue failed to load. Verify public/game-engine assets and host permissions.'
        console.error(msg)
        emit('gameError', msg)
        return
      }
      console.debug('[A4D][WASM] glue module loaded via dynamic script')
    }
    wasmModule = mod
    
    // Track WASM loading progress
    console.debug('[A4D][WASM] initializing module default() with progress tracking')
    emit('loadingProgress', { loaded: 0, total: 100, phase: 'Initializing WASM module...' })
    
    if (wasmModule) {
      // Create progress tracking wrapper for WASM initialization
      const wasmUrl = selected.wasmUrl
      
      // Fetch WASM file with progress tracking
      emit('loadingProgress', { loaded: 0, total: 100, phase: 'Downloading game engine...' })
      
      try {
        const response = await fetch(wasmUrl)
        if (!response.ok) {
          throw new Error(`Failed to fetch WASM: ${response.status}`)
        }
        
        const contentLength = response.headers.get('content-length')
        const total = contentLength ? parseInt(contentLength, 10) : 0
        
        if (total > 0) {
          const reader = response.body?.getReader()
          let loaded = 0
          const chunks: Uint8Array[] = []
          
          if (reader) {
            while (true) {
              const { done, value } = await reader.read()
              if (done) break
              
              chunks.push(value)
              loaded += value.length
              
              const progressPercent = Math.round((loaded / total) * 100)
              emit('loadingProgress', { 
                loaded, 
                total, 
                phase: `Downloading game engine... ${progressPercent}% (${Math.round(loaded/1024/1024)}MB / ${Math.round(total/1024/1024)}MB)`
              })
            }
            
            // Combine chunks into single array
            const wasmBytes = new Uint8Array(loaded)
            let offset = 0
            for (const chunk of chunks) {
              wasmBytes.set(chunk, offset)
              offset += chunk.length
            }
            
            emit('loadingProgress', { loaded: total, total, phase: 'Initializing game engine...' })
            
            // Initialize WASM with the downloaded bytes
            await wasmModule.default(wasmBytes)
          } else {
            // Fallback if streaming not supported
            emit('loadingProgress', { loaded: 0, total: 100, phase: 'Downloading game engine...' })
            await wasmModule.default(wasmUrl)
          }
        } else {
          // Fallback if content-length not available
          emit('loadingProgress', { loaded: 0, total: 100, phase: 'Downloading game engine...' })
          await wasmModule.default(wasmUrl)
        }
      } catch (fetchError) {
        console.warn('Progress tracking failed, falling back to standard loading:', fetchError)
        emit('loadingProgress', { loaded: 0, total: 100, phase: 'Loading game engine...' })
        await wasmModule.default() // Fallback to standard loading
      }
      console.debug('[A4D][WASM] module initialized')
      
      gameEngine = new wasmModule.GameEngine()
      // Expose game engine globally for camera debug panel and other components
      if (window.__A4D_WASM__) {
        try {
          // Delete existing property if it exists to avoid redefinition errors
          if ('game_engine' in window.__A4D_WASM__) {
            delete window.__A4D_WASM__.game_engine
          }
          Object.defineProperty(window.__A4D_WASM__, 'game_engine', {
            value: gameEngine,
            writable: true,
            configurable: true
          })
          
          // Also expose the standalone critter functions for CritterSelection component
          if (wasmModule.critters_ready) {
            // Delete existing property if it exists to avoid redefinition errors
            if ('critters_ready' in window.__A4D_WASM__) {
              delete window.__A4D_WASM__.critters_ready
            }
            Object.defineProperty(window.__A4D_WASM__, 'critters_ready', {
              value: wasmModule.critters_ready,
              writable: true,
              configurable: true
            })
          }
          if (wasmModule.get_available_critters) {
            // Delete existing property if it exists to avoid redefinition errors
            if ('get_available_critters' in window.__A4D_WASM__) {
              delete window.__A4D_WASM__.get_available_critters
            }
            Object.defineProperty(window.__A4D_WASM__, 'get_available_critters', {
              value: wasmModule.get_available_critters,
              writable: true,
              configurable: true
            })
          }
        } catch (error) {
          console.warn('Could not add properties to WASM module, creating new reference:', error)
          // Fallback: create a new extensible object with the WASM module + game engine + functions
          const originalWasm = window.__A4D_WASM__
          const newWasmModule: UnifiedWasmModule = Object.create(null)
          
          // Copy enumerable properties safely
          if (originalWasm) {
            for (const key in originalWasm) {
              if (key !== 'game_engine' && key !== 'critters_ready' && key !== 'get_available_critters') {
                try {
                  newWasmModule[key] = originalWasm[key]
                } catch (copyError) {
                  console.debug(`Skipping property ${key} during fallback copy:`, copyError)
                }
              }
            }
          }
          
          // Add our properties
          newWasmModule.game_engine = gameEngine
          if (wasmModule.critters_ready) {
            newWasmModule.critters_ready = wasmModule.critters_ready
          }
          if (wasmModule.get_available_critters) {
            newWasmModule.get_available_critters = wasmModule.get_available_critters
          }
          
          window.__A4D_WASM__ = newWasmModule
        }
      }
    } else {
      throw new Error('WASM module not available')
    }
    
    if (!gameEngine) {
      throw new Error('Game engine not initialized')
    }
    gameEngine.start_game()
    
    // Set up canvas
    if (gameCanvas.value) {
      const canvas = gameCanvas.value
      canvas.width = canvas.offsetWidth
      canvas.height = canvas.offsetHeight
    }
    
    emit('gameReady')
    console.log('🎮 Game engine initialized successfully')
  } catch (error) {
    console.error('❌ Failed to initialize game engine:', error)
    emit('gameError', `Failed to load game: ${String(error)}`)
  }

  // Fallback: load critter sprite sheets and render simple animation if WASM is not active
  await preloadSprites()
  if (!gameEngine) startFallbackRenderLoop()
})

onUnmounted(() => {
  if (gameEngine) {
    gameEngine.free?.() // Clean up WASM resources
  }
})

// Touch handling for pet interactions
const handleTouch = async (event: TouchEvent) => {
  event.preventDefault()
  const touch = event.touches[0]!
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  
  // Initialize AudioContext on first user gesture
  await initializeAudioContext()
  
  isInteracting = true
  lastInteractionPos = {
    x: touch.clientX - rect.left,
    y: touch.clientY - rect.top
  }
  
  // Send tap interaction to game engine
  sendInteractionToGame('tap', lastInteractionPos)
}

const handleTouchMove = (event: TouchEvent) => {
  if (!isInteracting) return
  
  event.preventDefault()
  const touch = event.touches[0]!
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  
  const currentPos = {
    x: touch.clientX - rect.left,
    y: touch.clientY - rect.top
  }
  
  // Calculate swipe direction
  const swipeDirection = {
    x: currentPos.x - lastInteractionPos.x,
    y: currentPos.y - lastInteractionPos.y
  }
  
  sendInteractionToGame('swipe', currentPos, swipeDirection)
  lastInteractionPos = currentPos
}

const handleTouchEnd = () => {
  isInteracting = false
}

// Mouse handling (for testing on desktop)
const handleMouse = async (event: MouseEvent) => {
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  
  // Initialize AudioContext on first user gesture
  await initializeAudioContext()
  
  isInteracting = true
  lastInteractionPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top
  }
  
  sendInteractionToGame('tap', lastInteractionPos)
}

const handleMouseMove = (event: MouseEvent) => {
  if (!isInteracting) return
  
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  const currentPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top
  }
  
  const swipeDirection = {
    x: currentPos.x - lastInteractionPos.x,
    y: currentPos.y - lastInteractionPos.y
  }
  
  sendInteractionToGame('swipe', currentPos, swipeDirection)
  lastInteractionPos = currentPos
}

const handleMouseEnd = () => {
  isInteracting = false
}

// Send interaction to Rust game engine
const sendInteractionToGame = (
  type: 'tap' | 'swipe' | 'hold',
  position: { x: number; y: number },
  direction?: { x: number; y: number }
) => {
  if (!gameEngine) {
    console.warn('🐾 Pet interaction ignored: Game engine not initialized')
    return
  }
  
  console.log(`🐾 Pet interaction: ${type} at (${position.x}, ${position.y})`)
  
  // Call the WASM game engine
  try {
    const dx = direction?.x ?? 0
    const dy = direction?.y ?? 0
    
    // Check if the method exists on the game engine
    if (gameEngine.handle_interaction) {
      gameEngine.handle_interaction(type, position.x, position.y, dx, dy)
      console.log(`✅ Interaction sent to WASM: ${type}`)
    } else {
      console.warn('⚠️ handle_interaction method not found on game engine')
      console.log('Available methods:', Object.getOwnPropertyNames(gameEngine))
    }
  } catch (error) {
    console.error('❌ Failed to send interaction to game engine:', error)
  }
}

// Game controls
const resumeGame = () => {
  gameState.value.isPaused = false
  gameEngine?.start_game?.()
}

const pauseGame = () => {
  gameState.value.isPaused = true
  gameEngine?.pause_game?.()
}

// Expose methods to parent components (consolidated)
defineExpose({
  // Game control methods
  pauseGame,
  resumeGame,
  resetGame: () => gameEngine?.reset_game?.(),
  
  // Audio context initialization for user gesture compliance
  initializeAudioContext,
  // Background music controls (Web Audio; no-op if unavailable)
  startBackgroundMusic: async (url?: string): Promise<boolean> => {
    // Global kill-switch
    if (window.__A4D_DISABLE_BGM__ === true) {
      console.warn('BGM globally disabled via __A4D_DISABLE_BGM__')
      return false
    }
    // Ensure audio context is ready
    await initializeAudioContext()
    const ctx = webAudioCtx
    if (!ctx) { console.warn('No WebAudio context; skipping BGM'); return false }

    // Stop existing
    if (bgmSource) { try { bgmSource.stop(0) } catch { /* ignore */ } bgmSource.disconnect(); bgmSource = null }
    if (!bgmGain) { bgmGain = ctx.createGain(); bgmGain.gain.value = 0.5; bgmGain.connect(ctx.destination) }

    // Choose candidate URL
    const base = import.meta.env.BASE_URL
    const candidates = [
      url,
      `${base}assets/audio/bgm/theme.mp3`,
      `${base}assets/audio/positive/yipee.ogg`, // fallback short loop
    ].filter(Boolean) as string[]

    for (const cand of candidates) {
      try {
        const res = await fetch(cand, { mode: 'cors' })
        if (!res.ok) continue
        const buf = await res.arrayBuffer()
        const audioBuffer = await ctx.decodeAudioData(buf.slice(0))
        const src = ctx.createBufferSource()
        src.buffer = audioBuffer
        src.loop = true
        if (!bgmGain) { bgmGain = ctx.createGain(); bgmGain.gain.value = 0.5; bgmGain.connect(ctx.destination) }
        if (bgmGain) src.connect(bgmGain)
        src.start(0)
        bgmSource = src
        console.log('🎼 BGM started:', cand)
        return true
      } catch (e) {
        console.warn('BGM candidate failed:', cand, e)
      }
    }
    console.warn('No BGM candidates playable')
    return false
  },
  stopBackgroundMusic: () => {
    if (bgmSource) { try { bgmSource.stop(0) } catch { /* ignore */ } bgmSource.disconnect(); bgmSource = null }
  },
  setBackgroundMusicVolume: (v: number) => {
    if (bgmGain) bgmGain.gain.value = Math.max(0, Math.min(1, v))
  },
  
  // Game engine access methods
  getGameEngine,
  // New normalized API: pass canonical string id
  loadCritterById: (id: string) => {
    if (gameEngine?.load_critter_by_id) {
      gameEngine.load_critter_by_id(id)
    } else if (gameEngine?.load_critter) {
      // Fallback to legacy signature if needed
      gameEngine.load_critter(0, id, '')
    }
  },
  getCritterInfo: () => {
    return gameEngine?.get_critter_info?.() || null
  }
})

// ---- Sprite loading & fallback rendering ----
function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error(`Failed to load image: ${url}`))
    img.src = url
  })
}

async function preloadSprites(): Promise<void> {
  spriteLoadInfo.value = { total: spriteConfig.length, loaded: 0, failed: [] }
  await Promise.all(
    spriteConfig.map(async (s) => {
      try {
        // Attempt HEAD for nicer diagnostics (ignore failures)
        try { await fetch(s.url, { method: 'HEAD' }) } catch { /* ignore */ }
        const img = await loadImage(s.url)
        s.image = img
        spriteLoadInfo.value.loaded += 1
      } catch {
        spriteLoadInfo.value.failed.push(`${s.name}: ${s.url}`)
      }
    })
  )
  if (spriteLoadInfo.value.failed.length) {
    console.warn('Some sprites failed to load. Place real PNGs under public/assets/sprites/', spriteLoadInfo.value.failed)
  }
}

function startFallbackRenderLoop(): void {
  const canvas = gameCanvas.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const start = performance.now()
  const draw = (t: number) => {
    const elapsed = (t - start) / 1000
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    // Layout sprites horizontally with spacing
    const margin = 24
    // Scale down large frames to a reasonable on-screen size
    const targetSize = Math.max(64, Math.min(160, Math.floor(canvas.height * 0.25)))
    let x = margin
    const y = Math.max(16, Math.floor(canvas.height / 2 - targetSize / 2))

    for (const s of spriteConfig) {
      if (!s.image) { x += 128 + margin; continue }
      const effectiveFps = Math.min(60, Math.max(1, s.frameRate * 1.75))
      const frame = Math.floor(elapsed * effectiveFps) % Math.max(1, s.frameCount)
      const sx = frame * s.frameWidth
      const sy = 0
      // Draw scaled down to targetSize x targetSize (maintain square)
      ctx.drawImage(
        s.image,
        sx, sy, s.frameWidth, s.frameHeight,
        x, y, targetSize, targetSize
      )
      // Label
      ctx.fillStyle = 'rgba(255,255,255,0.85)'
      ctx.font = '12px sans-serif'
      ctx.fillText(s.name, x, y + targetSize + 14)
      x += targetSize + margin
    }

    _rafIdLocal = requestAnimationFrame(draw)
  }
  _rafIdLocal = requestAnimationFrame(draw)
}
</script>

<style scoped lang="scss">
.game-canvas-container {
  position: relative;
  width: 100%;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
}

.game-canvas {
  width: 100%;
  height: 100%;
  display: block;
  touch-action: none; // Prevent scrolling
  user-select: none;
}

.game-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 10;
}

.score-display {
  position: absolute;
  top: 20px;
  right: 20px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 18px;
  font-weight: 600;
}

.resume-btn {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  pointer-events: auto;
}
</style>
