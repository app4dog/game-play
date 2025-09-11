# Bevy Platform Integration Pattern (b00t)

> **Idiomatic, ergonomic pattern for complex platform integrations in Bevy games**
> 
> Handles: Bluetooth, AI workflows, cloud APIs, crypto systems, dynamic assets
> 
> **Principles**: DRY, testable, maintainable, loosely coupled

## 🎯 Core Architecture Pattern

### Resource-Based System Design

Each platform integration becomes a **Bevy Resource + Systems + Plugin**:

```rust
// 1. Resource = State Container
#[derive(Resource)]
pub struct BluetoothManager {
    scanning: bool,
    connected_devices: Vec<DeviceInfo>,
    pending_requests: HashMap<String, BluetoothRequest>,
}

// 2. Events = Communication Protocol  
#[derive(Event)]
pub enum BluetoothRequest {
    Scan { device_filter: String },
    Connect { device_id: String },
    Disconnect { device_id: String },
}

#[derive(Event)]
pub enum BluetoothResponse {
    ScanCompleted { devices: Vec<DeviceInfo> },
    Connected { device_id: String },
    Error { message: String },
}

// 3. Systems = Business Logic
fn handle_bluetooth_requests(
    mut bt: ResMut<BluetoothManager>,
    mut requests: EventReader<BluetoothRequest>,
    mut responses: EventWriter<BluetoothResponse>,
) {
    for request in requests.read() {
        match request {
            BluetoothRequest::Scan { device_filter } => {
                bt.start_scan(device_filter);
            }
            // Handle other requests...
        }
    }
}

// 4. Plugin = Self-Contained Integration
pub struct BluetoothPlugin;
impl Plugin for BluetoothPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BluetoothManager>()
            .add_event::<BluetoothRequest>()
            .add_event::<BluetoothResponse>()
            .add_systems(Update, (
                handle_bluetooth_requests,
                process_bluetooth_responses,
                bluetooth_connection_monitor,
            ));
    }
}
```

## 🔗 TypeScript Bridge Integration

### Single Event Bus Pattern

```typescript
// Unified event interface for all platform integrations
export interface PlatformEvent {
  // Bluetooth Low Energy
  Bluetooth: {
    action: 'scan' | 'connect' | 'disconnect'
    device_id?: string
    device_filter?: string
  }
  
  // Cloud API integration
  CloudApi: {
    action: 'fetch_profile' | 'save_progress' | 'sync_data'
    endpoint?: string
    data?: any
  }
  
  // Cryptocurrency/Points system
  Crypto: {
    action: 'transfer_points' | 'check_balance' | 'mint_reward'
    amount?: number
    to?: string
    transaction_id?: string
  }
  
  // AI/ML workflows
  AI: {
    action: 'generate_asset' | 'process_image' | 'run_inference'
    prompt?: string
    asset_type?: string
    model?: string
    input_data?: any
  }
  
  // Dynamic asset downloading
  AssetDownload: {
    action: 'download' | 'cache' | 'verify'
    asset_url: string
    asset_type: string
    priority?: 'high' | 'normal' | 'low'
  }
}
```

### TypeScript Event Handler

```typescript
class PlatformEventBridge {
  private async handlePlatformEvent(event: PlatformEvent) {
    const eventType = Object.keys(event)[0] as keyof PlatformEvent
    const eventData = event[eventType]
    
    switch (eventType) {
      case 'Bluetooth':
        await this.handleBluetoothEvent(eventData)
        break
      case 'CloudApi':
        await this.handleCloudApiEvent(eventData)
        break
      case 'Crypto':
        await this.handleCryptoEvent(eventData)
        break
      case 'AI':
        await this.handleAIEvent(eventData)
        break
      case 'AssetDownload':
        await this.handleAssetDownloadEvent(eventData)
        break
    }
  }
  
  private async handleBluetoothEvent(data: PlatformEvent['Bluetooth']) {
    switch (data.action) {
      case 'scan':
        return await this.bluetoothService.scan(data.device_filter)
      case 'connect':
        return await this.bluetoothService.connect(data.device_id!)
      // ... other actions
    }
  }
}
```

### **🚨 CRITICAL: Event Handler Error Boundary Requirements**

**ALL TypeScript event handlers MUST implement error boundaries. Fire-and-forget patterns without error handling are FORBIDDEN.**

