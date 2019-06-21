use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use chrono::{Local, DateTime};

pub fn init(debug_mode: bool, rx_logger: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        match debug_mode {
            true => {
                let mut now: DateTime<Local> = Local::now();
                let mut logger = File::create(format!("log{}.txt", now.to_rfc3339())).unwrap();
                let mut output;
                loop {
                    output = rx_logger.recv().unwrap();
                    if output == "END" {
                        return;
                    }
                    now = Local::now();
                    println!("[{}] {}", now, output);
                    logger.write_all(format!("[{}] {}\n", now, output).as_bytes()).unwrap();
                }
            },

            false => {
                let mut output;
                loop {
                    output = rx_logger.recv().unwrap();
                    if output == "END" {
                        return;
                    }
                    println!("{}", output);
                }
            }
        }
    });
}
