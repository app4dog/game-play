// Native Audio Handler for b00t AudioPlugin integration
// Handles audio requests from Bevy AudioPlugin via custom events

import { ref, onMounted, onUnmounted } from 'vue'
// Unified WASM types are now globally declared

// TypeScript types matching the Rust AudioPlugin events
export interface AudioRequest {
  type: 'Play' | 'Stop' | 'SetVolume' | 'Test'
  request_id: string
  sound_id?: string
  context?: 'Enter' | 'Exit' | 'UI' | 'Critter' | 'Ambient' | 'Test'
  volume?: number
  loop_audio?: boolean
  test_type?: string
}

export interface AudioResponse {
  type: 'PlayCompleted' | 'Stopped' | 'VolumeChanged' | 'TestCompleted'
  request_id: string
  success?: boolean
  duration_seconds?: number
  error_message?: string
  new_volume?: number
  result?: string
}

// WASM interface imported from unified types

export class NativeAudioHandler {
  private eventHandlers: Map<string, (request: AudioRequest) => void | Promise<void>> = new Map()
  private isInitialized = false
  private audioCache: Map<string, HTMLAudioElement> = new Map()
  private contextVolumes: Map<string, number> = new Map()
  private boundEventHandler?: (event: Event) => void
  private errorCount: Map<string, number> = new Map()
  
  constructor() {
    this.setupEventHandlers()
    this.setupContextVolumes()
  }

  private setupEventHandlers() {
    this.eventHandlers.set('Play', this.handlePlayRequest.bind(this))
    this.eventHandlers.set('Stop', this.handleStopRequest.bind(this))
    this.eventHandlers.set('SetVolume', this.handleVolumeRequest.bind(this))
    this.eventHandlers.set('Test', this.handleTestRequest.bind(this))
  }

  private setupContextVolumes() {
    // Set default volumes for different audio contexts
    this.contextVolumes.set('Enter', 0.8)
    this.contextVolumes.set('Exit', 0.7)
    this.contextVolumes.set('UI', 0.6)
    this.contextVolumes.set('Critter', 0.9)
    this.contextVolumes.set('Ambient', 0.4)
    this.contextVolumes.set('Test', 0.8)
  }

  init() {
    if (this.isInitialized) return
    
    // ✅ REQUIRED: Error boundary wrapper for async handlers (b00t pattern)
    this.boundEventHandler = (event: Event) => {
      this.handleAudioRequest(event).catch(error => {
        // 🔥 MANDATORY: Generic error/crash logger
        console.error('🔥 b00t platform integration error [native-audio-handler]:', {
          error: error.message,
          stack: error.stack,
          eventType: event.type,
          timestamp: new Date().toISOString(),
          context: 'native-audio-handler'
        })
        
        // 📊 REQUIRED: Error metrics for Resource state management
        this.incrementErrorCount('native_audio_failed')
      })
    }
    
    window.addEventListener('bevy-audio-request', this.boundEventHandler)
    this.isInitialized = true
    console.log('🎵 NativeAudioHandler initialized')
  }

  destroy() {
    if (!this.isInitialized) return
    
    if (this.boundEventHandler) {
      window.removeEventListener('bevy-audio-request', this.boundEventHandler)
    }
    this.cleanupAudioCache()
    this.isInitialized = false
    console.log('🎵 NativeAudioHandler destroyed')
  }

  private async handleAudioRequest(event: Event) {
    const customEvent = event as CustomEvent
    try {
      const raw = JSON.parse(customEvent.detail)
      let audioRequest: AudioRequest
      if (raw && typeof raw === 'object' && typeof raw.type === 'string') {
        audioRequest = raw as AudioRequest
      } else {
        const coerced = this.coerceLegacyRequest(raw)
        if (!coerced) throw new Error('Unsupported audio request shape')
        audioRequest = coerced
      }
      console.log('🎵 Received native audio request:', audioRequest)
      
      const handler = this.eventHandlers.get(audioRequest.type)
      if (handler) {
        await handler(audioRequest)
      } else {
        console.warn('❌ No handler for audio request type:', audioRequest.type)
        this.sendErrorToBevy(audioRequest.request_id, `No handler for request type: ${audioRequest.type}`)
      }
    } catch (error) {
      console.error('❌ Failed to handle native audio request:', error, event)
    }
  }

  private coerceLegacyRequest(raw: unknown): AudioRequest | null {
    if (!raw || typeof raw !== 'object') return null
    const keys = ['Play', 'Stop', 'SetVolume', 'Test'] as const
    for (const k of keys) {
      const rec = (raw as Record<string, unknown>)[k]
      if (rec && typeof rec === 'object') {
        return { type: k, ...(rec as Record<string, unknown>) } as AudioRequest
      }
    }
    return null
  }

