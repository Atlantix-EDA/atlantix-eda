//! The generate library module. Contains the primary methods for generating Resistor values.
//!

extern crate num_traits;

use self::num_traits::Pow;

///
/// Resistor type data structure
///
/// # Structure members
///
/// * `series`         - The series such as E-96, E-48, E-24 for resistor values.
/// * `name`           - Resistor name as you want it to appear in your PCB library.
/// * `full_part_name` - Full name that is CSV formatted and writtent to a file.
/// * `value`          - Ohmic value, such as 1.00K, 4.99K, 100K, etc.
/// * `manuf`          - Vishay, KOA, Panasonic, etc. Currently Vishay is implemented.
/// * `case`           - The case size, such as 0402, 0603, 0805, 1206, etc.
/// * `power`          - power rating which is corresponding to the package/case.
/// * `series_array`   - Vector of floating point values for the resistor series.
///
/// # Remarks
///
/// This structure can be extended to include other fields that one may want
/// in the library data. Overall this is targeted at Altium but could easily
/// be extened for other EDA software.
///
/// *Note*: One may want to have manuf_1, manuf_2, manuf_3, etc.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Resistor {
    display: bool,
    series: usize,
    name: String,
    full_part_name: String,
    full_series: String,
    value: String,
    manuf: String,
    case: String,
    power: String,
    series_array: Vec<f64>,
}

impl Resistor {
    /// Impl Function : new (constructor)
    /// #  Remarks
    ///
    /// Constructor for the Resistor object with various
    /// parameters determined by introspection and matching.
    /// Instantiating this object by by passing the series
    /// and package arguments will construct a new Resistor
    /// object. The Resistor object can then be iterated upon
    /// to generate all the library values for this package/series combination.
    ///
    /// 	pub fn new(eseries : usize, package : String ) -> Resistor {
    /// 	let mut alpha = vec![0.0 ; eseries];
    ///     for index in 0..eseries{
    ///         let gamma : f64 = Pow::pow(10.0,index as f32/eseries as f32);
    ///         alpha[index] = (gamma * 100.0).round() / 100.0;
    ///     }
    /// 	let watts : String;
    /// 		match package.as_ref() {
    /// 			"0201" => watts = "1/20".to_string(),
    /// 			"0402" => watts = "1/16".to_string(),
    /// 			"0603" => watts = "1/10".to_string(),
    /// 			"0805" => watts = "1/8".to_string(),
    /// 			"1206" => watts = "1/4".to_string(),
    /// 			"1210" => watts = "1/2".to_string(),
    /// 			"1812" => watts = "1".to_string(),
    /// 			"2010" => watts = "3/4".to_string(),
    /// 			"2512" => watts = "1".to_string(),
    /// 			_ => watts = "0".to_string(),
    /// 		};
    ///
    /// 		Resistor {
    /// 			display : false,
    /// 			series : eseries,
    /// 			name  : "RES".to_string() + &package + &"_".to_string() + &"1.00K".to_string(),
    /// 			full_part_name : "RES".to_string() + &package + &"_".to_string() + &"1.00K".to_string(),
    /// 			full_series : "".to_string(),
    /// 			value : "1.00K".to_string(),
    /// 			manuf : "Vishay".to_string(),
    /// 			case  : package,
    /// 			power : watts,
    /// 			series_array : alpha,
    /// 		}
    ///
    /// 	}
    ///
    pub fn new(eseries: usize, package: String) -> Resistor {
        let mut alpha = vec![0.0; eseries];
        for index in 0..eseries {
            let gamma: f64 = Pow::pow(10.0, index as f32 / eseries as f32);
            alpha[index] = (gamma * 100.0).round() / 100.0;
        }
        let watts: String;
        match package.as_ref() {
            "0201" => watts = "1/20".to_string(),
            "0402" => watts = "1/16".to_string(),
            "0603" => watts = "1/10".to_string(),
            "0805" => watts = "1/8".to_string(),
            "1206" => watts = "1/4".to_string(),
            "1210" => watts = "1/2".to_string(),
            "1218" => watts = "1".to_string(),
            "2010" => watts = "3/4".to_string(),
            "2512" => watts = "1".to_string(),
            _ => watts = "0".to_string(),
        };

        Resistor {
            display: false,
            series: eseries,
            name: "RES".to_string() + &package + &"_".to_string() + &"1.00K".to_string(),
            full_part_name: "RES".to_string() + &package + &"_".to_string() + &"1.00K".to_string(),
            full_series: "".to_string(),
            value: "1.00K".to_string(),
            manuf: "Vishay".to_string(),
            case: package,
            power: watts,
            series_array: alpha,
        }
    }
    ///  Impl Function : set_vishay_num
    ///  #  Remarks
    ///
    /// This will assign a Manufacturer or Vendor number to the self.manuf field.
    /// This is true for all decades other than decade 1, which has special exception.
    ///
    /// ```
    /// pub fn set_vishay_num(&mut self) -> String {
    ///		match self.case {
    /// 		0402 => self.manuf = format!("541-%sLCT-ND",self.series_array[index]);
    ///			0603 => self.manuf = format!("541-%sHCT-ND",self.series_array[index]);
    ///			0805 => self.manuf = format!("541-%sCCT-ND",self.series_array[index])
    ///			}
    /// }
    ///
    /// ```
    pub fn set_vishay_num(&mut self, index: usize, decade: u32) {
        if decade == 1 {
            match self.case.as_str() {
                "0402" => self.manuf = format!("541-{}LLCT-ND", self.series_array[index]),
                "0603" => self.manuf = format!("541-{}HHCT-ND", self.series_array[index]),
                "0805" => self.manuf = format!("541-{}CCCT-ND", self.series_array[index]),
                "1206" => self.manuf = format!("541-{}FFCT-ND", self.series_array[index]),
                "1210" => self.manuf = format!("541-{}AACT-ND", self.series_array[index]),
                "1218" => self.manuf = format!("541-{}ANCT-ND", self.series_array[index]),
                "2010" => self.manuf = format!("541-{}ACCT-ND", self.series_array[index]),
                "2512" => self.manuf = format!("541-{}AFCT-ND", self.series_array[index]),
                _ => self.manuf = format!("541-{}XXXX-ND", self.series_array[index]),
            }
        } else {
        match self.case.as_str() {
            "0402" => self.manuf = format!("541-{}LCT-ND", self.value),
            "0603" => self.manuf = format!("541-{}HCT-ND", self.value),
            "0805" => self.manuf = format!("541-{}CCT-ND", self.value),
            "1206" => self.manuf = format!("541-{}FCT-ND", self.value),
            "1210" => self.manuf = format!("541-{}VCT-ND", self.value),
            "1218" => self.manuf = format!("541-{}KANCT-ND", self.value),
            "2010" => self.manuf = format!("541-{}KACCT-ND", self.value),
            "2512" => self.manuf = format!("541-{}KAFCT-ND", self.value),
            _ => self.manuf = format!("541-{}XXX-ND", self.value),
        }
    }
    }

