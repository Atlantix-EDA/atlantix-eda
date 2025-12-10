# KiParse Examples

This directory contains examples demonstrating how to use KiParse for various KiCad file parsing tasks.

## Quick Start

All examples now work immediately without requiring any arguments!

```bash
# Basic examples - no arguments needed!
cargo run --example basic          # Basic PCB analysis
cargo run --example get_layers     # Extract layer information
cargo run --example get_positions  # Extract component positions
cargo run --example get_symbols    # Parse symbol libraries
cargo run --example get_details    # Detailed PCB information
cargo run --example get_3d_models  # Analyze 3D model coverage
```

## Available Examples

### 📋 Basic

**File**: `basic.rs`  
**Purpose**: Basic PCB analysis showing layers and file statistics  
**Usage**: 
```bash
cargo run --example basic
```
**What it shows**:
- Layer parsing and categorization (signal vs other layers)
- Basic file statistics and complexity estimation
- Error handling patterns

---

### 🔍 Get Layers

**File**: `get_layers.rs`  
**Purpose**: Focused layer extraction with detailed layer information  
**Usage**:
```bash
cargo run --example get_layers
```
**What it shows**:
- Detailed layer stack analysis
- Layer type categorization
- CAM workflow preparation

---

### 📚 Get Symbols

**File**: `get_symbols.rs`  
**Purpose**: Parse KiCad symbol library files  
**Usage**:
```bash
cargo run --example get_symbols
```
**What it shows**:
- Symbol library parsing
- Symbol metadata extraction
- Component library analysis

---

### 🎯 Get Positions ⭐ **RECOMMENDED**

**File**: `get_positions.rs`  
**Purpose**: **Real-world component position extraction** from actual PCB files  
**Usage**:
```bash
cargo run --example get_positions
```
**What it shows**:
- ✅ **Best example to try first** - just run it!
- Hybrid parsing (simple parser + regex)
- Component grouping by type (R, C, U, etc.)
- Natural sorting of references (R1, R2, R10)
- Board statistics and density analysis
- **Real results**: 413 components from FPGA board

**Dependencies**: Requires `regex = "1.10"` in your Cargo.toml

---

### 📊 Get Details

**File**: `get_details.rs`  
**Purpose**: Comprehensive PCB file analysis with detailed statistics  
**Usage**:
```bash
cargo run --example get_details
```
**What it shows**:
- Complete PCB file analysis
- Detailed statistics and metrics
- Board complexity assessment

---

### 🎨 Get 3D Models ⭐ **NEW**

**File**: `get_3d_models.rs`  
**Purpose**: Analyze 3D model coverage for PCB visualization  
**Usage**:
```bash
cargo run --example get_3d_models
```
**What it shows**:
- 3D model coverage statistics (which components have 3D models)
- Model type distribution (STEP, WRL, etc.)
- 3D library usage analysis
- Component types with/without 3D models
- Lists components missing 3D models for better visualization

**Dependencies**: Requires `regex = "1.10"` in your Cargo.toml

## Test Files

The `assets/` directory contains real KiCad files for testing:
- **`fpga.kicad_pcb`**: Real FPGA development board (8MB+, 413 components)
- Perfect for testing component extraction and performance

## Common Patterns

### Layer Analysis
```rust
use kiparse::pcb::parse_layers_only;

let content = std::fs::read_to_string("design.kicad_pcb")?;
let pcb = parse_layers_only(&content)?;

for (id, layer) in &pcb.layers {
    println!("Layer {}: {} ({})", id, layer.name, layer.layer_type);
}
```

### Component Extraction (Recommended)
```rust
use kiparse::pcb::parse_layers_only;
use regex::Regex;

// Get basic structure
let pcb = parse_layers_only(&content)?;

// Extract components with regex
let component_re = Regex::new(
    r#"(?s)\(footprint\s+"([^"]+)".*?\(property\s+"Reference"\s+"([^"]+)""#
)?;

for cap in component_re.captures_iter(&content) {
    println!("Component {}: {}", &cap[2], &cap[1]);
}
```

### Symbol Library Analysis
```rust
use kiparse::symbol::parse_symbol_lib;

let content = std::fs::read_to_string("library.kicad_sym")?;
let symbols = parse_symbol_lib(&content)?;

for symbol in &symbols {
    println!("Symbol: {}", symbol.name);
}
```

## Performance Notes

- **Layer parsing**: ~10 MB/s, works with all KiCad versions
- **Component extraction**: ~5 MB/s for targeted regex patterns
- **Symbol parsing**: ~15 MB/s with complete metadata

## Dependencies

Most examples only need the base `kiparse` crate. Component extraction examples require:

```toml
[dependencies]
kiparse = "0.1.0"
regex = "1.10"  # For component_positions_* examples
```

## Best Practices

1. **Start with layer parsing** - Always works, gives you basic file structure
2. **Use regex for specific data** - More reliable than complex parsing
3. **Handle errors gracefully** - KiCad files can be large and complex
4. **Test with real files** - Use the provided FPGA board example

## Need Help?

- Check the main README.md for architecture details
- Look at KNOWN_ISSUES.md for current limitations
- Submit issues at: https://github.com/saturn77/KiParse/issues