  private async handlePlayRequest(request: AudioRequest): Promise<void> {
    if (!request.sound_id) {
      console.error('❌ PlayRequest missing sound_id:', request)
      return
    }

    console.log(`🎵 Playing native audio: ${request.sound_id} (context: ${request.context})`)
    
    const startTime = performance.now()
    try {
      const duration = await this.playAudioFile(
        request.sound_id, 
        request.volume || 0.8,
        request.context || 'Test',
        request.loop_audio || false
      )
      const elapsed = performance.now() - startTime
      
      console.log(`✅ Native audio completed: ${request.sound_id} (${elapsed.toFixed(0)}ms)`)
      
      this.sendToBevy({
        type: 'PlayCompleted',
        request_id: request.request_id,
        success: true,
        duration_seconds: duration,
      })
    } catch (error) {
      const elapsed = performance.now() - startTime
      console.error(`❌ Native audio failed: ${request.sound_id} (${elapsed.toFixed(0)}ms)`, error)
      
      this.sendToBevy({
        type: 'PlayCompleted',
        request_id: request.request_id,
        success: false,
        error_message: error instanceof Error ? error.message : String(error),
      })
    }
  }

  private handleStopRequest(request: AudioRequest): void {
    console.log(`⏹️ Stopping audio: ${request.sound_id || 'all'}`)
    
    try {
      if (request.sound_id) {
        // Stop specific sound
        const audio = this.audioCache.get(request.sound_id)
        if (audio) {
          audio.pause()
          audio.currentTime = 0
        }
      } else {
        // Stop all sounds
        this.audioCache.forEach((audio) => {
          audio.pause()
          audio.currentTime = 0
        })
      }

      this.sendToBevy({
        type: 'Stopped',
        request_id: request.request_id,
        success: true,
      })
    } catch (error) {
      console.error('❌ Failed to stop audio:', error)
      this.sendToBevy({
        type: 'Stopped',
        request_id: request.request_id,
        success: false,
      })
    }
  }

  private handleVolumeRequest(request: AudioRequest): void {
    const newVolume = request.volume || 1.0
    console.log(`🔊 Setting global volume: ${newVolume}`)
    
    // Apply volume to all cached audio elements
    this.audioCache.forEach((audio) => {
      audio.volume = newVolume
    })

    this.sendToBevy({
      type: 'VolumeChanged',
      request_id: request.request_id,
      new_volume: newVolume,
    })
  }

  private async handleTestRequest(request: AudioRequest): Promise<void> {
    console.log(`🧪 Running audio test: ${request.test_type}`)
    
    try {
      // Run a simple audio test
      await this.playAudioFile('yipee', 0.8, 'Test', false)
      
      this.sendToBevy({
        type: 'TestCompleted',
        request_id: request.request_id,
        result: `Test passed: ${request.test_type}`,
      })
    } catch (error) {
      this.sendToBevy({
        type: 'TestCompleted',
        request_id: request.request_id,
        result: `Test failed: ${error instanceof Error ? error.message : String(error)}`,
      })
    }
  }

  private async playAudioFile(
    soundId: string, 
    volume: number, 
    context: string,
    loop: boolean
  ): Promise<number> {
    // Get context-specific volume adjustment
    const contextVolume = this.contextVolumes.get(context) || 1.0
    const effectiveVolume = Math.max(0, Math.min(1, volume * contextVolume))

    // Build audio file candidates based on sound ID and context
    const base = import.meta.env.BASE_URL
    const candidates = this.buildAudioCandidates(soundId, base)

    let lastError: unknown = null
    
    for (const url of candidates) {
      try {
        // Check cache first (per soundId logical id)
        let audio = this.audioCache.get(soundId)
        if (!audio) {
          audio = new Audio()
          audio.preload = 'metadata'
          audio.crossOrigin = 'anonymous'
          this.audioCache.set(soundId, audio)
        }

        audio.src = url
        audio.volume = effectiveVolume
        audio.loop = loop

        // Best-effort metadata load with timeout; don't fail hard here
        await new Promise<void>((resolve) => {
          let done = false
          const onLoaded = () => { if (!done) { done = true; cleanup(); resolve() } }
          const onError = () => { if (!done) { done = true; cleanup(); resolve() } }
          const cleanup = () => {
            audio.removeEventListener('loadedmetadata', onLoaded)
            audio.removeEventListener('error', onError)
          }
          audio.addEventListener('loadedmetadata', onLoaded, { once: true })
          audio.addEventListener('error', onError, { once: true })
          setTimeout(() => { if (!done) { done = true; cleanup(); resolve() } }, 800)
        })

        await audio.play()
        const dur = Number.isFinite(audio.duration) ? audio.duration : 0
        console.log('🔊 Native playing candidate:', url, 'duration:', dur)
        return dur
      } catch (err) {
        lastError = err
        console.warn(`Failed to play ${url}:`, err)
        // Continue to next candidate
      }
    }

    // If we get here, all candidates failed
    throw new Error(`All audio candidates failed for ${soundId}. Last error: ${String(lastError)}`)
  }

