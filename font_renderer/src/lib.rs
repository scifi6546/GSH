use font_kit::font::Font;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use nalgebra::Vector2;
use image::{RgbaImage,RgbImage};
use image::buffer::ConvertBuffer;
pub struct Renderer{
    font: Font,
}
impl Renderer{
    pub fn new()->Self{
        let font = SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
        Self{
            font
        }
    }
    pub fn write_to_image(&self,image: RgbaImage,data:&String,point_size:f32)->RgbaImage{
        let width = image.width();
        let height = image.height();
        let image2:RgbImage = image.convert();
        let mut canvas = Canvas{
            pixels: image2.into_vec(),
            size: Vector2I::new(width as i32,height as i32),
            stride:width as usize*3,
            format: Format::Rgb24,
        };
        self.write_to_canvas(&mut canvas, data,point_size);
        let img = RgbImage::from_vec(width, height, canvas.pixels).unwrap();
        let img2 = img.convert();
        return img2
    }
    fn write_to_canvas(&self,canvas: &mut Canvas,data: &String,point_size:f32){
        
        for (glyph,position) in self.get_string_position(&data,Vector2::new(canvas.size.x(),canvas.size.y()),point_size){
            self.font.rasterize_glyph(canvas, glyph, point_size, Transform2F::from_translation(Vector2F::new(
                position.x as f32,position.y as f32
            )), HintingOptions::None, RasterizationOptions::SubpixelAa).ok().unwrap();
        }
    }
    fn get_string_position(&self,data:&String,canvas_size:Vector2<i32>,point_size:f32)->Vec<(u32,Vector2<i32>)>{
        let mut current_pos:Vector2<i32> = Vector2::new(0,0);
        data.chars().map(|c|{
            if (current_pos+self.get_char_size(c,point_size)).x<canvas_size.x{
                current_pos.x+=self.get_char_size(c,point_size).x;
                let glyph = self.font.glyph_for_char(c).unwrap();
                (glyph,current_pos.clone())
            }else{
                current_pos.x=0;
                let s = self.get_char_size(c,point_size);
                current_pos+=s;
                let glyph = self.font.glyph_for_char(c).unwrap();
                (glyph,current_pos.clone())
            }
            
        }).collect()
        
    }
    fn get_char_size(&self,character:char,point_size:f32)->Vector2<i32>{
        let glyph_id = self.font.glyph_for_char(character).unwrap();
        let raster_bounds = self.font.raster_bounds(
            glyph_id,
            point_size,
            Transform2F::from_translation(Vector2F::new(0.0, 32.0)),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        ).unwrap();
        return Vector2::new(raster_bounds.width(),raster_bounds.height());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut canvas = Canvas::new(Vector2I::splat(32), Format::A8);
        let r = Renderer::new();
        r.write_to_canvas(&mut canvas, &String::from("Hello World"),12.0);
    }
}
