<template>
  <q-card class="critter-selection">
    <q-card-section class="text-center">
      <div class="text-h5">🎭 Choose Your Critter</div>
      <div class="text-subtitle2">Select an anthropomorphic friend for your pet to play with</div>
    </q-card-section>
    
    <q-card-section>
      <div class="critter-grid">
        <div
          v-for="critter in availableCritters"
          :key="critter.id"
          class="critter-card"
          :class="{ 
            'selected': selectedCritter === critter.id,
            'locked': !critter.unlocked
          }"
          @click="selectCritter(critter)"
        >
          <div class="critter-image">
            <SpritePreview
              :name="critter.name"
              :url="critter.imagePath"
              :frame-width="critter.frameWidth"
              :frame-height="critter.frameHeight"
              :frame-count="critter.frameCount"
              :frame-rate="critter.frameRate"
              :frames="critter.frames"
              :size="80"
            />
            
            <!-- Lock overlay for locked critters -->
            <div v-if="!critter.unlocked" class="lock-overlay">
              <q-icon name="lock" size="2em" color="grey-6" />
              <div class="unlock-text">Level {{ critter.unlockLevel }}</div>
            </div>
          </div>
          
          <div class="critter-info">
            <h6 class="critter-name">{{ critter.name }}</h6>
            <div class="critter-species">{{ critter.species }}</div>
            
            <!-- Stats bars -->
            <div class="critter-stats">
              <div class="stat">
                <q-icon name="speed" size="sm" />
                <q-linear-progress 
                  :value="critter.stats.speed / 200" 
                  color="blue"
                  size="4px"
                  class="stat-bar"
                />
              </div>
              
              <div class="stat">
                <q-icon name="mood" size="sm" />
                <q-linear-progress 
                  :value="critter.stats.playfulness" 
                  color="orange"
                  size="4px"
                  class="stat-bar"
                />
              </div>
              
              <div class="stat">
                <q-icon name="pets" size="sm" />
                <q-linear-progress 
                  :value="critter.stats.obedience" 
                  color="green"
                  size="4px"
                  class="stat-bar"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </q-card-section>
    
    <q-card-actions align="right">
      <q-btn flat label="Cancel" @click="$emit('close')" />
      <q-btn 
        color="primary" 
        label="Select"
        :disabled="!selectedCritter || !selectedCritterData?.unlocked"
        @click="confirmSelection"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuasar } from 'quasar'
import SpritePreview from './SpritePreview.vue'

interface CritterStats {
  speed: number
  playfulness: number
  obedience: number
  energy: number
}

interface Critter {
  id: string
  name: string
  species: string
  imagePath: string
  unlocked: boolean
  unlockLevel: number
  stats: CritterStats
  description: string
  frameWidth: number
  frameHeight: number
  frameCount: number
  frameRate: number
  frames: Array<{ x: number; y: number }>
}

const $q = useQuasar()

// Emit events
const emit = defineEmits<{
  critterSelected: [critterName: string]
  close: []
}>()

// Selected critter state
const selectedCritter = ref<string | null>(null)

// Mock critter data (in real app, this would come from Rust WASM module)
const availableCritters = ref<Critter[]>([
  {
    id: 'chirpy',
    name: 'Chirpy',
    species: 'Bird',
    imagePath: '/assets/sprites/bird-animation.png',
    unlocked: true,
    unlockLevel: 1,
    stats: {
      speed: 150,
      playfulness: 0.8,
      obedience: 0.6,
      energy: 100
    },
    description: 'A cheerful bird who loves to fly around and play catch!',
    // Bird sheet: 3000x2000 with 6 frames at 1000x1000 (3x2)
    frameWidth: 1000,
    frameHeight: 1000,
    frameCount: 6,
    frameRate: 10,
    frames: [
      { x: 0, y: 0 }, { x: 1000, y: 0 }, { x: 2000, y: 0 },
      { x: 0, y: 1000 }, { x: 1000, y: 1000 }, { x: 2000, y: 1000 }
    ]
  },
  {
    id: 'bouncy',
    name: 'Bouncy',
    species: 'Bunny',
    imagePath: '/assets/sprites/bunny-sprite-sheet.png',
    unlocked: true, // Will be level-gated in real game
    unlockLevel: 2,
    stats: {
      speed: 120,
      playfulness: 0.9,
      obedience: 0.7,
      energy: 120
    },
    description: 'An energetic bunny who hops around and loves to be chased!',
    // Bunny sheet: 512x512; frames at (0,0) and (256,384) of 128x128
    frameWidth: 128,
    frameHeight: 128,
    frameCount: 2,
    frameRate: 2,
    frames: [
      { x: 0, y: 0 }, { x: 256, y: 384 }
    ]
  }
])

// Computed selected critter data
const selectedCritterData = computed(() => {
  return availableCritters.value.find(c => c.id === selectedCritter.value)
})

// Select a critter
const selectCritter = (critter: Critter) => {
  if (!critter.unlocked) {
    $q.notify({
      type: 'warning',
      message: `${critter.name} unlocks at level ${critter.unlockLevel}!`,
      position: 'top'
    })
    return
  }
  
  selectedCritter.value = critter.id
  
  $q.notify({
    type: 'info',
    message: critter.description,
    position: 'top',
    timeout: 2000
  })
}

// Confirm selection
const confirmSelection = () => {
  if (!selectedCritterData.value) return
  
  emit('critterSelected', selectedCritterData.value.name)
}

// No image error handler needed; SpritePreview handles fallbacks
</script>

<style scoped lang="scss">
.critter-selection {
  min-width: 400px;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.critter-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}

.critter-card {
  border: 2px solid transparent;
  border-radius: 8px;
  padding: 1rem;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
  background: $grey-1;
  
  &:hover:not(.locked) {
    border-color: $primary;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  
  &.selected {
    border-color: $primary;
    background: rgba($primary, 0.1);
  }
  
  &.locked {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

.critter-image {
  position: relative;
  width: 80px;
  height: 80px;
  margin: 0 auto 1rem;
  border-radius: 50%;
  overflow: hidden;
  background: white;
  
  canvas {
    width: 100%;
    height: 100%;
    display: block;
  }
}

.lock-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.unlock-text {
  color: white;
  font-size: 0.8rem;
  margin-top: 0.25rem;
}

.critter-info {
  text-align: center;
}

.critter-name {
  margin: 0 0 0.25rem 0;
  font-size: 1.1rem;
  font-weight: 600;
}

.critter-species {
  color: $grey-6;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
}

.critter-stats {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.stat {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
}

.stat-bar {
  flex: 1;
}
</style>
