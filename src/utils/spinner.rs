use std::io::Write;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use crate::utils::{get_unicode_string, Colors, Cursor};

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

    fn update_frame(frame: &str, message: String, dot: usize) -> String {
        let dots = ".".repeat(dot.min(3));
        format!("{} {} {}", Colors.magenta(frame), message, dots)
    }

    fn spin(&mut self, receiver: std::sync::mpsc::Receiver<()>) {
        print!("\x1B[?25l"); // hide cursor
        let circle = get_unicode_string("○", "o");
        let s_bar = get_unicode_string("│", "|");
        let bar = Colors.gray(s_bar);
        let circle = Colors.magenta(circle);
        let message = self.message.clone();
        print!("{bar}\n{circle}  {message}");
        let mut current_index = 0;
        let frames = &self.data;
        let mut dot = 0;
        loop {
            let frame = frames[current_index];
            let message = Self::update_frame(frame, self.message.clone(), dot);
            print!("\r{}\x1B[K", message); // clear line after printing message
            current_index = (current_index + 1) % frames.len();
            dot = (dot + 1) % (frames.len() * 4);
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
            if let Ok(_) = receiver.try_recv() {}
        }
    }
}
