use super::{DrawCall, Event, Model, ModelId, Scene, SceneCtor, Texture, TextureId};
use image::Rgba;
use nalgebra::{Vector2, Vector3};
use font_renderer::Renderer;
use parser::{Deserializer,serializer,ParsedAST};
mod io;
mod render_surfaces;
pub struct Terminal {
    terminal_mesh: ModelId,
    texture: TextureId,
    input_buffer: String,
    front_end: io::FrontEnd,
    deseralizer: Deserializer,
    rendering_buffer: Vec<(ParsedAST,DrawObject)>,
    font: Renderer,
}
enum DrawObject{
    Text,
}
impl DrawObject{
    pub fn from_ast(ast: &ParsedAST)->Self{
        match ast{
            ParsedAST::String(_) => todo!(),
            _ =>todo!()
        }

    }

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
                font: Renderer::new(),
                rendering_buffer: vec![],
                deseralizer: Deserializer::new(),
                front_end: io::FrontEnd::new(io::Settings{command: "foo".to_string()}),
            }),
        )
    }
}
impl Terminal{
    /// Returns output given input
    fn process_layout(&self){
        unimplemented!()
    }

}
impl Scene for Terminal {
    fn get_draw_calls(&mut self) -> Vec<DrawCall> {
        self.front_end.send_input(serializer::build_text(self.input_buffer.chars().map(|c|c).collect()));

        if let Some(mut new_ast) = self.deseralizer.parse(&mut self.front_end.poll_output()).ok(){
            let mut ast = new_ast.iter().map(|a| (a.clone(),DrawObject::from_ast(a))).collect();
            self.rendering_buffer.append(&mut ast);

        }
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
