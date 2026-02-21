# Phase 6: Persistence

**Status**: 📋 Not Started  
**Duration**: Week 3-4  
**Goal**: Save/load game state, auto-save, and save file management

## Overview

This phase implements the persistence layer: saving game state to disk, loading on startup, auto-save functionality, and handling save file corruption gracefully.

## Prerequisites

- Phase 5 complete (animations working)
- Understanding of serde serialization
- XDG directories knowledge

## Step 1: Save Data Structure

Create `src/save.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::pet::Pet;

/// Current save format version
pub const SAVE_VERSION: u32 = 1;

/// Serializable save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub saved_at: DateTime<Utc>,
    pub pet: Pet,
    pub total_play_time_secs: u64,
    pub session_count: u32,
}

impl SaveData {
    /// Create save data from current app state
    pub fn from_app(pet: &Pet, total_play_time: std::time::Duration, session_count: u32) -> Self {
        Self {
            version: SAVE_VERSION,
            saved_at: Utc::now(),
            pet: pet.clone(),
            total_play_time_secs: total_play_time.as_secs(),
            session_count,
        }
    }
    
    /// Migrate from older versions
    pub fn migrate(&mut self) -> Result<()> {
        match self.version {
            1 => {
                // Current version, no migration needed
                Ok(())
            }
            0 => {
                // Example migration from version 0
                // In real scenario, handle field renames, etc.
                self.version = 1;
                Ok(())
            }
            _ => {
                Err(AppError::Save(format!(
                    "Unsupported save version: {}",
                    self.version
                )))
            }
        }
    }
    
    /// Validate save data integrity
    pub fn validate(&self) -> Result<()> {
        // Check pet name not empty
        if self.pet.name.is_empty() {
            return Err(AppError::Save("Pet name cannot be empty".to_string()));
        }
        
        // Check stats in valid range (they should be due to StatValue, but verify)
        if self.pet.stats.hunger.value() > 100 {
            return Err(AppError::Save("Invalid hunger value".to_string()));
        }
        
        Ok(())
    }
}

/// Save file manager
pub struct SaveManager {
    save_dir: PathBuf,
    backup_dir: PathBuf,
}

impl SaveManager {
    /// Create save manager with XDG directories
    pub fn new() -> Result<Self> {
        let save_dir = Self::get_save_dir()?;
        let backup_dir = save_dir.join("backups");
        
        // Create directories if needed
        fs::create_dir_all(&save_dir)?;
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            save_dir,
            backup_dir,
        })
    }
    
    /// Get the save directory (XDG compliant)
    fn get_save_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Save("Could not find data directory".to_string()))?;
        
        Ok(data_dir.join("mypet-tui"))
    }
    
    /// Get path to main save file
    pub fn save_file_path(&self) -> PathBuf {
        self.save_dir.join("save.json")
    }
    
    /// Check if save exists
    pub fn save_exists(&self) -> bool {
        self.save_file_path().exists()
    }
    
    /// Save game state
    pub fn save(&self, data: &SaveData) -> Result<()> {
        let save_path = self.save_file_path();
        
        // Create backup of existing save
        if save_path.exists() {
            self.create_backup()?;
        }
        
        // Serialize to JSON with pretty printing (for debugging)
        let json = serde_json::to_string_pretty(data)?;
        
        // Write to temporary file first (atomic write)
        let temp_path = save_path.with_extension("tmp");
        fs::write(&temp_path, json)?;
        
        // Rename to final location (atomic on most filesystems)
        fs::rename(&temp_path, save_path)?;
        
        Ok(())
    }
    
    /// Load game state
    pub fn load(&self) -> Result<SaveData> {
        let save_path = self.save_file_path();
        
        if !save_path.exists() {
            return Err(AppError::Save("Save file not found".to_string()));
        }
        
        let json = fs::read_to_string(&save_path)?;
        
        // Parse JSON
        let mut data: SaveData = match serde_json::from_str(&json) {
            Ok(d) => d,
            Err(e) => {
                // Try to recover from backup
                log::error!("Failed to parse save file: {}", e);
                return self.load_from_backup();
            }
        };
        
        // Migrate if needed
        data.migrate()?;
        
        // Validate
        data.validate()?;
        
        Ok(data)
    }
    
    /// Create a backup of current save
    fn create_backup(&self) -> Result<()> {
        let save_path = self.save_file_path();
        
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("save_{}.json", timestamp);
        let backup_path = self.backup_dir.join(backup_name);
        
        fs::copy(&save_path, backup_path)?;
        
        // Clean old backups (keep last 10)
        self.clean_old_backups(10)?;
        
        Ok(())
    }
    
    /// Clean old backups, keeping only N most recent
    fn clean_old_backups(&self, keep: usize) -> Result<()> {
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension()
                    .map_or(false, |ext| ext == "json")
            })
            .collect();
        
        // Sort by modified time (newest first)
        backups.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        backups.reverse();
        
        // Remove excess backups
        for entry in backups.iter().skip(keep) {
            let _ = fs::remove_file(entry.path());
        }
        
        Ok(())
    }
    
    /// Try to load from most recent backup
    fn load_from_backup(&self) -> Result<SaveData> {
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension()
                    .map_or(false, |ext| ext == "json")
            })
            .collect();
        
        // Sort by modified time (newest first)
        backups.sort_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        backups.reverse();
        
        // Try each backup
        for entry in backups {
            let json = fs::read_to_string(entry.path())?;
            
            if let Ok(mut data) = serde_json::from_str::<SaveData>(&json) {
                log::info!("Recovered from backup: {:?}", entry.path());
                data.migrate()?;
                return Ok(data);
            }
        }
        
        Err(AppError::Save("Could not recover from any backup".to_string()))
    }
    
    /// Delete all save data
    pub fn delete_all(&self) -> Result<()> {
        let save_path = self.save_file_path();
        if save_path.exists() {
            fs::remove_file(save_path)?;
        }
        
        // Optionally keep backups, or clear them too
        // fs::remove_dir_all(&self.backup_dir)?;
        
        Ok(())
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new().expect("Failed to create save manager")
    }
}
```

