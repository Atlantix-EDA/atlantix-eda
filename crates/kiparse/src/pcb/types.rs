//! Types for PCB files
//! 
//! This module defines the data structures used to represent PCB files in KiCad format.
//! It includes structures for points, rectangles, arcs, layers, footprints, tracks, vias, zones, texts, and graphics.
//! The structures are designed to be serializable and deserializable using Serde.
//! The `PcbFile` structure serves as the main entry point for parsing and manipulating PCB files.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point,
    pub start_angle: f64,
    pub end_angle: f64,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: i32,
    pub name: String,
    pub layer_type: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PcbFile {
    pub version: String,
    pub generator: String,
    pub board_thickness: Option<f64>,
    pub paper_size: Option<String>,
    pub layers: HashMap<i32, Layer>,
    pub footprints: Vec<Footprint>,
    pub tracks: Vec<Track>,
    pub vias: Vec<Via>,
    pub zones: Vec<Zone>,
    pub texts: Vec<Text>,
    pub graphics: Vec<Graphic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Footprint {
    pub name: String,
    pub uuid: String,
    pub position: Point,
    pub rotation: f64,
    pub layer: String,
    pub locked: bool,
    pub placed: bool,
    pub properties: HashMap<String, String>,
    pub pads: Vec<Pad>,
    pub graphics: Vec<Graphic>,
    pub texts: Vec<Text>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pad {
    pub number: String,
    pub pad_type: String,
    pub shape: String,
    pub position: Point,
    pub size: Point,
    pub drill: Option<f64>,
    pub layers: Vec<String>,
    pub net: Option<String>,
    pub roundrect_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub start: Point,
    pub end: Point,
    pub width: f64,
    pub layer: String,
    pub net: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Via {
    pub position: Point,
    pub size: f64,
    pub drill: f64,
    pub layers: Vec<String>,
    pub net: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub net: Option<String>,
    pub layer: String,
    pub priority: i32,
    pub connect_pads: bool,
    pub polygon: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
    pub position: Point,
    pub layer: String,
    pub effects: TextEffects,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextEffects {
    pub font_size: Point,
    pub thickness: f64,
    pub bold: bool,
    pub italic: bool,
    pub justify: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Graphic {
    Line {
        start: Point,
        end: Point,
        layer: String,
        width: f64,
    },
    Circle {
        center: Point,
        radius: f64,
        layer: String,
        width: f64,
        filled: bool,
    },
    Arc {
        arc: Arc,
        layer: String,
        width: f64,
    },
    Rectangle {
        rect: Rect,
        layer: String,
        width: f64,
        filled: bool,
    },
    Polygon {
        points: Vec<Point>,
        layer: String,
        width: f64,
        filled: bool,
    },
}

impl PcbFile {
    pub fn new() -> Self {
        Self {
            version: String::new(),
            generator: String::new(),
            board_thickness: None,
            paper_size: None,
            layers: HashMap::new(),
            footprints: Vec::new(),
            tracks: Vec::new(),
            vias: Vec::new(),
            zones: Vec::new(),
            texts: Vec::new(),
            graphics: Vec::new(),
        }
    }

    pub fn get_footprints_on_layer(&self, layer_name: &str) -> Vec<&Footprint> {
        self.footprints
            .iter()
            .filter(|f| f.layer == layer_name)
            .collect()
    }

    pub fn get_tracks_on_layer(&self, layer_name: &str) -> Vec<&Track> {
        self.tracks
            .iter()
            .filter(|t| t.layer == layer_name)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub pin_names_offset: f64,
    pub in_bom: bool,
    pub on_board: bool,
    pub properties: Vec<Property>,
    pub pins: Vec<Pin>,
    pub rectangles: Vec<Rectangle>,
    pub circles: Vec<Circle>,
    pub arcs: Vec<SymbolArc>,
    pub polylines: Vec<Polyline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub id: i32,
    pub at: Point,
    pub effects: Option<Effects>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effects {
    pub font: Font,
    pub justify: Option<String>,
    pub hide: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub size: Point,
    pub thickness: Option<f64>,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pin {
    pub number: String,
    pub name: String,
    pub pin_type: String,
    pub at: Point,
    pub length: f64,
    pub rotation: f64,
    pub name_effects: Option<Effects>,
    pub number_effects: Option<Effects>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    pub start: Point,
    pub end: Point,
    pub stroke: Stroke,
    pub fill: Fill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
    pub stroke: Stroke,
    pub fill: Fill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolArc {
    pub start: Point,
    pub mid: Point,
    pub end: Point,
    pub stroke: Stroke,
    pub fill: Fill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    pub points: Vec<Point>,
    pub stroke: Stroke,
    pub fill: Fill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub width: f64,
    pub stroke_type: String,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub fill_type: String,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}