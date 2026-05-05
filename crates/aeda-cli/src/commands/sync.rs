//! Verify ref-designator + footprint + value consistency between a KiCad PCB
//! and a netlist exported from Eeschema.
//!
//! PCB side: kiparse `DetailParser`.
//! Schematic side: parsed from a `.net` file (KiCad netlist export). The netlist
//! is the same data Eeschema's "Update PCB from Schematic" uses internally,
//! which makes it the right source-of-truth for refs without re-implementing
//! hierarchical-sheet walking.

use kiparse::pcb::detail_parser::ComponentInfo;
use kiparse::pcb::DetailParser;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
struct NetComponent {
    reference: String,
    value: String,
    footprint: String,
    sheetname: Option<String>,
    sheetfile: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct SyncReport {
    pcb_path: String,
    netlist_path: String,
    pcb_component_count: usize,
    netlist_component_count: usize,
    stale_netlist_warning: Option<String>,
    missing_from_pcb: Vec<NetComponent>,
    extra_on_pcb: Vec<PcbOnlyEntry>,
    value_mismatches: Vec<FieldMismatch>,
    footprint_mismatches: Vec<FieldMismatch>,
}

#[derive(Debug, Clone, Serialize)]
struct PcbOnlyEntry {
    reference: String,
    footprint: String,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FieldMismatch {
    reference: String,
    schematic: String,
    pcb: String,
}

pub fn run(pcb_path: &Path, sch_or_net: &Path, json: bool) -> Result<(), String> {
    let pcb_content = fs::read_to_string(pcb_path)
        .map_err(|e| format!("Failed to read PCB file {}: {}", pcb_path.display(), e))?;

    // If given a .kicad_sch, auto-export a fresh netlist via kicad-cli.
    // The TempNetlist guard removes the file when it goes out of scope.
    let (net_path_owned, _temp_guard): (PathBuf, Option<TempNetlist>) =
        if is_schematic(sch_or_net) {
            let exported = export_netlist(sch_or_net)?;
            let path = exported.path.clone();
            (path, Some(exported))
        } else {
            (sch_or_net.to_path_buf(), None)
        };
    let net_path = net_path_owned.as_path();

    let net_content = fs::read_to_string(net_path)
        .map_err(|e| format!("Failed to read netlist {}: {}", net_path.display(), e))?;

    let pcb_components = DetailParser::new(&pcb_content)
        .extract_components()
        .map_err(|e| format!("kiparse failed to parse PCB: {}", e))?;
    let net_components = parse_netlist(&net_content);

    // Freshness only matters for user-supplied .net files; auto-exported is fresh by definition.
    let stale_warning = if _temp_guard.is_some() {
        None
    } else {
        check_netlist_freshness(pcb_path, net_path)
    };

    let report = build_report(
        pcb_path,
        net_path,
        pcb_components,
        net_components,
        stale_warning,
    );

    if json {
        let s = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("JSON serialize failed: {}", e))?;
        println!("{}", s);
    } else {
        print_report(&report);
    }

    let mismatch = !report.missing_from_pcb.is_empty()
        || !report.extra_on_pcb.is_empty()
        || !report.value_mismatches.is_empty()
        || !report.footprint_mismatches.is_empty();
    if mismatch {
        std::process::exit(2);
    }
    Ok(())
}

// --- kicad-cli auto-export ---------------------------------------------------

fn is_schematic(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()) == Some("kicad_sch")
}

/// RAII guard for an auto-exported netlist. Best-effort cleanup on Drop.
struct TempNetlist {
    path: PathBuf,
}

