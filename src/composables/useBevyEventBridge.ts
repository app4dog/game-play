// TypeScript side of the Bevy <-> JS event bridge
// Handles audio, bluetooth, and other platform integration events

import { ref, onMounted, onUnmounted } from 'vue'
// Unified WASM types are now globally declared

// TypeScript types matching the Rust events (manually synced)
export interface BevyToJsEvent {
  type: 'PlayAudio' | 'BluetoothScan' | 'TestEvent'
  request_id?: string
  sound_id?: string
  volume?: number
  device_filter?: string
  message?: string
}

export interface JsToBevyEvent {
  type: 'AudioCompleted' | 'BluetoothScanCompleted' | 'TestEventResponse' | 'SettingsUpdated'
  request_id: string
  success?: boolean
  error_message?: string
  duration_seconds?: number
  devices_found?: string[]
  response_data?: string
  settings?: {
    music_enabled: boolean
    bgm_volume: number
    sfx_volume: number
  }
}

// WASM interface imported from unified types

export class BevyEventBridge {
  private eventHandlers: Map<string, (event: BevyToJsEvent) => void | Promise<void>> = new Map()
  private isInitialized = false
  private boundEventHandler?: (event: Event) => void
  private errorCount: Map<string, number> = new Map()
  
  constructor() {
    this.setupEventHandlers()
  }

  private setupEventHandlers() {
    // Audio event handler
    this.eventHandlers.set('PlayAudio', this.handlePlayAudio.bind(this))
    this.eventHandlers.set('BluetoothScan', this.handleBluetoothScan.bind(this))
    this.eventHandlers.set('TestEvent', this.handleTestEvent.bind(this))
  }

  init() {
    if (this.isInitialized) return
    
    // ✅ REQUIRED: Error boundary wrapper for async handlers (b00t pattern)
    this.boundEventHandler = (event: Event) => {
      this.handleBevyEvent(event).catch(error => {
        // 🔥 MANDATORY: Generic error/crash logger
        console.error('🔥 b00t platform integration error [bevy-event-bridge]:', {
          error: error.message,
          stack: error.stack,
          eventType: event.type,
          timestamp: new Date().toISOString(),
          context: 'bevy-event-bridge'
        })
        
        // 📊 REQUIRED: Error metrics for Resource state management
        this.incrementErrorCount('bevy_event_failed')
      })
    }
    
    window.addEventListener('bevy-to-js-event', this.boundEventHandler)
    this.isInitialized = true
    console.log('🔗 BevyEventBridge initialized')
  }

  destroy() {
    if (!this.isInitialized) return
    
    if (this.boundEventHandler) {
      window.removeEventListener('bevy-to-js-event', this.boundEventHandler)
    }
    this.isInitialized = false
    console.log('🔗 BevyEventBridge destroyed')
  }

  private async handleBevyEvent(event: Event) {
    const customEvent = event as CustomEvent
    try {
      const bevyEvent: BevyToJsEvent = JSON.parse(customEvent.detail)
      console.log('📩 Received Bevy event:', bevyEvent)
      
      const handler = this.eventHandlers.get(bevyEvent.type)
      if (handler) {
        await handler(bevyEvent)
      } else {
        console.warn('❌ No handler for Bevy event type:', bevyEvent.type)
        this.sendErrorToBevy(bevyEvent.request_id || 'unknown', `No handler for event type: ${bevyEvent.type}`)
      }
    } catch (error) {
      console.error('❌ Failed to handle Bevy event:', error, event)
    }
  }

  private async handlePlayAudio(event: BevyToJsEvent): Promise<void> {
    if (!event.request_id || !event.sound_id) {
      console.error('❌ PlayAudio event missing required fields:', event)
      return
    }

    console.log(`🎵 Playing audio: ${event.sound_id} (volume: ${event.volume || 1.0})`)
    
    const startTime = performance.now()
    try {
      const duration = await this.playAudioFile(event.sound_id, event.volume || 1.0)
      const elapsed = performance.now() - startTime
      
      console.log(`✅ Audio completed: ${event.sound_id} (${elapsed.toFixed(0)}ms)`)
      
      this.sendToBevy({
        type: 'AudioCompleted',
        request_id: event.request_id,
        success: true,
        duration_seconds: duration,
      })
    } catch (error) {
      const elapsed = performance.now() - startTime
      console.error(`❌ Audio failed: ${event.sound_id} (${elapsed.toFixed(0)}ms)`, error)
      
      this.sendToBevy({
        type: 'AudioCompleted',
        request_id: event.request_id,
        success: false,
        error_message: error instanceof Error ? error.message : String(error),
      })
    }
  }

