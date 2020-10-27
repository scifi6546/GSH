use image::RgbaImage;
use nalgebra::Vector2;
mod deserializer;
pub use deserializer::Deserializer;
pub mod serializer;
/// The parsed result.
#[derive(Debug, PartialEq, Clone)]
pub enum ParsedAST {
    String(String),
    Figure(Figure),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Figure {
    dimensions: Vector2<u32>,
    contents: Vec<FigureContents>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    InvalidDatatype(u32),
    InvalidImage,
    InvalidLine,
    StringNotUTF8,
}
enum Datatypes {
    Text = 0x0,
    Figure = 0x1,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FigureContents {
    data: FigureContentsData,
    position: Vector2<i32>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum FigureContentsData {
    Image(RgbaImage),
    Line(Line),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    pub color: u32,
    pub thickness: f32,
    pub segments: Vec<Vector2<f32>>,
}
