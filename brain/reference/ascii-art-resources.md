# ASCII Art Resources

## ASCII Art Generators

### Online Tools

1. **Text to ASCII Art Generator (TAAG)**
   - URL: https://patorjk.com/software/taag/
   - Best for: Headers, titles, large text
   - Features: Many fonts, custom sizing

2. **ASCII Art Generator**
   - URL: https://www.asciiart.eu/
   - Best for: Pre-made art gallery
   - Features: Animals, objects, scenes

3. **Convert Image to ASCII**
   - URL: https://manytools.org/hacker-tools/convert-images-to-ascii-art/
   - Best for: Converting pet images
   - Features: Adjustable resolution, contrast

4. **ASCII Flow**
   - URL: http://asciiflow.com/
   - Best for: Diagrams, boxes, UI elements
   - Features: Draw shapes, arrows

### Command Line Tools

```bash
# figlet - Large text banners
figlet -f slant "MyPet"

# toilet - Alternative to figlet
toilet -f mono12 "TUI"

# jp2a - Convert JPEG to ASCII
jp2a --width=40 --height=20 pet.jpg

# img2txt - From libcaca (color support)
img2txt -W 40 -H 20 pet.png

# aview (for Linux) - View images as ASCII
asciiview pet.jpg
```

## ASCII Art Style Guide for MyPet

### Pet Size by Life Stage

```
Egg:     5x4 chars max
Baby:    8x6 chars max
Child:   12x10 chars max
Teen:    16x14 chars max
Adult:   20x18 chars max
```

### Character Selection

```rust
pub const PET_CHARS: &[(char, &str)] = &[
    ('○', "Head/outline"),
    ('●', "Filled body"),
    ('◕', "Eyes open"),
    ('◔', "Eyes half-closed"),
    ('•', "Small features"),
    ('(', "Left curve"),
    (')', "Right curve"),
    ('/', "Diagonal"),
    ('\\', "Diagonal other"),
    ('|', "Vertical"),
    ('-', "Horizontal"),
    ('_', "Base/ground"),
    ('^', "Ears/horns"),
    ('~', "Wavy/tail"),
    ('*', "Sparkle"),
    ('♥', "Heart"),
    ('Z', "Sleep zzz (large)"),
    ('z', "Sleep zzz (small)"),
    ('@', "Special feature"),
    ('#', "Texture/pattern"),
];

pub const PARTICLE_CHARS: &[(char, &str)] = &[
    ('♥', "Love/health"),
    ('♡', "Love outline"),
    ('★', "Star filled"),
    ('☆', "Star outline"),
    ('✦', "Sparkle small"),
    ('✧', "Sparkle outline"),
    ('✨', "Magic sparkle"),
    ('⚡', "Energy"),
    ('💧', "Water/drop"),
    ('🔥', "Fire/anger"),
    ('💤', "Sleep"),
    ('🎵', "Music/happy"),
    ('🍖', "Food meat"),
    ('🐟', "Food fish"),
    ('🍎', "Food fruit"),
];
```

## Pet Templates

### Egg Stage

```
   _~^~^~_
)\\_\   /_/(   
  / _x_ \
  \_____/
```

### Baby Stage (Cat)

```
   ^~^  /
  (O O) 
  ( > )
--m-m---
```

### Child Stage

```
    /\_/\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
```

### Teen Stage

```
      /\_____/\
     /  o   o  \
    ( ==  ^  == )
     )         (
    (           )
   ( (  )   (  ) )
  (__(__)___(__)__)
```

### Adult Stage

```
       /\     /\
      /  \   /  \
     (    \ /    )
     |  o   o   |
     |   ___)   |
     \  (____   /
      (_____   /
      /         \
     /   /\_/\   \
    /   /     \   \
   (   (       )   )
   /\  /\     /\  /\
  /  \/  \   /  \/  \
```

## Animation Frames Example

### Breathing (3 frames)

Frame 1 (Inhale):
```
   /\_/\
  ( o.o )
   > ^ <
```

Frame 2 (Hold):
```
   /\_/\
  ( o.o )
   >   <
```

