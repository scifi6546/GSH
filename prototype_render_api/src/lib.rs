mod prototype;
enum Command{
    Text{
        pos: (f32,f32),
        text: String,
    },
    Line{
        start: (f32,f32),
        end: (f32,f32),
    }
}
pub struct RenderApi{

}
impl RenderApi{
    pub fn draw_boarder(&self,dims:WindowDimensions){}
    pub fn draw_text(&self,cords: (f32,f32),text:String){}
    pub fn draw_line(&self,start: (f32,f32),end: (f32,f32)){}
}
pub struct WindowDimensions{

}
trait Terminal{
    fn draw(&self,render_api:&mut RenderApi){
        render_api.draw_boarder(WindowDimensions{});
        self.getCommands().iter().map(|c|
        match c{
            Command::Text{
                pos,
                text
            }=>render_api.draw_text(pos.clone(), text.clone()),
            Command::Line{
                start,
                end
            }=>render_api.draw_line(start.clone(), end.clone()),
            
        }
        ).for_each(drop);
    }
    fn getCommands(&self)->Vec<Command>;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
