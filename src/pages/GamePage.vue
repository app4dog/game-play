<template>
  <q-page class="game-page">
    <!-- Game canvas is always mounted; we overlay loading/errors -->
    <GameCanvas
      ref="gameCanvas"
      @game-ready="onGameReady"
      @game-error="onGameError"
      @score-changed="onScoreChanged"
      class="full-height"
    />
    
    <!-- Loading overlay -->
    <q-inner-loading :showing="gameLoading">
      <div class="game-loading">
        <q-spinner-puff color="primary" size="4em" />
        <p class="loading-text">🐕 Loading App4.Dog Game...</p>
      </div>
    </q-inner-loading>
    
    <!-- Error overlay -->
    <q-dialog v-model="gameErrorDialog">
      <q-card class="game-error">
        <q-card-section class="text-center">
          <q-icon name="error" color="negative" size="4em" />
          <h3>Game Loading Error</h3>
          <p>{{ gameError }}</p>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn color="primary" label="Retry" @click="retryInit" />
        </q-card-actions>
      </q-card>
    </q-dialog>
    
    <!-- Game menu overlay -->
    <q-dialog v-model="showMenu">
      <q-card class="game-menu font-hero">
        <q-card-section class="text-center">
          <div class="text-h4">🐕 App4.Dog Game</div>
          <div class="text-subtitle2">Interactive Pet Training</div>
        </q-card-section>
        
        <q-card-section>
          <div class="menu-stats">
            <div class="stat-item">
              <q-icon name="stars" color="amber" />
              <span>High Score: {{ highScore }}</span>
            </div>
            <div class="stat-item">
              <q-icon name="trending_up" color="positive" />
              <span>Current Level: {{ currentLevel }}</span>
            </div>
          </div>
        </q-card-section>
        
        <q-card-section class="menu-buttons">
          <q-btn
            color="primary"
            label="Start Game"
            size="lg"
            @click="startGame"
            class="full-width q-mb-sm"
          />
          <q-btn
            color="positive"
            label="Play Test Sound"
            size="md"
            @click="playTestSound"
            class="full-width q-mb-sm"
          />
          <q-btn
            color="secondary"
            label="Select Critter"
            size="md"
            @click="showCritterSelection = true"
            class="full-width q-mb-sm"
          />
          <q-btn
            color="info"
            label="Training Mode"
            size="md"
            @click="startTrainingMode"
            class="full-width q-mb-sm"
          />
          <q-btn
            color="accent"
            label="Settings"
            size="md"
            @click="showSettings = true"
            class="full-width"
          />
        </q-card-section>
      </q-card>
    </q-dialog>
    
    <!-- Critter selection dialog -->
    <q-dialog v-model="showCritterSelection">
      <CritterSelection
        @critter-selected="onCritterSelected"
        @close="showCritterSelection = false"
      />
    </q-dialog>
    
    <!-- Settings dialog -->
    <q-dialog v-model="showSettings">
      <GameSettings
        @settings-changed="onSettingsChanged"
        @close="showSettings = false"
      />
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'
import GameCanvas from '../components/GameCanvas.vue'
// Type for methods exposed by GameCanvas via defineExpose
type GameCanvasExposed = {
  pauseGame: () => void
  resumeGame: () => void
  resetGame: () => void
  getGameEngine: () => unknown
  loadCritterById: (id: string) => void
  getCritterInfo: () => { id: number; name: string; species: string; happiness: number; energy: number } | null
}
import CritterSelection from '../components/CritterSelection.vue'
import GameSettings from '../components/GameSettings.vue'

const $q = useQuasar()

// Game state
const gameLoading = ref(true)
const gameError = ref<string | null>(null)
const gameErrorDialog = ref(false)
const showMenu = ref(true)
const showCritterSelection = ref(false)
const showSettings = ref(false)

// Game stats
const highScore = ref(0)
const currentLevel = ref(1)
const currentScore = ref(0)

// Game canvas reference
const gameCanvas = ref<GameCanvasExposed | null>(null)

// Initialize game on mount
onMounted(() => {
  initializeGame()
  loadGameStats()
})

const initializeGame = () => {
  gameLoading.value = true
  gameError.value = null
  
  // Game will be initialized by GameCanvas component
}

const onGameReady = () => {
  gameLoading.value = false
  console.log('🎮 Game is ready to play!')
}