  private handleBluetoothScan(event: BevyToJsEvent): void {
    console.log(`🔵 Bluetooth scan requested: ${event.device_filter}`)
    // TODO: Implement bluetooth scanning
    this.sendToBevy({
      type: 'BluetoothScanCompleted',
      request_id: event.request_id || 'unknown',
      success: false,
      error_message: 'Bluetooth not implemented yet',
      devices_found: [],
    })
  }

  private handleTestEvent(event: BevyToJsEvent): void {
    console.log(`🧪 Test event: ${event.message}`)
    this.sendToBevy({
      type: 'TestEventResponse',
      request_id: event.request_id || 'test',
      response_data: `Echo: ${event.message}`,
    })
  }

  private async playAudioFile(soundId: string, volume: number): Promise<number> {
    // Build robust candidate list against both positive/ and ui/ folders and both extensions
    const base = import.meta.env.BASE_URL
    // Strip any leading slashes to avoid double slashes with BASE_URL
    const id = soundId.replace(/^\/+/, '')
    const bare = id.replace(/\.(mp3|ogg|wav)$/i, '')
    const direct: string | undefined = /^https?:\/\//i.test(id)
      ? id
      : id.startsWith('assets/')
        ? `${base}${id}`
        : id.startsWith('/')
          ? `${base}${id.replace(/^\//,'')}`
          : undefined
    const candidates = [
      direct,
      `${base}assets/audio/positive/${id}`,
      `${base}assets/audio/positive/${bare}.mp3`,
      `${base}assets/audio/positive/${bare}.ogg`,
      `${base}assets/audio/ui/${id}`,
      `${base}assets/audio/ui/${bare}.mp3`,
      `${base}assets/audio/ui/${bare}.ogg`,
      `${base}assets/audio/positive/yipee.ogg`, // known in repo
      `${base}assets/audio/positive/yipee.mp3`,
      'https://interactive-examples.mdn.mozilla.net/media/cc0-audio/t-rex-roar.mp3',
      'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3',
    ].filter((u): u is string => Boolean(u))

    let lastError: unknown = null
    for (const url of candidates) {
      try {
        const audio = new Audio()
        audio.preload = 'metadata'
        audio.crossOrigin = 'anonymous'
        audio.volume = Math.max(0, Math.min(1, volume))
        audio.src = url

        // Best-effort metadata load (don’t fail if it times out)
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
        console.log('🔊 Playing candidate:', url, 'duration:', dur)
        return dur
      } catch (err) {
        lastError = err
        console.warn(`Failed to play ${url}:`, err)
        // try next
      }
    }

    throw new Error(`All audio candidates failed. Last error: ${String(lastError)}`)
  }

  private sendToBevy(event: JsToBevyEvent): void {
    const wasm = window.__A4D_WASM__
    if (!wasm?.send_event_to_bevy && !wasm?.send_js_to_bevy_event) {
      console.error('❌ WASM bridge not available - cannot send event to Bevy')
      return
    }

    try {
      const eventJson = JSON.stringify(event)
      console.log('📨 Sending event to Bevy:', eventJson)
      
      // Try the preferred function first
      if (wasm.send_js_to_bevy_event) {
        wasm.send_js_to_bevy_event(eventJson)
      } else if (wasm.send_event_to_bevy) {
        wasm.send_event_to_bevy(eventJson)
      }
    } catch (error) {
      console.error('❌ Failed to send event to Bevy:', error)
    }
  }

  private sendErrorToBevy(requestId: string, errorMessage: string): void {
    // Send a generic error response based on request ID pattern
    if (requestId.startsWith('audio-')) {
      this.sendToBevy({
        type: 'AudioCompleted',
        request_id: requestId,
        success: false,
        error_message: errorMessage,
      })
    } else {
      console.error('❌ Cannot send error for unknown request type:', requestId, errorMessage)
    }
  }
  
  private incrementErrorCount(errorType: string): void {
    const currentCount = this.errorCount.get(errorType) || 0
    this.errorCount.set(errorType, currentCount + 1)
    console.warn(`📊 b00t error metrics: ${errorType} count = ${currentCount + 1}`)
  }
}

// Vue composable for easy integration
export function useBevyEventBridge() {
  const bridge = ref<BevyEventBridge | null>(null)
  const isReady = ref(false)

  onMounted(() => {
    bridge.value = new BevyEventBridge()
    bridge.value.init()
    isReady.value = true
  })

  onUnmounted(() => {
    if (bridge.value) {
      bridge.value.destroy()
      bridge.value = null
    }
    isReady.value = false
  })

  return {
    bridge: bridge.value,
    isReady
  }
}

// Export singleton for global use
export const globalBevyEventBridge = new BevyEventBridge()
