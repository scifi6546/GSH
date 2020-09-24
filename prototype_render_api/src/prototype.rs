pub struct RenderEngine{
    scene: Box<dyn Scene>
}
impl RenderEngine{
}
pub enum DrawCall{
    TextBox{
        text: String,
        upper_right: (f32,f32),
        lower_right: (f32,f32)
    },
    TerminalBoarder{
        title: String,
        upper_right: (f32,f32),
        lower_right: (f32,f32),
    },
    Line{
        start: (f32,f32),
        end: (f32,f32),
    }
}
pub enum Event{
    Keyboard(char),
    Mouse(f32,f32),
}
//Trait to be used by terminal horizontal split, vertiocal split and regular terminal will 
pub trait Scene{
    fn get_dimensions(&self)->(f32,f32);
    fn get_position(&self)->(f32,f32);
    fn get_draw_calls(&self)->Vec<DrawCall>;
    fn process_event(&mut self,event: Event);
}
