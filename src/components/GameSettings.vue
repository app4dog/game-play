<template>
  <q-card class="game-settings">
    <q-card-section class="text-center">
      <div class="text-h5">⚙️ Game Settings</div>
      <div class="text-subtitle2">Configure your pet's gaming experience</div>
    </q-card-section>
    
    <q-card-section>
      <div class="settings-form">
        <!-- Audio Settings -->
        <div class="setting-group">
          <h6 class="setting-title">🔊 Audio</h6>
          
          <q-item tag="label" v-ripple>
            <q-item-section>
              <q-item-label>Sound Effects</q-item-label>
              <q-item-label caption>Play critter sounds and game audio</q-item-label>
            </q-item-section>
            <q-item-section avatar>
              <q-toggle v-model="settings.audioEnabled" />
            </q-item-section>
          </q-item>
          
          <q-item v-if="settings.audioEnabled">
            <q-item-section>
              <q-item-label>Volume</q-item-label>
              <q-slider
                v-model="settings.volume"
                :min="0"
                :max="100"
                :step="10"
                label
                markers
                class="volume-slider"
              />
            </q-item-section>
          </q-item>
        </div>
        
        <!-- Interaction Settings -->
        <div class="setting-group">
          <h6 class="setting-title">🐾 Interaction</h6>
          
          <q-item tag="label" v-ripple>
            <q-item-section>
              <q-item-label>Vibration Feedback</q-item-label>
              <q-item-label caption>Vibrate device on interactions (mobile only)</q-item-label>
            </q-item-section>
            <q-item-section avatar>
              <q-toggle v-model="settings.vibrationEnabled" />
            </q-item-section>
          </q-item>
          
          <q-item>
            <q-item-section>
              <q-item-label>Touch Sensitivity</q-item-label>
              <q-item-label caption>How responsive the game is to pet interactions</q-item-label>
              <q-select
                v-model="settings.touchSensitivity"
                :options="sensitivityOptions"
                emit-value
                map-options
                class="touch-sensitivity-select"
              />
            </q-item-section>
          </q-item>
        </div>
        
        <!-- Game Difficulty -->
        <div class="setting-group">
          <h6 class="setting-title">🎯 Difficulty</h6>
          
          <q-item>
            <q-item-section>
              <q-item-label>Game Difficulty</q-item-label>
              <q-item-label caption>Adjust challenge level for your pet</q-item-label>
              <q-select
                v-model="settings.difficulty"
                :options="difficultyOptions"
                emit-value
                map-options
                class="difficulty-select"
              />
            </q-item-section>
          </q-item>
          
          <q-item tag="label" v-ripple>
            <q-item-section>
              <q-item-label>Training Mode</q-item-label>
              <q-item-label caption>Enable vocabulary and behavior training</q-item-label>
            </q-item-section>
            <q-item-section avatar>
              <q-toggle v-model="settings.trainingModeEnabled" />
            </q-item-section>
          </q-item>
        </div>
        
        <!-- Display Settings -->
        <div class="setting-group">
          <h6 class="setting-title">🖥️ Display</h6>
          
          <q-item tag="label" v-ripple>
            <q-item-section>
              <q-item-label>Keep Screen On</q-item-label>
              <q-item-label caption>Prevent screen from dimming during play</q-item-label>
            </q-item-section>
            <q-item-section avatar>
              <q-toggle v-model="settings.keepScreenOn" />
            </q-item-section>
          </q-item>
          
          <q-item tag="label" v-ripple>
            <q-item-section>
              <q-item-label>Show Performance Stats</q-item-label>
              <q-item-label caption>Display FPS and performance info (debug)</q-item-label>
            </q-item-section>
            <q-item-section avatar>
              <q-toggle v-model="settings.showPerformanceStats" />
            </q-item-section>
          </q-item>
        </div>
        
        <!-- Pet Profile -->
        <div class="setting-group">
          <h6 class="setting-title">🐕 Pet Profile</h6>
          
          <q-item>
            <q-item-section>
              <q-item-label>Pet Name</q-item-label>
              <q-input
                v-model="settings.petName"
                placeholder="Enter your pet's name"
                dense
                class="pet-name-input"
              />
            </q-item-section>
          </q-item>
          
          <q-item>
            <q-item-section>
              <q-item-label>Pet Age (months)</q-item-label>
              <q-input
                v-model.number="settings.petAgeMonths"
                type="number"
                :min="1"
                :max="300"
                dense
                class="pet-age-input"
              />
            </q-item-section>
          </q-item>
          
          <q-item>
            <q-item-section>
              <q-item-label>Breed</q-item-label>
              <q-input
                v-model="settings.petBreed"
                placeholder="e.g., Golden Retriever"
                dense
                class="pet-breed-input"
              />
            </q-item-section>
          </q-item>
        </div>
      </div>
    </q-card-section>
    
    <q-card-actions align="between">
      <q-btn flat label="Reset to Defaults" @click="resetSettings" />
      <div class="action-buttons">
        <q-btn flat label="Cancel" @click="$emit('close')" />
        <q-btn color="primary" label="Save" @click="saveSettings" />
      </div>
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useQuasar } from 'quasar'