impl Drop for TempNetlist {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Build the kicad-cli argv. Honors `KICAD_CLI` env var (whitespace-split);
/// otherwise defaults to `flatpak run --command=kicad-cli org.kicad.KiCad`,
/// which is the stable 10.0.1 install on this machine.
fn kicad_cli_argv() -> Vec<String> {
    if let Ok(s) = std::env::var("KICAD_CLI") {
        let parts: Vec<String> = s.split_whitespace().map(|p| p.to_string()).collect();
        if !parts.is_empty() {
            return parts;
        }
    }
    vec![
        "flatpak".to_string(),
        "run".to_string(),
        "--command=kicad-cli".to_string(),
        "org.kicad.KiCad".to_string(),
    ]
}

/// Export a netlist for the given .kicad_sch via kicad-cli. Output is written
/// next to the schematic (flatpak sandboxing forbids /tmp writes), under a
/// hidden filename that's removed when the returned guard is dropped.
fn export_netlist(sch_path: &Path) -> Result<TempNetlist, String> {
    let parent = sch_path.parent().ok_or_else(|| {
        format!(
            "Schematic path has no parent directory: {}",
            sch_path.display()
        )
    })?;
    let stem = sch_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("schematic");
    let pid = std::process::id();
    let out_path = parent.join(format!(".aeda-sync-{}-{}.net", stem, pid));

    let mut argv = kicad_cli_argv();
    argv.extend([
        "sch".into(),
        "export".into(),
        "netlist".into(),
        "-o".into(),
        out_path.to_string_lossy().into_owned(),
        sch_path.to_string_lossy().into_owned(),
    ]);

    let output = Command::new(&argv[0])
        .args(&argv[1..])
        .output()
        .map_err(|e| {
            format!(
                "Failed to invoke kicad-cli ({}): {}. \
                 Override the binary with KICAD_CLI env var if needed.",
                argv[0], e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Don't leave a partial file behind.
        let _ = fs::remove_file(&out_path);
        return Err(format!(
            "kicad-cli sch export netlist failed (exit {}). \
             argv: {:?}\nstderr:\n{}\nstdout:\n{}",
            output.status.code().unwrap_or(-1),
            argv,
            stderr.trim(),
            stdout.trim()
        ));
    }

    if !out_path.exists() {
        return Err(format!(
            "kicad-cli reported success but {} was not created. \
             If using the flatpak, the sandbox may be blocking the write path.",
            out_path.display()
        ));
    }

    Ok(TempNetlist { path: out_path })
}

// --- Netlist parsing ---------------------------------------------------------

static COMP_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\(comp\s+\(ref\s+"([^"]+)"\)"#).unwrap());
static VALUE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\(value\s+"([^"]*)"\)"#).unwrap());
static FOOTPRINT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\(footprint\s+"([^"]*)"\)"#).unwrap());
static SHEETNAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\(property\s+\(name\s+"Sheetname"\)\s+\(value\s+"([^"]*)"\)\s*\)"#).unwrap()
});
static SHEETFILE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\(property\s+\(name\s+"Sheetfile"\)\s+\(value\s+"([^"]*)"\)\s*\)"#).unwrap()
});

fn parse_netlist(content: &str) -> Vec<NetComponent> {
    // Bracket each (comp ...) by its starting offset and slice up to the next
    // (comp or end of content. The first (value/(footprint hits inside that
    // slice are this component's fields.
    let starts: Vec<(usize, String)> = COMP_START
        .captures_iter(content)
        .map(|c| {
            let m = c.get(0).unwrap();
            (m.start(), c[1].to_string())
        })
        .collect();

    let mut out = Vec::with_capacity(starts.len());
    for (i, (start, reference)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map(|(s, _)| *s).unwrap_or(content.len());
        let block = &content[*start..end];

        let value = VALUE_RE
            .captures(block)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let footprint = FOOTPRINT_RE
            .captures(block)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let sheetname = SHEETNAME_RE.captures(block).map(|c| c[1].to_string());
        let sheetfile = SHEETFILE_RE.captures(block).map(|c| c[1].to_string());

        out.push(NetComponent {
            reference: reference.clone(),
            value,
            footprint,
            sheetname,
            sheetfile,
        });
    }
    out
}

// --- Diff --------------------------------------------------------------------

fn build_report(
    pcb_path: &Path,
    net_path: &Path,
    pcb_components: Vec<ComponentInfo>,
    net_components: Vec<NetComponent>,
    stale_warning: Option<String>,
) -> SyncReport {
    let pcb_by_ref: BTreeMap<String, ComponentInfo> = pcb_components
        .iter()
        .map(|c| (c.reference.clone(), c.clone()))
        .collect();
    let net_by_ref: BTreeMap<String, NetComponent> = net_components
        .iter()
        .map(|c| (c.reference.clone(), c.clone()))
        .collect();

    let pcb_refs: BTreeSet<&String> = pcb_by_ref.keys().collect();
    let net_refs: BTreeSet<&String> = net_by_ref.keys().collect();

    let missing_from_pcb: Vec<NetComponent> = net_refs
        .difference(&pcb_refs)
        .map(|r| net_by_ref[*r].clone())
        .collect();
    let extra_on_pcb: Vec<PcbOnlyEntry> = pcb_refs
        .difference(&net_refs)
        .map(|r| {
            let c = &pcb_by_ref[*r];
            PcbOnlyEntry {
                reference: c.reference.clone(),
                footprint: c.footprint.clone(),
                value: c.value.clone(),
            }
        })
        .collect();

    let mut value_mismatches = Vec::new();
    let mut footprint_mismatches = Vec::new();
    for r in pcb_refs.intersection(&net_refs) {
        let p = &pcb_by_ref[*r];
        let n = &net_by_ref[*r];
        if let Some(pv) = &p.value {
            if pv != &n.value {
                value_mismatches.push(FieldMismatch {
                    reference: (*r).clone(),
                    schematic: n.value.clone(),
                    pcb: pv.to_string(),
                });
            }
        }
        if !n.footprint.is_empty() && p.footprint != n.footprint {
            footprint_mismatches.push(FieldMismatch {
                reference: (*r).clone(),
                schematic: n.footprint.clone(),
                pcb: p.footprint.clone(),
            });
        }
    }

    SyncReport {
        pcb_path: pcb_path.display().to_string(),
        netlist_path: net_path.display().to_string(),
        pcb_component_count: pcb_components.len(),
        netlist_component_count: net_components.len(),
        stale_netlist_warning: stale_warning,
        missing_from_pcb,
        extra_on_pcb,
        value_mismatches,
        footprint_mismatches,
    }
}

