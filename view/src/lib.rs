use std::sync::mpsc::{channel, Receiver, Sender};
pub trait Split {
    fn get_children(&self) -> Vec<Receiver<String>>;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