Frame 3 (Exhale):
```
   /\_/\
  ( o.o )
   > . <
```

### Eating Animation (5 frames)

Frame 1:
```
   /\_/\
  ( o.o )  🍖
   > ^ <
```

Frame 2:
```
   /\_/\
  ( o.o )🍖
   > ^ <
```

Frame 3:
```
   /\_/\
  ( Oo  )
   > ^ <
```

Frame 4:
```
   /\_/\
  (  -  )
   > ^ <
```

Frame 5:
```
   /\_/\
  ( o.o ) ♥
   > ^ <
```

## Color Palettes for TUI

### Standard Ratatui Colors

```rust
use ratatui::style::Color;

// Basic colors
Color::Black
Color::Red
Color::Green
Color::Yellow
Color::Blue
Color::Magenta
Color::Cyan
Color::Gray
Color::DarkGray
Color::LightRed
Color::LightGreen
Color::LightYellow
Color::LightBlue
Color::LightMagenta
Color::LightCyan
Color::White

// RGB (0-255)
Color::Rgb(u8, u8, u8)

// Indexed (0-255)
Color::Indexed(u8)
```

### Recommended Palettes

**Warm/Friendly**:
```rust
const PALETTE_WARM: &[Color] = &[
    Color::Rgb(255, 200, 150),  // Peach
    Color::Rgb(255, 180, 120),  // Orange
    Color::Rgb(200, 150, 100),  // Brown
    Color::Rgb(255, 220, 180),  // Cream
];
```

**Cool/Calm**:
```rust
const PALETTE_COOL: &[Color] = &[
    Color::Rgb(150, 200, 255),  // Light blue
    Color::Rgb(100, 150, 200),  // Steel blue
    Color::Rgb(180, 220, 255),  // Sky
    Color::Rgb(200, 230, 255),  // Ice
];
```

**High Contrast (Accessibility)**:
```rust
const PALETTE_ACCESSIBLE: &[Color] = &[
    Color::White,
    Color::Yellow,
    Color::Cyan,
    Color::Green,
];
```

## Box Drawing Characters

```
Light:  ┌─┬─┐  Heavy:  ┏━┳━┓  Double:  ╔═╦═╗
        ├─┼─┤          ┣━╋━┫          ╠═╬═╣
        └─┴─┘          ┗━┻━┛          ╚═╩═╝

Rounded: ╭─┬─╮  Block:  ▗▄▖  ▐█▌  ▝▀▘
         ├─┼─┤
         ╰─┴─╯
```

## File Structure for Assets

```
assets/
├── pets/
│   ├── egg.txt
│   ├── baby.txt
│   ├── child.txt
│   ├── teen.txt
│   └── adult.txt
├── animations/
│   ├── idle/
│   │   ├── neutral_01.txt
│   │   ├── neutral_02.txt
│   │   ├── happy_01.txt
│   │   └── sad_01.txt
│   ├── eat/
│   │   ├── frame_01.txt
│   │   ├── frame_02.txt
│   │   └── frame_03.txt
│   └── sleep/
│       └── ...
└── effects/
    ├── particles.json
    └── colors.json
```

## Loading ASCII Art in Rust

```rust
use std::fs;
use std::path::Path;

pub fn load_ascii_art(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

pub fn load_animation_frames(dir: &Path) -> Result<Vec<Vec<String>>> {
    let mut frames = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |e| e == "txt") {
            frames.push(load_ascii_art(&path)?);
        }
    }
    
    // Sort by filename to ensure correct order
    frames.sort_by(|a, b| {
        // Extract frame number and sort
        a.first().cmp(&b.first())
    });
    
    Ok(frames)
}
```

## Testing ASCII Art Rendering

```rust
#[test]
fn test_pet_art_bounds() {
    let art = load_ascii_art(Path::new("assets/pets/baby.txt")).unwrap();
    
    // Check size constraints
    assert!(art.len() <= 6, "Baby pet too tall");
    
    for line in &art {
        assert!(line.len() <= 8, "Baby pet too wide: {}", line);
    }
}
```
