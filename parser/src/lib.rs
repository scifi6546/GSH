use image::RgbaImage;
use nalgebra::Vector2;
mod to_binary;
/// The parsed result.
#[derive(Debug, PartialEq)]
pub enum ParsedAST {
    String(String),
    Figure(Figure)
}
#[derive(Debug, PartialEq)]
pub struct Figure{
  
        dimensions: Vector2<u32>,
        contents: Vec<(FigureContents,Vector2<i32>)>
}
#[derive(Debug)]
pub enum ParseError {
    InvalidDatatype(u32),
    StringNotUTF8,
}
enum Datatypes {
    Text = 0x0,
}
#[derive(Debug, PartialEq)]
pub enum FigureContents{
    Image(RgbaImage),
    Line(Line),

}
#[derive(Debug, PartialEq)]
pub struct Line{
    pub color: u32,
    pub thickness:f32,
    pub segments: Vec<Vector2<i32>>,
}
pub struct Parser {
    buffer: Vec<u8>,
}

impl Parser {
    const HEADER_SIZE: usize = 8;
    ///generates new parser
    pub fn new() -> Parser {
        Parser { buffer: vec![] }
    }
    ///Takes in buffer if a payload is completed it is outputted in the parsed ast
    pub fn parse(&mut self, mut buffer: &mut Vec<u8>) -> Result<Vec<ParsedAST>, ParseError> {
        self.buffer.append(&mut buffer);

        let mut parsed = vec![];
        loop {
            if self.buffer.len() >= Self::HEADER_SIZE {
                let data_type = u32::from_le_bytes([
                    self.buffer[0],
                    self.buffer[1],
                    self.buffer[2],
                    self.buffer[3],
                ]);

                const TEXT_TYPE: u32 = Datatypes::Text as u32;
                let node = match data_type {
                    TEXT_TYPE => self.parse_text(),
                    _ => Some(Err(ParseError::InvalidDatatype(data_type))),
                };
                if let Some(node) = node {
                    if node.is_ok() {
                        parsed.push(node.ok().unwrap())
                    } else {
                        return Err(node.err().unwrap());
                    }
                } else {
                    return Ok(parsed);
                }
            } else {
                return Ok(parsed);
            }
        }
    }
    
    //parses contents of text
    fn parse_text(&mut self) -> Option<Result<ParsedAST, ParseError>> {
        let length = u32::from_le_bytes([
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ]) as usize;
        if length + Self::HEADER_SIZE > self.buffer.len() {
            None
        } else {
            let (data_and_header, remaining) = self.buffer.split_at_mut(length+Self::HEADER_SIZE);
            let (_,data) = data_and_header.split_at_mut(Self::HEADER_SIZE);
            let string_result = String::from_utf8(data.to_vec());
            if let Some(string) = string_result.ok() {
                self.buffer = remaining.to_vec();
                return Some(Ok(ParsedAST::String(string)));
            } else {
                return Some(Err(ParseError::StringNotUTF8));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty() {
        let mut p = Parser::new();
        assert_eq!(p.parse(&mut vec![]).ok().unwrap().len(), 0);
    }
    #[test]
    fn parse_text() {
        let mut p = Parser::new();
        let parsed_res = p.parse(&mut vec![0, 0, 0, 0, 1, 0, 0, 0, 'a' as u8]);
        if parsed_res.is_err() {
            panic!("{:?}", parsed_res.err().unwrap())
        }
        let parsed = parsed_res.ok().unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], ParsedAST::String("a".to_string()));
    }
    #[test]
    fn parse_text_partial() {
        let mut p = Parser::new();
        let parsed = p
            .parse(&mut vec![0, 0, 0, 0, 3, 0, 0, 0, 'a' as u8])
            .ok()
            .unwrap();
        assert_eq!(parsed.len(), 0);
    }
    #[test]
    fn parse_figure(){
        let mut p = Parser::new();
        let parsed = p.parse(&mut vec![1,0,0,0,
            8,0,0,0,
            5,0,0,0,
            5,0,0,0]).ok().unwrap();
        assert_eq!(parsed[0],ParsedAST::Figure(Figure{dimensions: Vector2::new(5,5),contents: vec![]}))
    }
    fn parse_figure_line(){
        let mut p = Parser::new();
        let figure_element_size = 4*4;
        let line_size = 4+4+2*(4+4);
        let parsed = p.parse(&mut vec![1,0,0,0,
            8+line_size+figure_element_size,0,0,0,
            5,0,0,0,
            5,0,0,0,
            //Element Type
            1,0,0,0,
            //Payload Length
            line_size,0,0,0,
            //x_start
            0,0,0,0,
            //y start,
            0,0,0,0,
            //line color
            0,0,0,1,
            //thickness
            0,0,0,0,
            //start cord
            0,0,0,0,    0,0,0,0,
            //end cord
            1,0,0,0,    1,0,0,0
            ]).ok().unwrap();
        assert_eq!(parsed[0],ParsedAST::Figure(Figure{dimensions: Vector2::new(5,5),contents: vec![]}))
    }
}
