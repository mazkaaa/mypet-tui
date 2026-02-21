# ADR 003: Save File Format

## Status
Accepted

## Context

We need to persist game state between sessions. The save system must:
- Store all pet state (stats, age, life stage)
- Handle game closure gracefully (auto-save)
- Support future updates (backward compatibility)
- Be human-readable for debugging
- Have reasonable file size
- Load/save quickly

## Options Considered

### Option 1: JSON (Human-Readable)
**Description**: Standard JSON with serde serialization

**Example**:
```json
{
  "version": 1,
  "saved_at": "2026-02-21T10:30:00Z",
  "pet": {
    "name": "Fluffy",
    "stage": "Child",
    "stats": {
      "hunger": 45,
      "happiness": 80,
      "energy": 60
    }
  }
}
```

**Pros**:
- Human-readable and editable
- Easy debugging (just open in text editor)
- Standard format, well-supported
- Self-describing structure
- Easy backward compatibility (add fields)

**Cons**:
- Larger file size than binary
- Slower to parse than binary
- Users can cheat by editing

### Option 2: Binary (Bincode)
**Description**: Compact binary format using bincode crate

**Example**:
```rust
let encoded = bincode::serialize(&save_data)?;
fs::write("save.dat", encoded)?;
```

**Pros**:
- Smallest file size
- Fastest load/save
- Not easily editable (anti-cheat)
- Type-safe with serde

**Cons**:
- Not human-readable
- Harder to debug corruption issues
- Breaking changes require migration code
- Platform-dependent if not careful

### Option 3: MessagePack
**Description**: Binary JSON-like format

**Pros**:
- Smaller than JSON
- Still structured/schemaless
- Fast parsing
- Language-agnostic

**Cons**:
- Not human-readable (need tool to view)
- Another dependency
- Overkill for simple save files

### Option 4: SQLite
**Description**: Embedded SQL database

**Pros**:
- ACID guarantees
- Supports complex queries
- Can store multiple saves
- Migration support via SQL

**Cons**:
- Overkill for single-pet game
- Larger dependency
- More complex code
- Slower for simple reads/writes

## Decision

We will use **Option 1: JSON** with the following conventions:
- Pretty-printed for human readability
- Version field for migrations
- ISO 8601 timestamps
- Standard XDG directory for location

## Rationale

1. **Debugging**: Can inspect and manually fix saves during development
2. **Transparency**: Users can understand their save data
3. **Simplicity**: No extra dependencies beyond serde_json
4. **Compatibility**: Easy to evolve format with version field
5. **Size**: Save files will be small (< 10KB), JSON overhead acceptable

## Implementation Details

### File Location
```rust
use dirs::data_dir;

pub fn save_path() -> PathBuf {
    let mut path = dirs::data_dir()
        .expect("Could not find data directory");
    path.push("mypet-tui");
    path.push("save.json");
    path
}
```

### Format Versioning
```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub saved_at: DateTime<Utc>,
    pub pet: Pet,
    pub total_play_time: Duration,
}

impl SaveData {
    pub const CURRENT_VERSION: u32 = 1;
    
    pub fn migrate(&mut self) -> Result<()> {
        match self.version {
            1 => Ok(()),
            0 => {
                // Migration from version 0 to 1
                // (example: rename field)
                self.version = 1;
                Ok(())
            }
            _ => Err(SaveError::UnsupportedVersion(self.version)),
        }
    }
}
```

### Save Structure
```rust
{
    "version": 1,
    "saved_at": "2026-02-21T10:30:00Z",
    "pet": {
        "name": "Fluffy",
        "species": "Cat",
        "birth_time": "2026-02-20T08:00:00Z",
        "stage": "Child",
        "state": "Normal",
        "stats": {
            "hunger": {"value": 45},
            "happiness": {"value": 80},
            "energy": {"value": 60},
            "health": {"value": 95},
            "hygiene": {"value": 70}
        },
        "last_interaction": "2026-02-21T10:25:00Z"
    },
    "total_play_time": {"secs": 3600, "nanos": 0},
    "achievements": ["first_feed", "evolve_baby"]
}
```

### Auto-Save Strategy

```rust
pub struct AutoSave {
    last_save: Instant,
    dirty: bool,
    interval: Duration,
}

impl AutoSave {
    pub const INTERVAL: Duration = Duration::from_secs(30);
    
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    pub fn tick(&mut self, app: &App) -> Result<()> {
        if self.dirty && self.last_save.elapsed() >= Self::INTERVAL {
            app.save()?;
            self.saved();
        }
        Ok(())
    }
}
```

### Corruption Handling

```rust
pub fn load_or_create() -> Result<SaveData> {
    let path = save_path();
    
    if !path.exists() {
        return Ok(SaveData::new());
    }
    
    let contents = fs::read_to_string(&path)?;
    
    match serde_json::from_str::<SaveData>(&contents) {
        Ok(mut data) => {
            data.migrate()?;
            Ok(data)
        }
        Err(e) => {
            log::error!("Failed to load save: {}", e);
            // Backup corrupted save
            let backup = path.with_extension("json.corrupted");
            fs::rename(&path, backup)?;
            Ok(SaveData::new())
        }
    }
}
```

## Consequences

### Positive
- Easy to debug and inspect
- Users can manually fix issues
- Simple implementation
- Easy to evolve format

### Negative
- Users can cheat (acceptable for single-player)
- Slightly larger files (insignificant at <10KB)
- Slightly slower than binary (insignificant)

## Security Considerations

This is a single-player game, so save file security is not a priority:
- Users editing saves is acceptable
- No sensitive data stored
- No competitive multiplayer

If security becomes important, we could add:
- HMAC signature to detect tampering
- Optional encryption
- Binary format option

## Future Considerations

- Compress saves with gzip if they grow large
- Add backup system (keep last N saves)
- Export/import functionality
- Cloud save sync (if multiplayer added)

## References

- [Serde JSON](https://docs.rs/serde_json/latest/serde_json/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [dirs crate](https://docs.rs/dirs/latest/dirs/)
