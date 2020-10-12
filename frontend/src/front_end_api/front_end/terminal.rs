use super::{DrawCall, Event, Model, ModelId, Scene, SceneCtor, Texture, TextureId};
use image::Rgba;
use font_kit::source::SystemSource;
use nalgebra::{Vector2, Vector3};
pub struct Terminal {
    terminal_mesh: ModelId,
    texture: TextureId,
    input_buffer: Vec<char>,
}
impl Terminal {
    pub fn new() -> SceneCtor<Self> {
        let model = Model {
            mesh: vec![
                (Vector3::new(-1.0, 1.0, 0.0), Vector2::new(0.0, 0.0)),
                (Vector3::new(1.0, 1.0, 0.0), Vector2::new(1.0, 0.0)),
                (Vector3::new(1.0, -1.0, 0.0), Vector2::new(1.0, 0.0)),
                (Vector3::new(-1.0, 1.0, 0.0), Vector2::new(0.0, 0.0)),
                (Vector3::new(1.0, -1.0, 0.0), Vector2::new(1.0, 0.0)),
                (Vector3::new(-1.0, -1.0, 0.0), Vector2::new(0.0, 1.0)),
            ],
            indicies: vec![],
        };
        let image = image::RgbaImage::from_pixel(100, 100, Rgba([0, 0, 25, 255]));
        (
            vec![model],
            vec![Texture { image }],
            Box::new(|model, textures| Terminal {
                terminal_mesh: model[0].clone(),
                texture: textures[0].clone(),
                input_buffer: vec![],
            }),
        )
    }
}
impl Scene for Terminal {
    fn get_draw_calls(&self) -> Vec<DrawCall> {
        //gets draw calls from sub scenes
        vec![
            DrawCall::DrawModel {
                model: self.terminal_mesh.clone(),
                texture: self.texture.clone(),
                position: Vector3::new(0.0, 0.0, 0.0),
            },
            DrawCall::UpdateTexture {
                texture: self.texture.clone(),
                new_texture: image::RgbaImage::from_pixel(
                    100,
                    100,
                    Rgba([self.input_buffer.len() as u8, 0, 25, 255]),
                ),
            },
        ]
    }
    fn process_event(&mut self, event: Event) {
        match event {
            Event::RegularKey(c) => {
                println!("pressed: {}", c);
                self.input_buffer.push(c)
            }
            Event::SpecialKey(k) => println!("special key {:?}", k),
            _ => (),
        }
    }
}
