<template>
  <q-page class="game-page">
    <!-- Game loading state -->
    <div v-if="gameLoading" class="game-loading">
      <q-spinner-puff color="primary" size="4em" />
      <p class="loading-text">🐕 Loading App4.Dog Game...</p>
    </div>
    
    <!-- Game error state -->
    <div v-else-if="gameError" class="game-error">
      <q-icon name="error" color="negative" size="4em" />
      <h3>Game Loading Error</h3>
      <p>{{ gameError }}</p>
      <q-btn color="primary" label="Retry" @click="initializeGame" />
    </div>
    
    <!-- Game canvas -->
    <GameCanvas
      v-else
      ref="gameCanvas"
      @game-ready="onGameReady"
      @game-error="onGameError"
      @score-changed="onScoreChanged"
      class="full-height"
    />
    
    <!-- Game menu overlay -->
    <q-dialog v-model="showMenu">
      <q-card class="game-menu">
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
import CritterSelection from '../components/CritterSelection.vue'
import GameSettings from '../components/GameSettings.vue'

const $q = useQuasar()

// Game state
const gameLoading = ref(true)
const gameError = ref<string | null>(null)
const showMenu = ref(true)
const showCritterSelection = ref(false)
const showSettings = ref(false)

// Game stats
const highScore = ref(0)
const currentLevel = ref(1)
const currentScore = ref(0)

// Game canvas reference
const gameCanvas = ref<InstanceType<typeof GameCanvas> | null>(null)

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

const startTrainingMode = () => {
  showMenu.value = false
  // Future: start vocabulary training mode
  
  $q.notify({
    type: 'info',
    message: '📚 Training mode coming soon!',
    position: 'top'
  })
}

const onCritterSelected = (critterName: string) => {
  showCritterSelection.value = false
  
  $q.notify({
    type: 'positive',
    message: `${critterName} selected! 🎉`,
    position: 'top'
  })
  
  // Future: tell game engine to spawn selected critter
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
