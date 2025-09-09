<template>
  <q-card class="critter-selection">
    <q-card-section class="text-center">
      <div class="text-h5">🎭 Choose Your Critter</div>
      <div class="text-subtitle2">Select an anthropomorphic friend for your pet to play with</div>
    </q-card-section>
    
    <q-card-section>
      <!-- Loading state -->
      <div v-if="isLoading" class="text-center q-py-lg">
        <q-spinner color="primary" size="3em" />
        <div class="text-h6 q-mt-md">Loading critters...</div>
        <div class="text-caption">Initializing WASM critter-keeper</div>
      </div>
      
      <!-- Critter grid -->
      <div v-else class="critter-grid">
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
        :disabled="isLoading || !selectedCritter || !selectedCritterData?.unlocked"
        @click="confirmSelection"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import SpritePreview from './SpritePreview.vue'
import init, { CritterKeeperWasm } from 'critter-keeper'

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

interface FrameLayout {
  frame_size: [number, number]
  layout: 'Horizontal' | 'Vertical' | { Grid: { cols: number; rows: number } }
  frame_count: number
  image_size: [number, number]
}

const $q = useQuasar()

// Emit events
const emit = defineEmits<{
  critterSelected: [critter: Critter]
  close: []
}>()

// Selected critter state
const selectedCritter = ref<string | null>(null)

// WASM critter keeper instance
const critterKeeper = ref<CritterKeeperWasm | null>(null)
const availableCritters = ref<Critter[]>([])
const isLoading = ref(true)

// Initialize WASM critter-keeper
onMounted(async () => {
  try {
    // Initialize WASM module with explicit URL
    await init('/wasm/critter_keeper_bg.wasm')
    
    // Load pointer-based catalog RON (id -> file path)
    const catalogResponse = await fetch('/critters/catalog.ron')
    const catalogText = await catalogResponse.text()

    // Extract critter file references from RON using a simple regex
    // Example matches: "chirpy_bird": "chirpy_bird.ron"
    const entryRegex = /"([^"]+)"\s*:\s*"([^"]+\.ron)"/g
    const baseDir = '/critters/'
    const entries: Array<{ id: string; url: string }> = []
    let match: RegExpExecArray | null
    while ((match = entryRegex.exec(catalogText)) !== null) {
      const id = match[1]
      const refPath = match[2]
      // Guard: ensure we have both capture groups
      if (!id || !refPath) {
        continue
      }
      const url: string = refPath.startsWith('/') ? refPath : baseDir + refPath
      entries.push({ id, url })
    }

    // Fetch each critter RON and build a full catalog string compatible with WASM
    const packages: Array<{ id: string; ron: string }> = []
    for (const e of entries) {
      const resp = await fetch(e.url)
      const ron = (await resp.text()).trim()
      // Validate/extract id from the file (fallback to pointer id)
      const idMatch = ron.match(/id:\s*"([^"]+)"/)
      const fileId = idMatch?.[1] || e.id
      packages.push({ id: fileId, ron })
    }

    // Compose the final embedded catalog RON for the WASM interface
    let finalCatalogRon = `(
    critters: {
`
    for (const p of packages) {
      finalCatalogRon += `        "${p.id}": ${p.ron},\n`
    }
    finalCatalogRon += `    }
)`

    // Create critter keeper instance with composed catalog
    critterKeeper.value = new CritterKeeperWasm(
      '', // relative URLs for same-origin requests
      finalCatalogRon
    )
    
    // Load critter data from WASM module
    const critterIds = critterKeeper.value.list_critters()
    const critters: Critter[] = []
    
    for (const critterId of critterIds) {
      const metadata = JSON.parse(critterKeeper.value.get_critter_metadata(critterId) || '{}')
      const stats = JSON.parse(critterKeeper.value.get_stats(critterId) || '{}')
      const frameLayout = JSON.parse(critterKeeper.value.get_frame_layout(critterId) || '{}')
      const spriteUrl = critterKeeper.value.get_sprite_url(critterId) || ''
      const idleAnimation = JSON.parse(critterKeeper.value.get_animation(critterId, 'idle') || '{}')
      
      // Generate correct frame positions using animation frame indices
      const frames = generateFramePositions(frameLayout, idleAnimation.frames || [])
      console.log(`🐛 Debug ${critterId} frames:`, frames)
      console.log(`🐛 Debug ${critterId} frameLayout:`, frameLayout)
      console.log(`🐛 Debug ${critterId} idleAnimation:`, idleAnimation)
      
      // Log what coordinates we're generating
      console.log(`🎯 ${critterId} generated coordinates:`, frames.map(f => `(${f.x},${f.y})`).join(', '))
      
      // Map WASM data to expected Critter interface (maintaining compatibility)
      const critter: Critter = {
        id: critterId,
        name: metadata.name,
        species: metadata.species,
        imagePath: spriteUrl,
        unlocked: true, // TODO: Implement level gating
        unlockLevel: 1,
        stats: {
          speed: stats.base_speed,
          playfulness: stats.happiness_boost,
          obedience: 0.7, // Default value for compatibility
          energy: stats.energy
        },
        description: `A ${metadata.species.toLowerCase()} critter`,
        frameWidth: frameLayout.frame_size[0],
        frameHeight: frameLayout.frame_size[1], 
        frameCount: frameLayout.frame_count,
        frameRate: idleAnimation.fps || 2,
        frames: frames
      }
      
      critters.push(critter)
    }
    
    availableCritters.value = critters
    isLoading.value = false
    
  } catch (error) {
    console.error('Failed to initialize critter-keeper WASM:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to load critter data',
      position: 'top'
    })
    isLoading.value = false
  }
})

// Generate frame positions based on layout configuration and animation frame indices
function generateFramePositions(frameLayout: FrameLayout, animationFrames: number[]): Array<{ x: number; y: number }> {
  const { frame_size, layout } = frameLayout
  const [frameWidth, frameHeight] = frame_size
  
  if (typeof layout === 'object' && layout.Grid) {
    const { cols, rows } = layout.Grid
    
    // Generate all possible frame positions in the grid
    const allFrames: Array<{ x: number; y: number }> = []
    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols; col++) {
        allFrames.push({
          x: col * frameWidth,
          y: row * frameHeight
        })
      }
    }
    
    // Return positions for the specific animation frames
    // E.g. for bunny with frames [0, 14], return coordinates for grid positions 0 and 14
    return animationFrames.map((frameIndex): { x: number; y: number } => {
      if (frameIndex < allFrames.length && frameIndex >= 0) {
        const frame = allFrames[frameIndex]
        return frame || { x: 0, y: 0 }
      } else {
        // Fallback for invalid frame indices
        return { x: 0, y: 0 }
      }
    })
    
  } else if (layout === 'Horizontal') {
    return animationFrames.map(frameIndex => ({
      x: frameIndex * frameWidth,
      y: 0
    }))
  } else if (layout === 'Vertical') {
    return animationFrames.map(frameIndex => ({
      x: 0,
      y: frameIndex * frameHeight
    }))
  }
  
  return []
}

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
  
  emit('critterSelected', selectedCritterData.value)
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