## Step 2: Auto-Save System

Create `src/auto_save.rs`:

```rust
use std::time::{Duration, Instant};

use crate::save::{SaveData, SaveManager};
use crate::error::Result;

/// Auto-save trigger conditions
#[derive(Debug, Clone)]
pub struct AutoSaveConfig {
    /// Auto-save interval
    pub interval: Duration,
    /// Save on significant stat changes
    pub save_on_stat_change: bool,
    /// Save when pet evolves
    pub save_on_evolution: bool,
    /// Save when app exits
    pub save_on_exit: bool,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(60), // Every minute
            save_on_stat_change: true,
            save_on_evolution: true,
            save_on_exit: true,
        }
    }
}

/// Auto-save manager
#[derive(Debug)]
pub struct AutoSave {
    config: AutoSaveConfig,
    last_save: Instant,
    dirty: bool,
    last_pet_state: Option<Vec<u8>>, // For change detection
}

impl AutoSave {
    pub fn new(config: AutoSaveConfig) -> Self {
        Self {
            config,
            last_save: Instant::now(),
            dirty: false,
            last_pet_state: None,
        }
    }
    
    /// Mark data as dirty (needs saving)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Check if auto-save should trigger
    pub fn should_save(&self) -> bool {
        if !self.dirty {
            return false;
        }
        
        self.last_save.elapsed() >= self.config.interval
    }
    
    /// Update and check if save needed
    pub fn tick(&mut self) -> bool {
        self.should_save()
    }
    
    /// Called after successful save
    pub fn saved(&mut self) {
        self.last_save = Instant::now();
        self.dirty = false;
    }
    
    /// Check if pet stats changed significantly
    pub fn check_stat_changes(&mut self, pet: &crate::pet::Pet) -> bool {
        if !self.config.save_on_stat_change {
            return false;
        }
        
        let current_state = vec![
            pet.stats.hunger.value(),
            pet.stats.happiness.value(),
            pet.stats.energy.value(),
            pet.stats.health.value(),
            pet.stats.hygiene.value(),
        ];
        
        let changed = self.last_pet_state.as_ref()
            .map(|last| last != &current_state)
            .unwrap_or(true);
        
        if changed {
            self.last_pet_state = Some(current_state);
            self.mark_dirty();
        }
        
        changed
    }
    
    /// Trigger save manually
    pub fn force_save(&mut self) {
        self.mark_dirty();
        self.last_save = Instant::now() - self.config.interval - Duration::from_secs(1);
    }
}

impl Default for AutoSave {
    fn default() -> Self {
        Self::new(AutoSaveConfig::default())
    }
}
```

## Step 3: Integration with App

