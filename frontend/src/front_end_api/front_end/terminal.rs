use super::{DrawCall, Event, Model, ModelId, Scene, SceneCtor, Texture, TextureId};
use image::Rgba;
use nalgebra::{Vector2, Vector3};
use font_renderer::Renderer;
pub struct Terminal {
    terminal_mesh: ModelId,
    texture: TextureId,
    input_buffer: String,
    font: Renderer,
}
impl Terminal {
    pub fn new() -> SceneCtor<Self> {
        #[rustfmt::skip]
        let model = Model {
            mesh: vec![
                
                (Vector3::new(-1.0,-1.0, 0.0), Vector2::new(0.0, 0.0)),
                (Vector3::new( 1.0,-1.0, 0.0), Vector2::new(1.0, 0.0)),
                (Vector3::new( 1.0, 1.0, 0.0), Vector2::new(1.0, 1.0)),
                (Vector3::new(-1.0,-1.0, 0.0), Vector2::new(0.0, 0.0)),
                (Vector3::new( 1.0, 1.0, 0.0), Vector2::new(1.0, 1.0)),
                (Vector3::new(-1.0, 1.0, 0.0), Vector2::new(0.0, 1.0)),
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
                input_buffer: "".to_string(),
                font: Renderer::new()
            }),
        )
    }
}
impl Scene for Terminal {
    fn get_draw_calls(&self) -> Vec<DrawCall> {
        //gets draw calls from sub scenes
        let texture = image::RgbaImage::from_pixel(
            1000,
            1000,
            Rgba([self.input_buffer.len() as u8, 0, 25, 255]),
        );
        
        let texture = self.font.write_to_image(texture, &self.input_buffer,12.0);
        vec![
            DrawCall::DrawModel {
                model: self.terminal_mesh.clone(),
                texture: self.texture.clone(),
                position: Vector3::new(0.0, 0.0, 0.0),
            },
            DrawCall::UpdateTexture {
                texture: self.texture.clone(),
                new_texture: texture,
            },
        ]
    }
    fn process_event(&mut self, event: Event) {
        match event {
            Event::RegularKey(c) => {
                println!("pressed: {}", c);
                self.input_buffer.push(c);
            }
            Event::SpecialKey(k) => println!("special key {:?}", k),
            _ => (),
        }
    }
}
