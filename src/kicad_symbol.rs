use chrono::Utc;

#[derive(Debug, Clone)]
pub struct KicadSymbol {
    pub name: String,
    pub reference: String,
    pub value: String,
    pub footprint: String,
    pub datasheet: String,
    pub keywords: String,
    pub description: String,
}

impl KicadSymbol {
    pub fn new(name: String, value: String, footprint: String) -> Self {
        let description = format!("Resistor, {}", value);
        KicadSymbol {
            name,
            reference: "R".to_string(),
            value,
            footprint,
            datasheet: "~".to_string(),
            keywords: "R res resistor".to_string(),
            description,
        }
    }

    pub fn generate_symbol(&self) -> String {
        format!(r#"  (symbol "{}" (pin_numbers hide) (pin_names (offset 0)) (in_bom yes) (on_board yes)
    (property "Reference" "{}" (id 0) (at 2.032 0 90)
      (effects (font (size 1.27 1.27)))
    )
    (property "Value" "{}" (id 1) (at 0 0 90)
      (effects (font (size 1.27 1.27)))
    )
    (property "Footprint" "{}" (id 2) (at -1.778 0 90)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "Datasheet" "{}" (id 3) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "ki_keywords" "{}" (id 4) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "ki_description" "{}" (id 5) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "ki_fp_filters" "R_*" (id 6) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (symbol "{}_0_1"
      (rectangle (start -1.016 -2.54) (end 1.016 2.54)
        (stroke (width 0.254) (type default) (color 0 0 0 0))
        (fill (type none))
      )
    )
    (symbol "{}_1_1"
      (pin passive line (at 0 3.81 270) (length 1.27)
        (name "~" (effects (font (size 1.27 1.27))))
        (number "1" (effects (font (size 1.27 1.27))))
      )
      (pin passive line (at 0 -3.81 90) (length 1.27)
        (name "~" (effects (font (size 1.27 1.27))))
        (number "2" (effects (font (size 1.27 1.27))))
      )
    )
  )"#,
            self.name,
            self.reference,
            self.value,
            self.footprint,
            self.datasheet,
            self.keywords,
            self.description,
            self.name,
            self.name
        )
    }
}

pub struct KicadSymbolLib {
    pub symbols: Vec<KicadSymbol>,
}

impl KicadSymbolLib {
    pub fn new() -> Self {
        KicadSymbolLib {
            symbols: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: KicadSymbol) {
        self.symbols.push(symbol);
    }

    pub fn generate_library(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d");
        let mut lib_content = format!(
            "(kicad_symbol_lib (version 20211014) (generator atlantix-eda)\n"
        );

        for symbol in &self.symbols {
            lib_content.push_str(&symbol.generate_symbol());
            lib_content.push('\n');
        }

        lib_content.push_str(")\n");
        lib_content
    }
}