  private buildAudioCandidates(soundId: string, base: string): string[] {
    const candidates: string[] = []
    const id = soundId.replace(/^\/+/, '')
    // If an absolute URL or explicit assets path is provided, try it first
    if (/^https?:\/\//i.test(soundId)) {
      candidates.push(soundId)
    } else if (id.startsWith('assets/')) {
      candidates.push(`${base}${id}`)
    } else if (soundId.startsWith('/')) {
      candidates.push(`${base}${soundId.replace(/^\//,'')}`)
    }
    
    // Primary candidates based on sound registry
    if (soundId === 'enter_area' || soundId === 'enter') {
      candidates.push(
        `${base}assets/audio/ui/enter_chime.mp3`,
        `${base}assets/audio/ui/enter_chime.ogg`,
        `${base}assets/audio/ui/chime.mp3`,
      )
    } else if (soundId === 'exit_area' || soundId === 'exit') {
      candidates.push(
        `${base}assets/audio/ui/exit_chime.mp3`,
        `${base}assets/audio/ui/exit_chime.ogg`,
        `${base}assets/audio/ui/chime.mp3`,
      )
    } else if (soundId === 'yipee') {
      candidates.push(
        `${base}assets/audio/positive/yipee.mp3`,
        `${base}assets/audio/positive/yipee.ogg`,
      )
    } else {
      // Generic sound ID handling
      candidates.push(
        `${base}assets/audio/${soundId}.mp3`,
        `${base}assets/audio/${soundId}.ogg`,
        `${base}assets/audio/ui/${soundId}.mp3`,
        `${base}assets/audio/positive/${soundId}.mp3`,
      )
    }

    // Fallback to test sounds
    candidates.push(
      `${base}assets/audio/positive/yipee.mp3`,
      `${base}assets/audio/positive/yipee.ogg`,
      'https://interactive-examples.mdn.mozilla.net/media/cc0-audio/t-rex-roar.mp3',
    )

    return candidates
  }

  private sendToBevy(response: AudioResponse): void {
    const wasm = window.__A4D_WASM__
    if (!wasm?.send_audio_response && !wasm?.send_audio_response_to_bevy) {
      console.error('❌ WASM audio response function not available')
      return
    }

    try {
      const responseJson = JSON.stringify(response)
      console.log('📨 Sending audio response to Bevy:', responseJson)
      
      // Try the preferred function first
      if (wasm.send_audio_response_to_bevy) {
        wasm.send_audio_response_to_bevy(responseJson)
      } else if (wasm.send_audio_response) {
        wasm.send_audio_response(responseJson)
      }
    } catch (error) {
      console.error('❌ Failed to send audio response to Bevy:', error)
    }
  }

  // Legacy response conversion removed now that WASM expects tagged shapes

  private sendErrorToBevy(requestId: string, errorMessage: string): void {
    this.sendToBevy({
      type: 'PlayCompleted',
      request_id: requestId,
      success: false,
      error_message: errorMessage,
    })
  }

  private cleanupAudioCache(): void {
    this.audioCache.forEach((audio) => {
      audio.pause()
      audio.src = ''
    })
    this.audioCache.clear()
  }
  
  private incrementErrorCount(errorType: string): void {
    const currentCount = this.errorCount.get(errorType) || 0
    this.errorCount.set(errorType, currentCount + 1)
    console.warn(`📊 b00t error metrics: ${errorType} count = ${currentCount + 1}`)
  }
}

// Vue composable for easy integration
export function useNativeAudio() {
  const handler = ref<NativeAudioHandler | null>(null)
  const isReady = ref(false)

  onMounted(() => {
    handler.value = new NativeAudioHandler()
    handler.value.init()
    isReady.value = true
  })

  onUnmounted(() => {
    if (handler.value) {
      handler.value.destroy()
      handler.value = null
    }
    isReady.value = false
  })

  return {
    handler: handler.value,
    isReady
  }
}

// Export singleton for global use
export const globalNativeAudioHandler = new NativeAudioHandler()
