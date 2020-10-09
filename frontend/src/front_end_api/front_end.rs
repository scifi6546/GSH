use super::{ModelId, TextureId};
use nalgebra::{Vector2, Vector3};
use image::Rgba;
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
//any key that is not a letter
#[allow(dead_code)]
#[derive(Debug)]
pub enum SpecialKey{
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,
    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}
pub enum Event {
    RegularKey(char),
    SpecialKey(SpecialKey),
    Unknown
}
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
        let mesh =  vec![
            (Vector3::new(-1.0,-1.0, 0.0),Vector2::new(1.0, 1.0)),
            (Vector3::new( 1.0,-1.0, 0.0),Vector2::new(1.0, 0.0)),
            (Vector3::new( 1.0, 1.0, 0.0),Vector2::new(1.0, 1.0)),];
        unsafe{
            let m_ptr = mesh.as_ptr() as *const f32;

            for i in 0..3{
                print!("(");
                for j in 0..5{
                    print!("{}, ",*(m_ptr.offset(i*5+j)));
                }
                println!(")");
                
            }
        }
        let model = Model {
            #[rustfmt::skip]
            mesh,
                //(Vector3::new(-1.0, 1.0, 0.0),Vector2::new(0.0, 0.0)),
            
            
            indicies: vec![0,1,2,0,2,3],
        };
        (
            vec![model],
            vec![Texture {
                image: image::RgbaImage::from_pixel(100, 100, Rgba([0,0,0,255])),
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
    pub fn process_event(&mut self, event: Event) {
        match event{
            Event::RegularKey(c)=>println!("pressed: {}",c),
            Event::SpecialKey(k)=>println!("special key {:?}",k),
            _=>()
        }
    }
}