    ///  Impl Resistor : set_name
    ///  #  Remarks
    ///
    ///  This is a helper function for set_full_name.
    ///
    /// ```
    /// pub fn set_name(&mut self) -> String {
    ///		"RES".to_string() + &self.case + &"_".to_string() + &self.value
    ///	}
    /// ```
    pub fn set_name(&mut self) -> String {
        "RES".to_string() + &self.case + &"_".to_string() + &self.value
    }

    ///  Impl Resistor : set_full_name
    ///  # Remarks
    ///
    ///  Assigns the full name of the component, as would be preferred to see in a resistor library. 
    ///  For example, when browsing in Altium seeing RES0402_1.00K or R0603_2.49K.
    ///
    pub fn set_full_name(&mut self) {
        self.name = self.set_name()
    }

    ///  Impl Resistor : set_part_string
    ///  #  Remarks
    ///
    ///  Populates a string with all the part's information.
    ///  Item, Description, Value, Case, Power, Supplier 1, Supplier Part Number 1, Library Path, Library Ref, Footprint Path, Footprint Ref, Company
    /// 
    pub fn set_part(&mut self) -> String {
        "RES".to_string()
            + &self.case
            + &"_".to_string()
            + &self.value + &", ".to_string()
            + &"\"".to_string() + &"RES " + &self.case + &" ".to_string() +  &self.value + &"Ohm ".to_string() + &self.power + &"W\", "
            + &self.value
            + &", ".to_string()
            + &self.case
            + &", ".to_string()
            + &self.power
            + &", ".to_string()
            + &"Digikey, ".to_string()
            + &self.manuf
            + &", ".to_string()
            + &"Atlantix_R.SchLib, ".to_string()
            + &"Res1, ".to_string()
            + &"Atlantix_R.PcbLib, ".to_string()
            + &"RES".to_string() + &self.case + &" , ".to_string()
            + &"Atlantix EDA, =Description ".to_string()
            + &"\r\n".to_string()
    }

    ///  Impl Resistor : function set_full_part_name
    ///  # Remarks
    ///
    ///  Assigns the full name of the component, as would be
    ///  preferred to see in a resistor library, such as
    ///  'R0402_1.00K' or 'R0805_4.99K' for ease of browsing
    ///  in tools such as Altium.
    ///
    ///
    pub fn set_full_part_name(&mut self) {
        self.full_part_name = self.set_part()
    }

    ///  Impl Resistor : function generate
    ///  # Remarks
    ///
    ///  Assigns the full name of the component, as would be
    ///  preferred to see in a resistor library, such as
    ///  'R0402_1.00K' or 'R0805_4.99K' for ease of browsing
    ///  in tools such as Altium.
    ///
    ///
    pub fn generate(&mut self, decade: u32) -> String {
        for index in 0..self.series {
            match decade {
                1 => {
                    self.value = format!("{:.2}", self.series_array[index]);
                    self.set_vishay_num(index, decade)
                }
                10 => {
                    self.value = format!("{:2.1}", (decade as f64) * self.series_array[index]);
                    self.set_vishay_num(index, decade)
                }
                100 => {
                    self.value = format!("{:3.0}", (decade as f64) * self.series_array[index]);
                    self.set_vishay_num(index, decade)
                }
                1000 => {
                    self.value = format!("{:.2}", self.series_array[index]) + &"K".to_string();
                    self.set_vishay_num(index, decade)
                }
                10000 => {
                    self.value = format!("{:2.1}", (10 as f64) * self.series_array[index])
                        + &"K".to_string();
                    self.set_vishay_num(index, decade)
                }
                100000 => {
                    self.value = format!("{:3.0}", (100 as f64) * self.series_array[index])
                        + &"K".to_string();
                    self.set_vishay_num(index, decade)
                }
                _ => (),
            }

            self.set_full_name();
            self.set_full_part_name();
            self.full_series += &self.full_part_name;
        }
        let alpha = &self.full_series;
        return alpha.to_string();
    }


}
