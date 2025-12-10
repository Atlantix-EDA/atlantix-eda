<div align="center">
<img width=260 height=260 src="./assets/media/KiParse Logo - Flat Vector Style.png"></img>

# KiParse
*Transforming engineering data is the foundation of EDA workflows.*



[![Crates.io](https://img.shields.io/crates/v/kiparse)](https://crates.io/crates/kiparse)
[![Documentation](https://docs.rs/kiparse/badge.svg)](https://docs.rs/kiparse)
[![Tests](https://github.com/saturn77/KiParse/workflows/Rust/badge.svg)](https://github.com/saturn77/KiParse/actions)
[![KiCad Version](https://img.shields.io/badge/KiCad-9.0+-blue)](https://www.kicad.org/)
[![MSRV](https://img.shields.io/badge/MSRV-1.65.0-blue)](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

</div>

Transforming data is the foundation of modern Electronic Design Automation workflows, 
employing lexers, parsers, and syntax analysis. 

KiParse is a practical KiCad parser written in Rust, operating on native KiCad files and formats. It provides reliable data useful for developers and designers during the course of a design, including an integrated CLI tool `kpx` for rapid PCB analysis and automation workflows.

This tool is for designers and developers using KiCad, and was created to serve
the needs of a CAM tool being developed in the [`KiForge`](https://github.com/Saturn77/KiForge.git) repository. In `KiForge` there are various CAM integrations being put
in place, and the necessity of a full parser became apparent. 

Additionally the latest version of KiCad has gone to IPC communications with both Python and [Rust bindings](https://gitlab.com/kicad/code/kicad-rs), however, the authors view is that native file parsing is still the
preferred approach. 

Finally, the methodology used for the `pcb parsing` does not employ any standard lexer or parsing library from Rust such as nom, pest, or logos. However, [`logos`](https://github.com/maciejhirsz/logos) is used for analysis
of symbol library files. 


## Testing and Verification for KiParse

Due to the nature of lexers, parsers, and semantic analysis, and based on 
years of experience developing parsers for many real world applications, a high priority was placed on testing the software with a complex KiCad example right
from the start. 

The thought was to choose an open source example that was on GitHub and was
a FPGA style board with decent complexity. The board chosen for all the tests,
and the file that is in the `assets` directory as `fpga.kicad_pcb`, comes from
the author AAWO and the repository : 

https://github.com/AAWO/CPArti-FPGA-Development-Board

The design has well over 400 components and the pcb file itself is very large at over 250,000 lines of code. This 
was considered a good test of KiParse. 

## Quick Start

```bash
git clone https://github.com/saturn77/KiParse.git
cd KiParse
cargo build
cargo test
```

### Feature Flags

If using kiparse from another project, the cli feature is what enables the kpx command line tool. 

| Feature | Description | Default |
|---------|-------------|---------|
| `cli` | Command-line interface (`kpx` binary) | ❌ |
| `json` | JSON serialization support | ❌ |
| `serde` | Serde serialization for all types | ✅ |

```toml
[dependencies]
kiparse = { version = "0.1.0", features = ["cli", "json"] }
```

## Examples

The repository includes several examples demonstrating common use cases:

```bash
# All examples work immediately - no arguments needed!
cargo run --example basic              # Basic PCB analysis
cargo run --example get_layers         # Extract layer information
cargo run --example get_positions      # Extract component positions
cargo run --example get_symbols        # Parse symbol libraries
cargo run --example get_details        # Detailed PCB information
cargo run --example get_3d_models      # Analyze 3D model coverage
cargo run --example two_stage_parsing  # Demonstrates the two-stage parser architecture
```

## Command Line Interface

KiParse includes a powerful CLI tool called `kpx` (KiParse eXtended) with subcommands for different analysis tasks:

```bash
# Build and run from source
cargo run --bin kpx --features cli -- <file> <command> [options]

# Once published to crates.io, you'll be able to install with:
# cargo install kiparse --features cli
# This installs the 'kpx' binary to your PATH

# Usage examples:
kpx board.kicad_pcb details          # Get detailed PCB information
kpx board.kicad_pcb layers           # Extract layer information
kpx board.kicad_pcb 3d               # Analyze 3D model coverage
kpx board.kicad_pcb positions        # Extract component positions
kpx components.kicad_sym symbols     # Parse symbol libraries

# JSON output for any command:
kpx board.kicad_pcb details --json
kpx board.kicad_pcb 3d --json

# Get help:
kpx --help
kpx <file> --help
```


## Two-Stage Parser Architecture

KiParse implements a `Two-Stage Parser Process` for optimal performance and flexibility:

**Stage 1: Simple Parser** (`parse_layers_only`)
- Fast extraction of PCB layers and basic structure
- Uses simple string methods for reliability
- Provides foundation for further analysis

**Stage 2: Detail Parser** (`DetailParser`)
- Extracts specific elements: components, tracks, vias, 3D models
- Uses optimized regex patterns with lazy static compilation
- Operates on the raw content for detailed analysis

This architecture allows you to quickly get layer information, then selectively extract only the details you need.

## Real-World Usage Examples 

### Stage 1: Layer Information

```rust
use kiparse::pcb::parse_layers_only;

fn analyze_layer_stack(pcb_file: &str) -> kiparse::Result<()> {
    let content = std::fs::read_to_string(pcb_file)?;
    let pcb = parse_layers_only(&content)?;
    
    println!("Layer stack analysis:");
    let mut signal_layers = 0;
    
    for (id, layer) in &pcb.layers {
        println!("Layer {}: {} ({})", id, layer.name, layer.layer_type);
        if layer.layer_type == "signal" {
            signal_layers += 1;
        }
    }
    
    println!("Total signal layers: {}", signal_layers);
    Ok(())
}
```

### Stage 2: Detailed Element Extraction

```rust
use kiparse::prelude::*;

fn extract_board_details(pcb_file: &str) -> kiparse::Result<()> {
    let content = std::fs::read_to_string(pcb_file)?;
    
    // Stage 1: Get layers
    let pcb = parse_layers_only(&content)?;
    println!("Found {} layers", pcb.layers.len());
    
    // Stage 2: Extract detailed information
    let detail_parser = DetailParser::new(&content);
    
    // Extract components with positions
    let components = detail_parser.extract_components()?;
    println!("Found {} components", components.len());
    
    // Extract board dimensions
    if let Some(outline) = detail_parser.extract_board_outline()? {
        println!("Board size: {:.1} × {:.1} mm", 
                 outline.width_mm, outline.height_mm);
    }
    
    // Extract 3D model coverage
    let models = detail_parser.extract_3d_models()?;
    println!("3D models: {} / {}", models.len(), components.len());
    
    Ok(())
}
```

### Component Analysis

```rust
use kiparse::pcb::PcbParser;
use std::collections::HashMap;

fn analyze_components(pcb_file: &str) -> kiparse::Result<()> {
    let content = std::fs::read_to_string(pcb_file)?;
    let pcb = PcbParser::parse_from_str(&content)?;
    
    let mut component_counts = HashMap::new();
    
    for footprint in &pcb.footprints {
        let component_type = footprint.name.split(':').next().unwrap_or("Unknown");
        *component_counts.entry(component_type).or_insert(0) += 1;
    }
    
    println!("Component Summary:");
    for (component, count) in component_counts {
        println!("  {}: {} instances", component, count);
    }
    
    Ok(())
}
```

### Component Details

```rust
use kiparse::pcb::PcbParser;

fn extract_component_positions(pcb_file: &str) -> kiparse::Result<()> {
    let content = std::fs::read_to_string(pcb_file)?;
    let pcb = PcbParser::parse_from_str(&content)?;
    
    println!("Reference,X(mm),Y(mm),Rotation,Layer");
    
    for footprint in &pcb.footprints {
        let reference = footprint.properties.get("Reference")
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        
        println!("{},{:.3},{:.3},{},{}",
                 reference,
                 footprint.position.x,
                 footprint.position.y,
                 footprint.rotation,
                 footprint.layer);
    }
    
    Ok(())
}
```













## Current Supported File Formats

| Format | Extension | Parser Status | Description |
|--------|-----------|--------------|-------------|
| PCB Files | `.kicad_pcb` | ✅ Layer Extraction | Fast layer parsing + regex-based component extraction |
| Symbol Libraries | `.kicad_sym` | ✅ Complete | Component symbol definitions and metadata |
| Schematics | `.kicad_sch` | 🚧 Planned | Schematic capture files |
| Footprint Libraries | `.kicad_mod` | 🚧 Planned | Footprint definitions |

> **KiCad Compatibility**: This library is continuously tested against **KiCad Nightly builds** to ensure compatibility with the latest file format changes. The included FPGA board example (`assets/fpga.kicad_pcb`) was created with KiCad 9.99 and serves as a reference for format compatibility.

## Roadmap

- [ ] **v0.2**: Schematic file support (`.kicad_sch`)
- [ ] **v0.3**: Footprint library support (`.kicad_mod`)
- [ ] **v0.4**: Project file support (`.kicad_pro`)
- [ ] **v0.5**: Write support (generate KiCad files)
- [ ] **v1.0**: Full KiCad file format support

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Disclaimer

⚠️ **This software is provided "as is" without warranty of any kind. It is intended for development, testing, and educational purposes. Users should thoroughly validate any results before use in critical applications.**

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- KiCad development team for the excellent EDA software
- Rust community for the amazing ecosystem
- Contributors and users providing feedback and testing

## Related Projects

- [KiCad](https://www.kicad.org/) - Free and open-source EDA software
- [kikit](https://github.com/yaqwsx/KiKit) - Python tooling for KiCad
- [kicad-automation-scripts](https://github.com/productize/kicad-automation-scripts) - KiCad automation tools

---

