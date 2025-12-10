# Known Issues

## Current Status

✅ **Fixed**: Removed the problematic full parser that was causing issues with modern KiCad files.

## Current Limitations

### PCB File Parsing

The library currently provides:
- ✅ **Layer extraction**: Fast, reliable parsing of layer definitions
- ✅ **Component position extraction**: Via regex patterns (see examples)
- ❌ **Full semantic parsing**: Not currently supported

For extracting component data, tracks, vias, etc., use the regex-based approach demonstrated in the `component_positions_working.rs` example.

### Workarounds

1. **Component Data**: Use regex patterns to extract specific information
2. **Track/Via Analysis**: Parse using targeted regex for the specific data needed
3. **DRC-style Analysis**: Combine layer parsing with custom regex extraction

## Example: Custom Component Extraction

```rust
use regex::Regex;

// Extract all component references and positions
let component_re = Regex::new(
    r#"(?s)\(footprint\s+"([^"]+)".*?\(property\s+"Reference"\s+"([^"]+)".*?\(at\s+([\d.-]+)\s+([\d.-]+)"#
)?;

for cap in component_re.captures_iter(&content) {
    let footprint = &cap[1];
    let reference = &cap[2]; 
    let x: f64 = cap[3].parse().unwrap();
    let y: f64 = cap[4].parse().unwrap();
    // Process component...
}
```

## Future Plans

- Focus on improving regex-based extraction patterns
- Add more utility functions for common parsing tasks  
- Consider a streaming approach for very large files

## Reporting Issues

For new issues, please provide:
1. KiCad version that created the file
2. Specific parsing task you're trying to accomplish
3. Sample file (if possible) or minimal reproduction

Submit issues at: https://github.com/saturn77/KiParse/issues