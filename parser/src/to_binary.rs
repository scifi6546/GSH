use super::{Figure,FigureContents,Line};
use image::RgbaImage;
use nalgebra::Vector2;
pub fn build_text(text:String)->Vec<u8>{
    return build_packet(0,&mut text.as_bytes().to_vec())
}
pub fn build_figure(figure: Figure)->Vec<u8>{
    unimplemented!()
}
fn build_packet(data_type:u32,data: &mut Vec<u8>)->Vec<u8>{
    let type_bytes = data_type.to_le_bytes();
    let length_bytes = (data.len() as u32).to_le_bytes();
    let mut bytes = vec![type_bytes[0],type_bytes[1],type_bytes[2],type_bytes[3],length_bytes[0],length_bytes[1],length_bytes[2],length_bytes[3]];
    bytes.append(data);
    return bytes;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text() {
        let t = build_text("hello world".to_string());
    }
    #[test]
    fn empty_fig(){
        let fig = Figure{
            dimensions: Vector2::new(200,200),
            contents: vec![],
        };
        build_figure(fig);
    }
    #[test]
    fn image(){
        let fig = Figure{
            dimensions: Vector2::new(200,200),
            contents: vec![(FigureContents::Image(RgbaImage::new(10,10)),Vector2::new(0,0))],
        };
        build_figure(fig);
    }
    #[test]
    fn lines(){
        let fig = Figure{
            dimensions: Vector2::new(200,200),
            contents: vec![(FigureContents::Line(Line{
                color:0x00_00_00_ff,
                segments: vec![
                    Vector2::new(0,0),
                    Vector2::new(1,1),
                ]
            }),Vector2::new(0,0))],
        };
        build_figure(fig);
    }
}