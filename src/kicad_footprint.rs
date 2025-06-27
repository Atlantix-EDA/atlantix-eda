use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Pad {
    pub number: String,
    pub pad_type: String,
    pub shape: String,
    pub at_x: f64,
    pub at_y: f64,
    pub size_x: f64,
    pub size_y: f64,
    pub roundrect_rratio: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct KicadFootprint {
    pub name: String,
    pub description: String,
    pub tags: String,
    pub pads: Vec<Pad>,
    pub body_size_x: f64,
    pub body_size_y: f64,
    pub courtyard_margin: f64,
}

impl KicadFootprint {
    pub fn new_smd_resistor(package: &str) -> Option<Self> {
        let specs = get_package_specs(package)?;
        
        let name = format!("R_{}_{}", specs.imperial, specs.metric);
        let description = format!(
            "Resistor SMD {} ({}), square (rectangular) end terminal, IPC_7351 nominal",
            specs.imperial, specs.metric
        );
        
        let pads = vec![
            Pad {
                number: "1".to_string(),
                pad_type: "smd".to_string(),
                shape: "roundrect".to_string(),
                at_x: -specs.pad_center_x,
                at_y: 0.0,
                size_x: specs.pad_width,
                size_y: specs.pad_height,
                roundrect_rratio: Some(0.25),
            },
            Pad {
                number: "2".to_string(),
                pad_type: "smd".to_string(),
                shape: "roundrect".to_string(),
                at_x: specs.pad_center_x,
                at_y: 0.0,
                size_x: specs.pad_width,
                size_y: specs.pad_height,
                roundrect_rratio: Some(0.25),
            },
        ];
        
        Some(KicadFootprint {
            name,
            description,
            tags: "resistor".to_string(),
            pads,
            body_size_x: specs.body_length,
            body_size_y: specs.body_width,
            courtyard_margin: 0.25,
        })
    }
    
    pub fn generate_footprint(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let courtyard_x = self.body_size_x / 2.0 + self.courtyard_margin;
        let courtyard_y = self.body_size_y / 2.0 + self.courtyard_margin;
        
        let mut footprint = format!(
            r#"(module {} (layer F.Cu) (tedit {})
  (descr "{}")
  (tags {})
  (attr smd)
  (fp_text reference REF** (at 0 -{:.2}) (layer F.SilkS)
    (effects (font (size 1 1) (thickness 0.15)))
  )
  (fp_text value {} (at 0 {:.2}) (layer F.Fab)
    (effects (font (size 1 1) (thickness 0.15)))
  )
"#,
            self.name,
            timestamp,
            self.description,
            self.tags,
            self.body_size_y / 2.0 + 1.0,
            self.name,
            self.body_size_y / 2.0 + 1.0
        );
        
        // Fabrication layer outline
        let half_x = self.body_size_x / 2.0;
        let half_y = self.body_size_y / 2.0;
        footprint.push_str(&format!(
            "  (fp_line (start -{:.3} {:.3}) (end -{:.3} -{:.3}) (layer F.Fab) (width 0.1))\n",
            half_x, half_y, half_x, half_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start -{:.3} -{:.3}) (end {:.3} -{:.3}) (layer F.Fab) (width 0.1))\n",
            half_x, half_y, half_x, half_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start {:.3} -{:.3}) (end {:.3} {:.3}) (layer F.Fab) (width 0.1))\n",
            half_x, half_y, half_x, half_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start {:.3} {:.3}) (end -{:.3} {:.3}) (layer F.Fab) (width 0.1))\n",
            half_x, half_y, half_x, half_y
        ));
        
        // Silkscreen lines (partial, not over pads)
        let silk_offset = 0.15;
        let silk_x = half_x - self.pads[0].size_x / 2.0 - silk_offset;
        footprint.push_str(&format!(
            "  (fp_line (start -{:.3} -{:.3}) (end {:.3} -{:.3}) (layer F.SilkS) (width 0.12))\n",
            silk_x, half_y + 0.11, silk_x, half_y + 0.11
        ));
        footprint.push_str(&format!(
            "  (fp_line (start -{:.3} {:.3}) (end {:.3} {:.3}) (layer F.SilkS) (width 0.12))\n",
            silk_x, half_y + 0.11, silk_x, half_y + 0.11
        ));
        
        // Courtyard
        footprint.push_str(&format!(
            "  (fp_line (start -{:.2} {:.2}) (end -{:.2} -{:.2}) (layer F.CrtYd) (width 0.05))\n",
            courtyard_x, courtyard_y, courtyard_x, courtyard_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start -{:.2} -{:.2}) (end {:.2} -{:.2}) (layer F.CrtYd) (width 0.05))\n",
            courtyard_x, courtyard_y, courtyard_x, courtyard_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start {:.2} -{:.2}) (end {:.2} {:.2}) (layer F.CrtYd) (width 0.05))\n",
            courtyard_x, courtyard_y, courtyard_x, courtyard_y
        ));
        footprint.push_str(&format!(
            "  (fp_line (start {:.2} {:.2}) (end -{:.2} {:.2}) (layer F.CrtYd) (width 0.05))\n",
            courtyard_x, courtyard_y, courtyard_x, courtyard_y
        ));
        
        // Pads
        for pad in &self.pads {
            footprint.push_str(&format!(
                "  (pad {} {} {} (at {:.3} {:.3}) (size {:.2} {:.2}) (layers F.Cu F.Paste F.Mask)",
                pad.number, pad.pad_type, pad.shape, pad.at_x, pad.at_y, pad.size_x, pad.size_y
            ));
            if let Some(rratio) = pad.roundrect_rratio {
                footprint.push_str(&format!(" (roundrect_rratio {:.2})", rratio));
            }
            footprint.push_str(")\n");
        }
        
        // 3D model reference
        footprint.push_str(&format!(
            r#"  (model ${{KICAD6_3DMODEL_DIR}}/Resistor_SMD.3dshapes/{}.wrl
    (at (xyz 0 0 0))
    (scale (xyz 1 1 1))
    (rotate (xyz 0 0 0))
  )
)
"#,
            self.name
        ));
        
        footprint
    }
}

