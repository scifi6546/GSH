use super::super::{DrawCall, ModelId, TextureId};
pub trait RenderSUrface {}
pub struct Font {}
pub struct TextRenderer {}
impl TextRenderer {
    pub fn new(font: &Font, string: String) -> Self {
        unimplemented!()
    }
    pub fn add_text(&self, data: String) {
        unimplemented!()
    }
    pub fn get_draw_call(&self) -> DrawCall {
        unimplemented!()
    }
}
pub struct TextureRenderer {}
pub struct LineRenderer {}
