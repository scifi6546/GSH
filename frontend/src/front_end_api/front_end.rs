use super::{ModelId, TextureId};
use nalgebra::{Vector2, Vector3};
pub struct Engine {
    box_mesh: ModelId,
    texture: TextureId,
}
pub struct Texture {
    pub image: image::RgbaImage,
}

pub struct Model {
    pub mesh: Vec<(Vector3<f32>, Vector2<f32>)>,
    pub indicies: Vec<u32>,
}
#[derive(Debug)]
pub enum DrawCall {
    DrawModel {
        model: ModelId,
        texture: TextureId,
        position: Vector3<f32>,
    },
}
pub enum Event {}
//Trait to be used by terminal horizontal split, vertiocal split and regular terminal will
pub trait Scene {
    fn request_models(
        &self,
    ) -> (
        Vec<Model>,
        Vec<Texture>,
        Box<dyn Fn(Vec<ModelId>, Vec<TextureId>) -> Self>,
    );
    fn get_dimensions(&self) -> Vector3<f32>;
    fn get_position(&self) -> Vector3<f32>;
    fn get_draw_calls(&self) -> Vec<DrawCall>;
    fn process_event(&mut self, event: Event);
}
impl Engine {
    pub fn new() -> (
        Vec<Model>,
        Vec<Texture>,
        Box<dyn Fn(Vec<ModelId>, Vec<TextureId>) -> Engine>,
    ) {
        let model = Model {
            #[rustfmt::skip]
            mesh: vec![
                (Vector3::new(-1.0, 1.0, 0.0),Vector2::new(0.0, 1.0)),
                (Vector3::new( 1.0, 1.0, 0.0),Vector2::new(1.0, 1.0)),
                (Vector3::new( 1.0,-1.0, 0.0),Vector2::new(1.0, 0.0)),
                (Vector3::new(-1.0,-1.0, 0.0),Vector2::new(0.0, 0.0)),
            ],
            indicies: vec![3, 2, 1, 3, 1, 0],
        };
        (
            vec![model],
            vec![Texture {
                image: image::RgbaImage::new(100, 100),
            }],
            Box::new(|model, textures| Engine {
                box_mesh: model[0].clone(),
                texture: textures[0].clone(),
            }),
        )
    }
    pub fn get_draw_calls(&self) -> Vec<DrawCall> {
        //gets draw calls from sub scenes
        vec![DrawCall::DrawModel {
            model: self.box_mesh.clone(),
            texture: self.texture.clone(),
            position: Vector3::new(0.0, 0.0, 0.0),
        }]
    }
    pub fn process_event(&mut self, event: Vec<Event>) {
        
    }
}
