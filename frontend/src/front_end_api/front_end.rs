mod terminal;
pub use super::{ModelId, TextureId};
use nalgebra::{Vector2, Vector3};
pub use terminal::Terminal;
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
pub enum SpecialKey {
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
    #[allow(dead_code)]
    SpecialKey(SpecialKey),
    Unknown,
}
pub type SceneCtor<S> = (
    Vec<Model>,
    Vec<Texture>,
    Box<dyn Fn(Vec<ModelId>, Vec<TextureId>) -> S>,
);
//Trait to be used by terminal horizontal split, vertiocal split and regular terminal will
pub trait Scene {
    fn get_draw_calls(&self) -> Vec<DrawCall>;
    fn process_event(&mut self, event: Event);
}