Update `src/app.rs`:

```rust
use crate::save::{SaveData, SaveManager};
use crate::auto_save::AutoSave;

#[derive(Debug)]
pub struct App {
    // ... existing fields ...
    pub save_manager: SaveManager,
    pub auto_save: AutoSave,
    pub total_play_time: std::time::Duration,
    pub session_count: u32,
    pub start_time: std::time::Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let save_manager = SaveManager::new()?;
        
        // Try to load existing save
        let (pet, total_play_time, session_count) = if save_manager.save_exists() {
            match save_manager.load() {
                Ok(data) => {
                    log::info!("Loaded save from {}", data.saved_at);
                    (data.pet, 
                     std::time::Duration::from_secs(data.total_play_time_secs),
                     data.session_count + 1)
                }
                Err(e) => {
                    log::error!("Failed to load save: {}", e);
                    (Pet::new("Fluffy"), std::time::Duration::ZERO, 1)
                }
            }
        } else {
            (Pet::new("Fluffy"), std::time::Duration::ZERO, 1)
        };
        
        Ok(Self {
            running: true,
            tick_count: 0,
            pet,
            art_cache: AsciiArtCache::new(),
            event_log: EventLog::new(50),
            animated_pet: AnimatedPet::new(),
            last_animation_update: std::time::Instant::now(),
            save_manager,
            auto_save: AutoSave::default(),
            total_play_time,
            session_count,
            start_time: std::time::Instant::now(),
        })
    }
    
    pub fn save(&self) -> Result<()> {
        let current_session = self.start_time.elapsed();
        let total = self.total_play_time + current_session;
        
        let data = SaveData::from_app(&self.pet,
            total,
            self.session_count
        );
        
        self.save_manager.save(&data)?;
        log::info!("Game saved successfully");
        
        Ok(())
    }
    
    pub fn check_auto_save(&mut self) {
        // Check stat changes
        self.auto_save.check_stat_changes(&self.pet);
        
        // Check if time to save
        if self.auto_save.tick() {
            if let Err(e) = self.save() {
                log::error!("Auto-save failed: {}", e);
                self.event_log.warning("Auto-save failed!");
            } else {
                self.event_log.info("Game auto-saved");
                self.auto_save.saved();
            }
        }
    }
    
    pub fn quit(&mut self) {
        // Save on exit if configured
        if self.auto_save.config.save_on_exit {
            if let Err(e) = self.save() {
                log::error!("Failed to save on exit: {}", e);
            }
        }
        
        self.running = false;
    }
}
```

## Step 4: Handle Game Loop Integration

Update `src/game_loop.rs`:

```rust
async fn tick(&self, app: &mut App) {
    // ... existing tick logic ...
    
    // Check auto-save
    app.check_auto_save();
}
```

## Step 5: Save File Format

Example save file (`~/.local/share/mypet-tui/save.json`):

```json
{
  "version": 1,
  "saved_at": "2026-02-21T10:30:00Z",
  "pet": {
    "name": "Fluffy",
    "stage": "Child",
    "state": "Normal",
    "stats": {
      "hunger": {"value": 45},
      "happiness": {"value": 80},
      "energy": {"value": 60},
      "health": {"value": 95},
      "hygiene": {"value": 70}
    },
    "birth_time": "2026-02-20T08:00:00Z",
    "last_interaction": "2026-02-21T10:25:00Z",
    "age_seconds": 86400
  },
  "total_play_time_secs": 3600,
  "session_count": 5
}
```

## Step 6: Error Handling

Update `src/error.rs`:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    // ... existing errors ...
    
    #[error("Save error: {0}")]
    Save(String),
}
```

## Phase 6 Success Criteria

- [x] Save file created in correct XDG directory
- [x] All pet state persists between sessions
- [x] Auto-save triggers on timer
- [x] Auto-save triggers on significant changes
- [x] Backup system keeps last 10 saves
- [x] Corrupted saves recover from backup
- [x] Save on exit works correctly
- [x] Save format is human-readable JSON
- [x] Version field allows future migrations

## Testing Save/Load

```bash
# Run game, play for a bit, press 'q' to quit
cargo run

# Check save file exists
cat ~/.local/share/mypet-tui/save.json

# Run again - should load previous state
cargo run

# Check backups
ls ~/.local/share/mypet-tui/backups/
```

## Next Steps

Move to **Phase 7: Extras** for mini-games, achievements, and polish.