// --- Freshness guard ---------------------------------------------------------

fn check_netlist_freshness(pcb_path: &Path, net_path: &Path) -> Option<String> {
    let net_mtime = mtime(net_path)?;

    // Look for a .kicad_sch sibling next to the netlist; if absent fall back
    // to the PCB. The point is: if the netlist is older than the source
    // schematic (or even the PCB), it can't reflect current truth.
    let sch_path = net_path.with_extension("kicad_sch");
    let candidates: Vec<&Path> = if sch_path.exists() {
        vec![sch_path.as_path(), pcb_path]
    } else {
        vec![pcb_path]
    };

    let stale_against = candidates.iter().find_map(|p| {
        let m = mtime(p)?;
        if m > net_mtime {
            Some(p.display().to_string())
        } else {
            None
        }
    });

    stale_against.map(|p| {
        format!(
            "Netlist is OLDER than {}. Re-export the netlist in Eeschema \
             (File -> Export -> Netlist -> KiCad) before trusting the diff.",
            p
        )
    })
}

fn mtime(p: &Path) -> Option<SystemTime> {
    fs::metadata(p).ok().and_then(|m| m.modified().ok())
}

// --- Reporting ---------------------------------------------------------------

fn print_report(r: &SyncReport) {
    println!("PCB     : {}", r.pcb_path);
    println!("Netlist : {}", r.netlist_path);
    println!(
        "Counts  : {} components on PCB | {} components in netlist",
        r.pcb_component_count, r.netlist_component_count
    );

    if let Some(w) = &r.stale_netlist_warning {
        println!();
        println!("WARNING: {}", w);
    }

    println!();
    println!("== In schematic, missing from PCB ({}) ==", r.missing_from_pcb.len());
    for c in &r.missing_from_pcb {
        let sheet = c.sheetname.as_deref().unwrap_or("?");
        println!(
            "  {:<8}  value={:<16}  footprint={:<48}  sheet={}",
            c.reference, c.value, c.footprint, sheet
        );
    }

    println!();
    println!("== On PCB, missing from schematic ({}) ==", r.extra_on_pcb.len());
    for c in &r.extra_on_pcb {
        let v = c.value.as_deref().unwrap_or("");
        println!(
            "  {:<8}  value={:<16}  footprint={}",
            c.reference, v, c.footprint
        );
    }

    println!();
    println!("== Value mismatches ({}) ==", r.value_mismatches.len());
    for m in &r.value_mismatches {
        println!(
            "  {:<8}  sch={:<20}  pcb={}",
            m.reference, m.schematic, m.pcb
        );
    }

    println!();
    println!("== Footprint mismatches ({}) ==", r.footprint_mismatches.len());
    for m in &r.footprint_mismatches {
        println!(
            "  {:<8}\n      sch={}\n      pcb={}",
            m.reference, m.schematic, m.pcb
        );
    }

    println!();
    let total = r.missing_from_pcb.len()
        + r.extra_on_pcb.len()
        + r.value_mismatches.len()
        + r.footprint_mismatches.len();
    if total == 0 {
        println!("OK: schematic and PCB are in sync.");
    } else {
        println!("FAIL: {} total discrepancies.", total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"(export (version "E")
  (components
    (comp (ref "C2")
      (value "10uF")
      (footprint "Atlantix_Components:C_1210_3225Metric")
      (property (name "Sheetname") (value "single_phase_buck"))
      (property (name "Sheetfile") (value "half_bridge_gan.kicad_sch")))
    (comp (ref "R1")
      (value "1k")
      (footprint "Resistor_SMD:R_0603_1608Metric")
      (property (name "Sheetname") (value "gate_driver"))
      (property (name "Sheetfile") (value "gate_driver.kicad_sch")))))
"#;

    #[test]
    fn parses_two_components() {
        let comps = parse_netlist(SAMPLE);
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].reference, "C2");
        assert_eq!(comps[0].value, "10uF");
        assert_eq!(
            comps[0].footprint,
            "Atlantix_Components:C_1210_3225Metric"
        );
        assert_eq!(comps[0].sheetname.as_deref(), Some("single_phase_buck"));
        assert_eq!(comps[1].reference, "R1");
        assert_eq!(comps[1].sheetname.as_deref(), Some("gate_driver"));
    }
}
