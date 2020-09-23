use std::sync::mpsc::{channel,Sender,Receiver};
pub trait Split{
    fn get_children(&self)->Vec<Receiver<String>>;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