interface GameSettings {
  // Audio
  audioEnabled: boolean
  volume: number
  
  // Interaction
  vibrationEnabled: boolean
  touchSensitivity: string
  
  // Game
  difficulty: string
  trainingModeEnabled: boolean
  
  // Display
  keepScreenOn: boolean
  showPerformanceStats: boolean
  
  // Pet Profile
  petName: string
  petAgeMonths: number
  petBreed: string
}

const $q = useQuasar()

// Emit events
const emit = defineEmits<{
  settingsChanged: [settings: GameSettings]
  close: []
}>()

// Settings state
const settings = ref<GameSettings>({
  audioEnabled: true,
  volume: 70,
  vibrationEnabled: true,
  touchSensitivity: 'normal',
  difficulty: 'easy',
  trainingModeEnabled: true,
  keepScreenOn: true,
  showPerformanceStats: false,
  petName: '',
  petAgeMonths: 12,
  petBreed: ''
})

// Option arrays
const sensitivityOptions = [
  { label: 'Low', value: 'low' },
  { label: 'Normal', value: 'normal' },
  { label: 'High', value: 'high' },
  { label: 'Very High', value: 'very-high' }
]

const difficultyOptions = [
  { label: 'Easy - Puppy Friendly', value: 'easy' },
  { label: 'Normal - Balanced', value: 'normal' },
  { label: 'Hard - Challenging', value: 'hard' },
  { label: 'Expert - Advanced Dogs', value: 'expert' }
]

// Load settings on mount
onMounted(() => {
  loadSettings()
})

// Load settings from storage
const loadSettings = () => {
  try {
    const saved = localStorage.getItem('app4dog-game-settings')
    if (saved) {
      const savedSettings = JSON.parse(saved)
      settings.value = { ...settings.value, ...savedSettings }
    }
  } catch (error) {
    console.warn('Failed to load settings:', error)
  }
}

// Save settings to storage and emit
const saveSettings = () => {
  try {
    localStorage.setItem('app4dog-game-settings', JSON.stringify(settings.value))
    emit('settingsChanged', settings.value)
    
    $q.notify({
      type: 'positive',
      message: 'Settings saved successfully! 📱',
      position: 'top'
    })
    
    // Apply screen wake lock if enabled
    if (settings.value.keepScreenOn && 'wakeLock' in navigator) {
      navigator.wakeLock.request('screen').catch(err => {
        console.warn('Failed to acquire wake lock:', err)
      })
    }
    
  } catch (error) {
    console.error('Failed to save settings:', error)
    $q.notify({
      type: 'negative',
      message: 'Failed to save settings',
      position: 'top'
    })
  }
}

// Reset to default settings
const resetSettings = () => {
  $q.dialog({
    title: 'Reset Settings',
    message: 'Are you sure you want to reset all settings to defaults?',
    cancel: true,
    persistent: true
  }).onOk(() => {
    settings.value = {
      audioEnabled: true,
      volume: 70,
      vibrationEnabled: true,
      touchSensitivity: 'normal',
      difficulty: 'easy',
      trainingModeEnabled: true,
      keepScreenOn: true,
      showPerformanceStats: false,
      petName: '',
      petAgeMonths: 12,
      petBreed: ''
    }
    
    $q.notify({
      type: 'info',
      message: 'Settings reset to defaults',
      position: 'top'
    })
  })
}
</script>

<style scoped lang="scss">
.game-settings {
  min-width: 400px;
  max-width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.setting-group {
  border: 1px solid $grey-4;
  border-radius: 8px;
  padding: 1rem;
  background: $grey-1;
}

.setting-title {
  margin: 0 0 1rem 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: $primary;
}

.volume-slider {
  margin-top: 0.5rem;
}

.touch-sensitivity-select,
.difficulty-select {
  margin-top: 0.5rem;
}

.pet-name-input,
.pet-age-input,
.pet-breed-input {
  margin-top: 0.5rem;
}

.action-buttons {
  display: flex;
  gap: 0.5rem;
}
</style>