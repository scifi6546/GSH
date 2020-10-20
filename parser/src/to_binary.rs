use super::{Figure,FigureContents,Line};
use image::RgbaImage;
use nalgebra::Vector2;
pub fn build_text(text:String)->Vec<u8>{
    return build_packet(0,&mut text.as_bytes().to_vec())
}
pub fn build_figure(figure: Figure)->Vec<u8>{
    let x_dim = figure.dimensions.x.to_le_bytes();
    let y_dim = figure.dimensions.y.to_le_bytes();
    let mut figure_contents = vec![x_dim[0],x_dim[1],x_dim[2],x_dim[3],y_dim[0],y_dim[1],y_dim[2],y_dim[3]];
    for (content,position) in figure.contents.iter(){
        figure_contents.append(&mut build_figure_content(content,position));
    }
    return build_packet(0x1, &mut figure_contents)
}
fn build_figure_content(content:&FigureContents,position: &Vector2<i32>)->Vec<u8>{
    let (element_type,mut payload) = match content{
        FigureContents::Image(image)=>(0u32,image.as_raw().clone()),
        FigureContents::Line(line)=>(1u32,build_line(line)),
    };
    let x_chord = position.x.to_le_bytes();
    let y_chord = position.y.to_le_bytes();
    let elm_type_bytes = element_type.to_le_bytes();
    let length_bytes = (payload.len() as u32).to_le_bytes();
    let mut data = vec![
        elm_type_bytes[0],elm_type_bytes[1],elm_type_bytes[2],elm_type_bytes[3],
        length_bytes[0],length_bytes[1],length_bytes[2],length_bytes[3],
        x_chord[0],x_chord[1],x_chord[2],x_chord[3],y_chord[0],y_chord[1],y_chord[2],y_chord[3],
    ];
    data.append(&mut payload);
    return data;
}
fn build_line(line: &Line)->Vec<u8>{
    let thickness_bytes = line.thickness.to_le_bytes();
    let color_bytes = line.color.to_le_bytes();
    let mut data = vec![
        color_bytes[0],color_bytes[1],color_bytes[2],color_bytes[3],
        thickness_bytes[0],thickness_bytes[1],thickness_bytes[2],thickness_bytes[3],
    ];
    for segment in line.segments.iter(){
        unimplemented!()
    } 
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