```typescript
class PlatformEventBridge {
  private boundEventHandler?: (event: Event) => void
  
  init() {
    // ✅ REQUIRED: Error boundary wrapper for async handlers
    this.boundEventHandler = (event: Event) => {
      this.handlePlatformEvent(event).catch(error => {
        // 🔥 MANDATORY: Generic error/crash logger
        console.error(`🔥 b00t platform integration error [${this.getEventType(event)}]:`, error)
        
        // 📊 REQUIRED: Error metrics for Resource state management  
        this.incrementErrorCount(`${this.getEventType(event)}_failed`)
        
        // 📤 OPTIONAL: Emit standardized error event back to Bevy
        this.emitErrorToBevy('PlatformEventFailed', error, event)
      })
    }
    
    window.addEventListener('bevy-platform-event', this.boundEventHandler)
  }
  
  destroy() {
    if (this.boundEventHandler) {
      window.removeEventListener('bevy-platform-event', this.boundEventHandler)
    }
  }
}

// ❌ FORBIDDEN: Silent failure patterns
window.addEventListener('event', async (event) => {
  void this.handleEvent(event)  // NO ERROR HANDLING - VIOLATES b00t PATTERN
})

// ❌ FORBIDDEN: Bare async handler without error boundary  
window.addEventListener('event', this.handleEvent.bind(this))  // PROMISE LEAK
```

### **Error Boundary Implementation Requirements**

1. **Generic Error/Crash Logger**: ALL errors MUST be logged with context
2. **Error Metrics**: Track error counts for Resource state management
3. **Structured Logging**: Include event type, error details, and context
4. **Optional Error Events**: Can emit errors back to Bevy for handling
5. **No Silent Failures**: Every async operation MUST have error handling

```typescript
// ✅ COMPLIANT: Full error boundary implementation
private createErrorBoundary<T>(
  handler: (event: Event) => Promise<T>, 
  context: string
): (event: Event) => void {
  return (event: Event) => {
    handler(event).catch(error => {
      // MANDATORY: Generic error/crash logging
      console.error(`🔥 b00t platform integration error [${context}]:`, {
        error: error.message,
        stack: error.stack,
        eventType: event.type,
        timestamp: new Date().toISOString(),
        context
      })
      
      // REQUIRED: Update error metrics
      this.incrementErrorCount(`${context}_failed`)
      
      // OPTIONAL: Error telemetry
      this.reportErrorTelemetry(context, error)
    })
  }
}
```

## 🧪 Testing Strategy

### Resource-Based Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bluetooth_scan_request() {
        let mut app = App::new();
        
        // 1. Add plugin with mock resource
        app.add_plugins(BluetoothPlugin);
        app.insert_resource(BluetoothManager::mock());
        
        // 2. Send request event
        app.world_mut().send_event(BluetoothRequest::Scan {
            device_filter: "dog_collar".to_string()
        });
        
        // 3. Process one frame
        app.update();
        
        // 4. Verify state change
        let bt = app.world().resource::<BluetoothManager>();
        assert_eq!(bt.scanning, true);
        
        // 5. Verify response event
        let responses: Vec<_> = app.world()
            .resource::<Events<BluetoothResponse>>()
            .get_reader().read().collect();
        assert!(!responses.is_empty());
    }
    
    #[test]
    fn test_error_handling() {
        let mut app = App::new();
        app.add_plugins(BluetoothPlugin);
        
        // Inject failing resource
        app.insert_resource(BluetoothManager::with_failure_mode());
        
        app.world_mut().send_event(BluetoothRequest::Scan {
            device_filter: "invalid".to_string()
        });
        
        app.update();
        
        // Verify error response
        let bt = app.world().resource::<BluetoothManager>();
        assert!(bt.last_error.is_some());
    }
}
```

### TypeScript Testing

```typescript
describe('PlatformEventBridge', () => {
  it('handles bluetooth scan events', async () => {
    const mockBluetoothService = {
      scan: jest.fn().mockResolvedValue(['device1', 'device2'])
    }
    
    const bridge = new PlatformEventBridge(mockBluetoothService)
    
    await bridge.handlePlatformEvent({
      Bluetooth: { action: 'scan', device_filter: 'dog_collar' }
    })
    
    expect(mockBluetoothService.scan).toHaveBeenCalledWith('dog_collar')
  })
})
```

## 🏗️ Scalability Patterns

### Plugin Composition

```rust
// Game just composes plugins for features it needs
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        
        // Platform integrations
        .add_plugins(BluetoothPlugin)
        .add_plugins(CloudApiPlugin)
        .add_plugins(CryptoPointsPlugin)
        .add_plugins(AIWorkflowPlugin)
        .add_plugins(DynamicAssetPlugin)
        
        .run();
}
```

### Feature Flags

```toml
# Cargo.toml
[features]
default = ["audio"]
bluetooth = ["dep:btleplug"]
cloud-api = ["dep:reqwest", "dep:serde_json"]
crypto = ["dep:web3", "dep:ethers"]
ai-workflows = ["dep:candle", "dep:tokenizers"]
dynamic-assets = ["dep:futures-util"]

