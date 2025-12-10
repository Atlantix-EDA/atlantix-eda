use serde::{Deserialize, Serialize};

/// A point in 2D space
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A KiCad symbol definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub description: String,
}

/// Font properties for text elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub size: Point,
    pub thickness: Option<f64>,
    pub bold: bool,
    pub italic: bool,
}

/// Text effects including font and styling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effects {
    pub font: Font,
    pub justify: Option<String>,
    pub hide: bool,
}

/// Color representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Stroke properties for drawing elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub width: f64,
    pub stroke_type: String,
    pub color: Option<Color>,
}

/// Fill properties for drawing elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub fill_type: String,
    pub color: Option<Color>,
}