import { reactive, watch } from 'vue'

export interface SharedSettings {
  music_enabled: boolean
  bgm_volume: number // 0..1
  sfx_volume: number // 0..1
}

const STORAGE_KEY = 'a4d-settings-v1'

function loadSettings(): SharedSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      return {
        music_enabled: Boolean(parsed.music_enabled ?? true),
        bgm_volume: clamp01(Number(parsed.bgm_volume ?? 0.6)),
        sfx_volume: clamp01(Number(parsed.sfx_volume ?? 0.8)),
      }
    }
  } catch (e) {
    console.warn('Failed to load settings, using defaults', e)
  }
  return { music_enabled: true, bgm_volume: 0.6, sfx_volume: 0.8 }
}

function clamp01(v: number): number { return Math.max(0, Math.min(1, isFinite(v) ? v : 0)) }

const settings = reactive<SharedSettings>(loadSettings())

function persist() {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(settings)) } catch { /* ignore */ }
}

function sendToBevy() {
  const wasm = window.__A4D_WASM__
  if (!wasm?.send_js_to_bevy_event) return
  try {
    const event = {
      type: 'SettingsUpdated',
      request_id: `settings-${Date.now()}`,
      settings: { ...settings },
    }
    wasm.send_js_to_bevy_event(JSON.stringify(event))
  } catch (e) {
    console.warn('Failed to send settings to Bevy', e)
  }
}

// Persist and send to engine on change (debounced via microtask by Vue)
watch(settings, () => { persist(); sendToBevy() }, { deep: true })

export function useSettings() {
  return { settings, persist, sendToBevy }
}