[dependencies]
btleplug = { version = "0.11", optional = true }
reqwest = { version = "0.11", optional = true, features = ["json"] }
# ... other optional deps
```

```rust
// Conditional compilation
#[cfg(feature = "bluetooth")]
pub use bluetooth::BluetoothPlugin;

#[cfg(feature = "cloud-api")]  
pub use cloud_api::CloudApiPlugin;
```

## 🔄 Error Handling Pattern

### Resource-Level Error Management

```rust
#[derive(Resource)]
pub struct BluetoothManager {
    state: BluetoothState,
    last_error: Option<BluetoothError>,
    error_count: u32,
    retry_backoff: Duration,
}

impl BluetoothManager {
    pub fn handle_error(&mut self, error: BluetoothError) {
        self.last_error = Some(error.clone());
        self.error_count += 1;
        
        // Exponential backoff
        self.retry_backoff = Duration::from_millis(
            100 * 2_u64.pow(self.error_count.min(10))
        );
        
        // Emit error event for game to handle
        // (Done through event system, not direct coupling)
    }
    
    pub fn should_retry(&self) -> bool {
        self.error_count < 3 && 
        self.retry_backoff.as_millis() < 30000 // Max 30s backoff
    }
}
```

## 🎮 Game Integration Examples

### Dynamic Asset Loading

```rust
#[derive(Resource)]
pub struct DynamicAssetManager {
    downloading: HashMap<String, AssetDownload>,
    cache: HashMap<String, AssetHandle>,
    download_queue: VecDeque<DownloadRequest>,
}

fn dynamic_asset_system(
    mut assets: ResMut<DynamicAssetManager>,
    mut requests: EventReader<AssetDownloadRequest>,
    asset_server: Res<AssetServer>,
) {
    for request in requests.read() {
        // Prioritize based on game state
        let priority = match request.asset_type {
            AssetType::CritterSprite => Priority::High,
            AssetType::BackgroundMusic => Priority::Low,
            _ => Priority::Normal,
        };
        
        assets.queue_download(request.clone(), priority);
    }
}
```

### AI Workflow Integration

```rust
#[derive(Resource)]
pub struct AIWorkflowManager {
    running_tasks: HashMap<String, AITask>,
    completed_results: HashMap<String, AIResult>,
    model_cache: HashMap<String, ModelHandle>,
}

fn ai_workflow_system(
    mut ai: ResMut<AIWorkflowManager>,
    mut requests: EventReader<AIRequest>,
    mut responses: EventWriter<AIResponse>,
) {
    for request in requests.read() {
        match request {
            AIRequest::GenerateSprite { prompt, style } => {
                let task_id = ai.start_generation_task(prompt, style);
                // Async task runs in background, system polls for completion
            }
            AIRequest::ProcessBehavior { critter_id, context } => {
                // Real-time AI for critter behavior
                let behavior = ai.infer_behavior(critter_id, context);
                responses.send(AIResponse::BehaviorUpdated { critter_id, behavior });
            }
        }
    }
}
```

## 🦨 b00t Principles Summary

1. **Single Responsibility**: Each Resource handles one integration domain
2. **Loose Coupling**: Event-driven communication between systems  
3. **Testability**: Resources can be mocked, systems tested in isolation
4. **Composability**: Plugin architecture enables feature combinations
5. **Error Boundaries**: Each Resource manages its own failure modes
6. **Type Safety**: Rust → TypeScript type generation with ts-rs
7. **Maintainability**: Clear separation between game logic and platform code

## 🔮 Future Extensions

This pattern scales to handle:

- **WebRTC** peer-to-peer multiplayer
- **File system** access for user content
- **Camera/microphone** for AR features  
- **Payment processing** for in-app purchases
- **Analytics** and telemetry collection
- **Push notifications** for engagement
- **Social media** integrations
- **Hardware sensors** (accelerometer, GPS)

Each becomes a self-contained Plugin with its own Resource, Events, and Systems.

---

**Pattern Status**: ✅ **Production Ready**  
**Complexity**: 🟢 **Low** (idiomatic Bevy/TypeScript)  
**Maintainability**: 🟢 **High** (clear separation of concerns)  
**Testability**: 🟢 **High** (isolated, mockable resources)  
**Scalability**: 🟢 **High** (plugin composition)