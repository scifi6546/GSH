use std::io::prelude::*;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
pub struct FrontEnd {
    stdin: ChildStdin,
    stdout: Receiver<u8>,
}
pub struct ProcessStdout {
    stdout: ChildStdout,
    stdout_return: Sender<u8>,
}
impl ProcessStdout {
    pub fn new(stdout: ChildStdout, stdout_return: Sender<u8>) -> Self {
        Self {
            stdout,
            stdout_return,
        }
    }
    /// Starts the operation of second thread
    pub fn start_loop(&mut self) {
        let mut buffer: [u8; 1] = [0];
        loop {
            if let Some(_) = self.stdout.read_exact(&mut buffer).err() {
                break;
            }
            if let Some(_) = self.stdout_return.send(buffer[0]).err() {
                break;
            }
        }
    }
}
impl FrontEnd {
    pub fn new(settings: Settings) -> Self {
        let child = Command::new(settings.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("??");
        let (sender, stdout) = channel();
        let mut other_thread = ProcessStdout::new(child.stdout.unwrap(), sender);
        thread::spawn(move || {
            other_thread.start_loop();
        });

        FrontEnd {
            stdout: stdout,
            stdin: child.stdin.unwrap(),
        }
    }
    pub fn send_input(&mut self, input: Vec<u8>) {
        self.stdin.write(&input);
    }
    pub fn poll_output(&self) -> Vec<u8> {
        let mut output = vec![];
        if let Some(byte) = self.stdout.try_recv().ok() {
            output.push(byte);
        }
        return output;
    }
}
pub struct Settings {
    pub command: String,
}