const onGameError = (error: string) => {
  gameLoading.value = false
  gameError.value = error
  gameErrorDialog.value = true
  
  $q.notify({
    type: 'negative',
    message: 'Game failed to load',
    caption: error,
    position: 'top'
  })
}

const onScoreChanged = (score: number) => {
  currentScore.value = score
  
  // Update high score if needed
  if (score > highScore.value) {
    highScore.value = score
    saveGameStats()
    
    $q.notify({
      type: 'positive',
      message: 'New High Score! 🏆',
      position: 'top'
    })
  }
}

const startGame = () => {
  showMenu.value = false
  gameCanvas.value?.resumeGame()
  
  $q.notify({
    type: 'info',
    message: '🐾 Let your pet play! Tap the screen to interact with critters.',
    position: 'top',
    timeout: 3000
  })
}

const retryInit = () => {
  gameErrorDialog.value = false
  initializeGame()
}

// Simple test sound playback using a real .mp3/.ogg asset path
const playTestSound = async () => {
  const base = import.meta.env.BASE_URL
  const candidates = [
    `${base}assets/audio/positive/yipee.mp3`,
    `${base}assets/audio/positive/yipee.ogg`,
  ]
  // Pick the first that responds OK to HEAD; otherwise use first
  let url = candidates[0]!
  for (const cand of candidates) {
    try {
      const res = await fetch(cand, { method: 'HEAD' })
      if (res.ok) { url = cand; break }
    } catch { /* ignore */ }
  }
  try {
    const audio = new Audio(url)
    audio.preload = 'auto'
    await audio.play()
    $q.notify({ type: 'positive', message: `🔊 Playing: ${url}`, position: 'top' })
  } catch (err) {
    console.error('Audio play failed', err)
    $q.notify({ type: 'negative', message: '❌ Failed to play sound', caption: String(err), position: 'top' })
  }
}

const startTrainingMode = () => {
  showMenu.value = false
  // Future: start vocabulary training mode
  
  $q.notify({
    type: 'info',
    message: '📚 Training mode coming soon!',
    position: 'top'
  })
}

const onCritterSelected = (critter: { id: string; name: string; species: string }) => {
  showCritterSelection.value = false
  
  $q.notify({
    type: 'positive',
    message: `${critter.name} selected! 🎉`,
    position: 'top'
  })
  
  // Communicate selected critter to WASM game engine
  const gameCanvasComponent = gameCanvas.value
  if (gameCanvasComponent?.loadCritterById) {
    console.log(`🐶 Loading critter in game engine by id: ${critter.id}`)
    gameCanvasComponent.loadCritterById(critter.id)
  } else {
    console.warn('⚠️ Game engine not ready for critter loading')
  }
}

const onSettingsChanged = (settings: unknown) => {
  showSettings.value = false
  void settings
  
  $q.notify({
    type: 'positive',
    message: 'Settings saved! ⚙️',
    position: 'top'
  })
  
  // Future: apply settings to game engine
}

// Local storage for game stats
const loadGameStats = () => {
  try {
    const saved = localStorage.getItem('app4dog-game-stats')
    if (saved) {
      const stats = JSON.parse(saved)
      highScore.value = stats.highScore || 0
      currentLevel.value = stats.currentLevel || 1
    }
  } catch (error) {
    console.warn('Failed to load game stats:', error)
  }
}

const saveGameStats = () => {
  try {
    const stats = {
      highScore: highScore.value,
      currentLevel: currentLevel.value,
      lastPlayed: new Date().toISOString()
    }
    localStorage.setItem('app4dog-game-stats', JSON.stringify(stats))
  } catch (error) {
    console.warn('Failed to save game stats:', error)
  }
}

// Handle device back button (for mobile)
onMounted(() => {
  document.addEventListener('backbutton', () => {
    if (!showMenu.value) {
      gameCanvas.value?.pauseGame()
      showMenu.value = true
    }
  })
})
</script>

<style scoped lang="scss">
.game-page {
  padding: 0;
  height: 100vh;
  overflow: hidden;
  background: url('/assets/logo/main-menu-splash-v1.png') center center / cover no-repeat fixed;
}

.game-loading,
.game-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  text-align: center;
  padding: 2rem;
}

.loading-text {
  margin-top: 1rem;
  font-size: 1.2rem;
  color: $primary;
}

.game-menu {
  min-width: 320px;
  max-width: 400px;
}

.menu-stats {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1rem;
}

.menu-buttons {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.full-height {
  height: 100vh;
}
</style>
