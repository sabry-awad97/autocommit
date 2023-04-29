use std::io::Write;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use crate::utils::{get_unicode_string, Colors};

#[derive(Clone)]
pub struct Spinner {
    data: Vec<&'static str>,
    sender: Option<Sender<()>>,
    message: String,
}

impl Spinner {
    pub fn new<T: Into<String>>(message: T) -> Self {
        let mut message = message.into();
        while message.ends_with('.') {
            message.pop();
        }
        Self {
            data: vec!["◒", "◐", "◓", "◑"],
            sender: None,
            message: message.into(),
        }
    }

    pub fn start(&mut self) {
        let (sender, receiver) = channel();
        self.sender = Some(sender);
        let mut spinner = self.clone();
        thread::spawn(move || {
            spinner.spin(receiver);
        });
    }

    pub fn stop(&mut self) {
        if let Some(sender) = &self.sender {
            sender.send(()).unwrap()
        }
    }

    fn spin(&mut self, receiver: std::sync::mpsc::Receiver<()>) {
        print!("\x1B[?25l"); // hide cursor
        let s_bar = get_unicode_string("│", "|");
        let bar = Colors.gray(s_bar);
        println!("{}", bar);
        let mut frame_idx = 0;
        let frames = &self.data;
        let mut dot = 0;
        loop {
            let frame = frames[frame_idx];
            let message = format!("{}  {} {}", Colors.magenta(frame), self.message, ".".repeat(dot.min(3)));
            print!("\r{}\x1B[K", message);
            frame_idx = (frame_idx + 1) % frames.len();
            dot = (dot + 1) % (frames.len() * 4);
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
            if let Ok(_) = receiver.try_recv() {}
        }
    }
}
