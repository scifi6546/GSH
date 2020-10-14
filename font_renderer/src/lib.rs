use font_kit::font::Font;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use nalgebra::Vector2;
pub struct Renderer{
    font: Font,
}
impl Renderer{
    pub fn new()->Self{
        let font = SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
        Self{
            font
        }
    }
    pub fn write_to_canvas(&self,canvas: &mut Canvas,data: String){
        unimplemented!();
    }
    fn get_string_position(&self,data:&String,canvas_size:Vector2<i32>)->Vec<(u32,Vector2<i32>)>{
        let mut current_pos:Vector2<i32> = Vector2::new(0,0);
        data.chars().map(|c|{
            current_pos+=self.get_char_size(c);
            let glyph = self.font.glyph_for_char(c).unwrap();
            (glyph,current_pos.clone())
        }).collect()
        
    }
    fn get_char_size(&self,character:char)->Vector2<i32>{
        let glyph_id = self.font.glyph_for_char(character).unwrap();
        let raster_bounds = self.font.raster_bounds(
            glyph_id,
            12.0,
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
        r.write_to_canvas(&mut canvas, String::from("Hello World"));
    }
}
