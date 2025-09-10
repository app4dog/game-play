<template>
  <div class="critter-selection">
    <div class="q-pa-md">
      <h4 class="text-h4 q-mb-md">Choose Your Critter</h4>
      
      <div v-if="isLoading" class="text-center q-pa-lg">
        <q-spinner-dots size="xl" color="primary" />
        <div class="text-caption">Loading critters...</div>
      </div>

      <div v-else class="row q-gutter-md">
        <div 
          v-for="critter in availableCritters" 
          :key="critter.id"
          class="col-12 col-sm-6 col-md-4"
        >
          <q-card 
            :class="['critter-card', { 'selected': selectedCritter === critter.id }]"
            clickable
            @click="selectedCritter = critter.id"
          >
            <q-card-section class="text-center">
              <div class="sprite-container q-mb-sm">
                <SpritePreview 
                  :name="critter.name"
                  :url="critter.sprite"
                  :frame-width="critter.frameWidth"
                  :frame-height="critter.frameHeight"
                  :frame-count="critter.frames.length"
                  :frame-rate="critter.idleFps"
                  :frames="critter.frames"
                />
              </div>
              <div class="text-h6">{{ critter.name }}</div>
              <div class="text-caption text-grey-7">{{ critter.species }}</div>
            </q-card-section>

            <q-card-section class="q-pt-none">
              <div class="stats-grid">
                <div class="stat-row">
                  <span class="stat-label">Base Speed:</span>
                  <span class="stat-value">{{ critter.stats.baseSpeed.toFixed(0) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">Energy:</span>
                  <span class="stat-value">{{ critter.stats.energy.toFixed(0) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">Happiness Boost:</span>
                  <span class="stat-value">{{ (critter.stats.happinessBoost * 10).toFixed(1) }}/10</span>
                </div>
              </div>
            </q-card-section>
          </q-card>
        </div>
      </div>

      <div v-if="!isLoading && selectedCritter" class="text-center q-mt-lg">
        <q-btn 
          color="primary" 
          size="lg" 
          @click="startGame"
          :disabled="!selectedCritter"
        >
          Start Playing with {{ selectedCritterName }}
        </q-btn>
      </div>
    </div>
  </div>
</template>

<style scoped>
.critter-selection {
  max-width: 1200px;
  margin: 0 auto;
}

.critter-card {
  transition: all 0.2s ease;
  border: 2px solid transparent;
}

.critter-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.critter-card.selected {
  border-color: var(--q-primary);
}

.sprite-container {
  background: #f5f5f5;
  border-radius: 8px;
  padding: 8px;
  display: inline-block;
}

.stats-grid {
  display: grid;
  gap: 4px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.85em;
}

.stat-label {
  color: var(--q-dark);
}

.stat-value {
  font-weight: bold;
  color: var(--q-primary);
}
</style>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import SpritePreview from './SpritePreview.vue'

interface CritterFrames { x: number; y: number }
interface CritterStats { baseSpeed: number; energy: number; happinessBoost: number }
interface Critter { id: string; name: string; species: string; sprite: string; frameWidth: number; frameHeight: number; idleFps: number; frames: CritterFrames[]; stats: CritterStats }

// WASM-exposed DTOs (source of truth)
interface WasmCritterFrame { x: number; y: number }
interface WasmCritterStats { baseSpeed: number; energy: number; happinessBoost: number }
interface WasmCritter { id: string; name: string; species: string; sprite: string; frameWidth: number; frameHeight: number; idleFps: number; frames: WasmCritterFrame[]; stats: WasmCritterStats }
interface WasmModule { critters_ready: () => boolean; get_available_critters: () => WasmCritter[] }

// Reactive state
const $q = useQuasar()
const selectedCritter = ref<string | null>(null)
const availableCritters = ref<Critter[]>([])
const isLoading = ref(true)

// Computed properties
const selectedCritterName = computed(() => {
  const critter = availableCritters.value.find(c => c.id === selectedCritter.value)
  return critter?.name || 'Unknown Critter'
})

// Initialize critter data - simplified for mobile build  
onMounted(async () => {
  try {
    // Pull from WASM-provided list: window.__A4D_WASM__ set by GameCanvas
    // and populated by the Bevy system after registry loads.
    const mod = (window as Window & { __A4D_WASM__?: Partial<WasmModule> }).__A4D_WASM__
    if (!mod || !mod.get_available_critters) {
      throw new Error('WASM module not available yet')
    }

    // Wait briefly for registry to initialize
    const waitUntil = async (fn: () => boolean, timeoutMs = 3000): Promise<boolean> => {
      const start = performance.now()
      return await new Promise(resolve => {
        const tick = () => {
          if (fn()) return resolve(true)
          if (performance.now() - start > timeoutMs) return resolve(false)
          setTimeout(tick, 50)
        }
        tick()
      })
    }
    await waitUntil(() => !!mod.critters_ready && !!mod.critters_ready())

    const list: WasmCritter[] = mod.get_available_critters?.() ?? []
    availableCritters.value = list.map((c) => ({
      id: c.id,
      name: c.name,
      species: c.species,
      sprite: c.sprite,
      frameWidth: Number(c.frameWidth ?? 64),
      frameHeight: Number(c.frameHeight ?? 64),
      idleFps: Number(c.idleFps ?? 4),
      frames: Array.isArray(c.frames) ? c.frames.map((f: WasmCritterFrame) => ({ x: Number(f.x), y: Number(f.y) })) : [],
      stats: {
        baseSpeed: Number(c.stats?.baseSpeed ?? 0),
        energy: Number(c.stats?.energy ?? 0),
        happinessBoost: Number(c.stats?.happinessBoost ?? 0)
      }
    }))

    if (availableCritters.value.length > 0) {
      selectedCritter.value = availableCritters.value[0]!.id
    }
  } catch (error) {
    console.error('Failed to load critter data from WASM:', error)
    $q.notify({ type: 'negative', message: 'Failed to load critters from engine' })
  } finally {
    isLoading.value = false
  }
})

// Emit selected critter to parent
const emit = defineEmits(['critter-selected'])

const startGame = () => {
  if (selectedCritter.value) {
    const critter = availableCritters.value.find(c => c.id === selectedCritter.value)
    if (critter) {
      emit('critter-selected', critter)
    }
  }
}
</script>