struct PackageSpec {
    imperial: &'static str,
    metric: &'static str,
    body_length: f64,
    body_width: f64,
    pad_width: f64,
    pad_height: f64,
    pad_center_x: f64,
}

fn get_package_specs(package: &str) -> Option<PackageSpec> {
    match package {
        "0201" => Some(PackageSpec {
            imperial: "0201",
            metric: "0603Metric",
            body_length: 0.6,
            body_width: 0.3,
            pad_width: 0.28,
            pad_height: 0.43,
            pad_center_x: 0.26,
        }),
        "0402" => Some(PackageSpec {
            imperial: "0402",
            metric: "1005Metric",
            body_length: 1.0,
            body_width: 0.5,
            pad_width: 0.6,
            pad_height: 0.65,
            pad_center_x: 0.48,
        }),
        "0603" => Some(PackageSpec {
            imperial: "0603",
            metric: "1608Metric",
            body_length: 1.6,
            body_width: 0.8,
            pad_width: 0.9,
            pad_height: 0.95,
            pad_center_x: 0.775,
        }),
        "0805" => Some(PackageSpec {
            imperial: "0805",
            metric: "2012Metric",
            body_length: 2.0,
            body_width: 1.25,
            pad_width: 1.0,
            pad_height: 1.45,
            pad_center_x: 0.95,
        }),
        "1206" => Some(PackageSpec {
            imperial: "1206",
            metric: "3216Metric",
            body_length: 3.2,
            body_width: 1.6,
            pad_width: 1.15,
            pad_height: 1.8,
            pad_center_x: 1.475,
        }),
        "1210" => Some(PackageSpec {
            imperial: "1210",
            metric: "3225Metric",
            body_length: 3.2,
            body_width: 2.5,
            pad_width: 1.15,
            pad_height: 2.7,
            pad_center_x: 1.475,
        }),
        "2010" => Some(PackageSpec {
            imperial: "2010",
            metric: "5025Metric",
            body_length: 5.0,
            body_width: 2.5,
            pad_width: 1.5,
            pad_height: 2.8,
            pad_center_x: 2.25,
        }),
        "2512" => Some(PackageSpec {
            imperial: "2512",
            metric: "6332Metric",
            body_length: 6.35,
            body_width: 3.2,
            pad_width: 1.6,
            pad_height: 3.5,
            pad_center_x: 2.875,
        }),
        _ => None,
    }
}