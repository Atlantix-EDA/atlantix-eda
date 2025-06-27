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
    pub symbol_style: String,
    pub manufacturer: String,
    pub mpn: String,
    pub supplier: String,
    pub supplier_pn: String,
    pub supplier_url: String,
}

impl KicadSymbol {
    pub fn new(name: String, value: String, footprint: String, symbol_style: &str) -> Self {
        let description = format!("Resistor, {}", value);
        KicadSymbol {
            name,
            reference: "R".to_string(),
            value,
            footprint,
            datasheet: "~".to_string(),
            keywords: "R res resistor".to_string(),
            description,
            symbol_style: symbol_style.to_string(),
            manufacturer: String::new(),
            mpn: String::new(),
            supplier: String::new(),
            supplier_pn: String::new(),
            supplier_url: String::new(),
        }
    }

    pub fn with_manufacturer_info(mut self, manufacturer: String, mpn: String, supplier: String, supplier_pn: String, supplier_url: String) -> Self {
        self.manufacturer = manufacturer;
        self.mpn = mpn;
        self.supplier = supplier;
        self.supplier_pn = supplier_pn;
        self.supplier_url = supplier_url;
        self
    }

    pub fn generate_symbol(&self) -> String {
        let symbol_geometry = match self.symbol_style.as_str() {
            "american" => self.generate_american_geometry(),
            "european" | _ => self.generate_european_geometry(),
        };

        let manufacturer_properties = if !self.manufacturer.is_empty() {
            format!(r#"
    (property "Manufacturer" "{}" (id 7) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "MPN" "{}" (id 8) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "Supplier" "{}" (id 9) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "SupplierPN" "{}" (id 10) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )
    (property "SupplierURL" "{}" (id 11) (at 0 0 0)
      (effects (font (size 1.27 1.27)) hide)
    )"#, self.manufacturer, self.mpn, self.supplier, self.supplier_pn, self.supplier_url)
        } else {
            String::new()
        };

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
    ){}
    (symbol "{}_0_1"
{}
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
            manufacturer_properties,
            self.name,
            symbol_geometry,
            self.name
        )
    }

    fn generate_european_geometry(&self) -> String {
        "      (rectangle (start -1.016 -2.54) (end 1.016 2.54)
        (stroke (width 0.254) (type default) (color 0 0 0 0))
        (fill (type none))
      )".to_string()
    }

    fn generate_american_geometry(&self) -> String {
        r#"      (polyline
        (pts
          (xy 0 -2.54)
          (xy 0.635 -1.905)
          (xy -0.635 -0.635)
          (xy 0.635 0.635)
          (xy -0.635 1.905)
          (xy 0 2.54)
        )
        (stroke (width 0.254) (type default) (color 0 0 0 0))
        (fill (type none))
      )"#.to_string()
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