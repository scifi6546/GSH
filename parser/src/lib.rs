use image::RgbaImage;
use nalgebra::Vector2;
mod to_binary;
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
pub struct Parser {
    buffer: Vec<u8>,
}

impl Parser {
    const HEADER_SIZE: usize = 8;
    const FIGURE_HEADER_SIZW: usize = 8;
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
                const FIGURE_TYPE: u32 = Datatypes::Figure as u32;
                let node = match data_type {
                    TEXT_TYPE => self.parse_text(),
                    FIGURE_TYPE => self.parse_figure(),
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
            let (data_and_header, remaining) = self.buffer.split_at_mut(length + Self::HEADER_SIZE);
            let (_, data) = data_and_header.split_at_mut(Self::HEADER_SIZE);
            let string_result = String::from_utf8(data.to_vec());
            if let Some(string) = string_result.ok() {
                self.buffer = remaining.to_vec();
                return Some(Ok(ParsedAST::String(string)));
            } else {
                return Some(Err(ParseError::StringNotUTF8));
            }
        }
    }
    /// Parses figure
    fn parse_figure(&mut self) -> Option<Result<ParsedAST, ParseError>> {
        if self.is_complete() != true {
            return None;
        }
        let packet_length = u32::from_le_bytes([
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ]) as usize;
        let dimensions = Vector2::new(
            u32::from_le_bytes([
                self.buffer[8],
                self.buffer[9],
                self.buffer[10],
                self.buffer[11],
            ]),
            u32::from_le_bytes([
                self.buffer[12],
                self.buffer[13],
                self.buffer[14],
                self.buffer[15],
            ]),
        );

        //figure-buffer
        let figure_contents =
            self.buffer[8 + 8..8 + 8 + packet_length - Self::FIGURE_HEADER_SIZW].to_vec();
        let mut i = 0;
        let mut figure_data = vec![];
        loop {
            if i >= figure_contents.len() {
                break;
            }
            let size = u32::from_le_bytes([
                figure_contents[i + 4],
                figure_contents[i + 5],
                figure_contents[i + 6],
                figure_contents[i + 7],
            ]);
            figure_data.push(figure_contents[i..i + size as usize + 16].to_vec());
            i += size as usize + 16;
        }
        let figures: Vec<Result<FigureContents, ParseError>> = figure_data
            .iter()
            .map(|data| {
                let data_type = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                match data_type {
                    0 => Self::parse_picture_element(data),
                    1 => Self::parse_line_element(data),
                    _ => Err(ParseError::InvalidDatatype(3)),
                }
            })
            .collect();
        let mut data = vec![];

        for fig in figures.iter() {
            if fig.is_err() {
                return Some(Err(fig.clone().err().unwrap()));
            }
            if let Some(f) = fig.clone().ok() {
                data.push(f);
            }
        }
        self.buffer = self.buffer[packet_length + Self::HEADER_SIZE..self.buffer.len()].to_vec();
        let figure = Figure {
            contents: data,
            dimensions,
        };
        return Some(Ok(ParsedAST::Figure(figure)));
    }
    fn parse_picture_element(data: &Vec<u8>) -> Result<FigureContents, ParseError> {
        if data.len() < 6*4 {
            return Err(ParseError::InvalidImage);
        }
        let position = Vector2::new(
            i32::from_le_bytes([data[8+0], data[8+1], data[8+2], data[8+3]]),
            i32::from_le_bytes([data[8+4], data[8+5], data[8+6], data[8+7]]),
        );

        let dimensions = Vector2::new(
            u32::from_le_bytes([data[16+0], data[16+1], data[16+2], data[16+3]]),
            u32::from_le_bytes([data[16+4], data[16+5], data[16+6], data[16+7]]),
        );
        if let Some(image) = RgbaImage::from_raw(dimensions.x,dimensions.y,data[8..data.len()].to_vec()){
            return Ok(FigureContents{
                data:FigureContentsData::Image(image),
                position
            })

        }
        return Err(ParseError::InvalidImage)

    }
    fn parse_line_element(data: &Vec<u8>) -> Result<FigureContents, ParseError> {
        if data.len()<6*4{
            return Err(ParseError::InvalidLine);
        }
        let position = Vector2::new(
            i32::from_le_bytes([data[8+0], data[8+1], data[8+2], data[8+3]]),
            i32::from_le_bytes([data[8+4], data[8+5], data[8+6], data[8+7]]),
        );
        let color = u32::from_le_bytes([data[16+0],data[16+1],data[16+2],data[16+3]]);
        let thickness = f32::from_le_bytes([data[16+4],data[16+1],data[16+2],data[16+3]]);
        let mut segments = vec![];
        for i in 0..data.len()/4-(4+2){
            let index = i*4;
            segments.push(Vector2::new(f32::from_le_bytes([data[index+0],data[index+1],data[index+2],data[index+3]]),
            f32::from_le_bytes([data[index+4],data[index+5],data[index+6],data[index+7]])));


        }
        return Ok(FigureContents{
            data:FigureContentsData::Line(Line{
                color,
                thickness,
                segments,
            }),

            position,

        })
    }
    /// Gets if current packet is complete
    fn is_complete(&self) -> bool {
        if self.buffer.len() < 8 {
            return false;
        }
        let data_size = u32::from_le_bytes([
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ]);
        if self.buffer.len() >= data_size as usize + 8 {
            return true;
        } else {
            return false;
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
    fn parse_figure() {
        let mut p = Parser::new();
        #[rustfmt::skip]
        let parsed_res = p.parse(&mut vec![1, 0, 0, 0, 
            8, 0, 0, 0, 
            5, 0, 0, 0,
            5, 0, 0, 0]);
        if parsed_res.is_err() {
            panic!("{:?}", parsed_res.err().unwrap());
        }
        assert_eq!(
            parsed_res.ok().unwrap()[0],
            ParsedAST::Figure(Figure {
                dimensions: Vector2::new(5, 5),
                contents: vec![]
            })
        )
    }
    #[test]
    fn parse_figure_line() {
        let mut p = Parser::new();
        let figure_element_size = 4 * 4;
        let line_size = 4 + 4 + 2 * (4 + 4);
        let parsed_res = p.parse(&mut vec![
            1,
            0,
            0,
            0,
            8 + line_size + figure_element_size,
            0,
            0,
            0,
            5,
            0,
            0,
            0,
            5,
            0,
            0,
            0,
            //Element Type
            1,
            0,
            0,
            0,
            //Payload Length
            line_size,
            0,
            0,
            0,
            //x_start
            0,
            0,
            0,
            0,
            //y start,
            0,
            0,
            0,
            0,
            //line color
            0,
            0,
            0,
            1,
            //thickness
            0,
            0,
            0,
            0,
            //start cord
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            //end cord
            1,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
        ]);
        if parsed_res.is_err() {
            panic!("{:?}", parsed_res.err().unwrap());
        }
        assert_eq!(
            parsed_res.ok().unwrap()[0],
            ParsedAST::Figure(Figure {
                dimensions: Vector2::new(5, 5),
                contents: vec![]
            })
        )
    }
}
