use parser::serializer;
use std::io::{self, Write};
fn main() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    #[allow(unused_must_use)]
    handle.write_all(&serializer::build_text("Hello World!".to_string()));
}
