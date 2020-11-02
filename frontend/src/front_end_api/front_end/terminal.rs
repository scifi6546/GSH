use super::{Event, Model, ModelId, Scene, SceneCtor, Texture, TextureId};
pub use super::DrawCall;
use image::Rgba;
use nalgebra::{Vector2, Vector3};
use font_renderer::Renderer;
use parser::{Deserializer,serializer,ParsedAST};
mod io;
mod render_surfaces;
use render_surfaces::{Font,TextRenderer,TextureRenderer,LineRenderer};
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
    Text(TextRenderer),
    Texture(TextureRenderer),
    Line(LineRenderer),
}
struct GlobalData{
    font: Font

}
impl DrawObject{
    pub fn from_ast(ast: &ParsedAST,global: GlobalData)->Self{
        match ast{
            ParsedAST::String(s) => DrawObject::Text(TextRenderer::new(global.font,s)),
            ParsedAST::Figure(_)=>todo!(),
        }
    }
    pub fn get_draw_calls(&mut self)->Vec<DrawCall>{
        todo!("get draw calls of sub objects")

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
                front_end: io::FrontEnd::new(io::Settings{command: "../test_app/target/release/test_app".to_string()}),
            }),
        )
    }
}
impl Terminal{
    /// reformats layout every frame. Will do things like merge text boxes
    fn process_layout(&self){
        // yes I know this looks bad but I need to check length every loop inorder to make
        // sure that the program does not walk off of the end of the array
    }

}
impl Scene for Terminal {
    fn get_draw_calls(&mut self) -> Vec<DrawCall> {
        self.front_end.send_input(serializer::build_text(self.input_buffer.chars().map(|c|c).collect()));

        if let Some(new_ast) = self.deseralizer.parse(&mut self.front_end.poll_output()).ok(){
            let mut ast = new_ast.iter().map(|a| (a.clone(),DrawObject::from_ast(a))).collect();
            self.rendering_buffer.append(&mut ast);

        }
        self.process_layout();
        let mut draw = vec![];
        for (_,draw_object) in self.rendering_buffer.iter_mut(){
            draw.append(&mut draw_object.get_draw_calls());

        }
        return draw
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
