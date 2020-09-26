use nalgebra::{Vector2,Vector3};
mod front_end;
use front_end::{Engine,Event,DrawCall};
struct Context{
    front_end: Engine,
}
impl Context{
    fn new()->Self{
        unimplemented!()
        
    }
    fn render_loop(&mut self){
        loop{
            let e = self.get_events();
            self.front_end.process_event(e);
            let draw = self.front_end.get_draw_calls();
            self.draw(draw);
        }
    }
    fn get_events(&self)->Vec<Event>{
        unimplemented!()
    }
    fn draw(&mut self,draw_calls: Vec<DrawCall>){
        unimplemented!()
    }
}