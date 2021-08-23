use io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::{error::Error, io};
use std::time::Duration;

mod message;
mod client;

use message::{Command, Message};

const DEFAUL_PORT: &'static str = "6697";

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Validate format of server
    let server_arg = std::env::args().nth(1).expect("Need to provide a host as the first argument. Example: irc.example.com");
    // TODO: Validate format of room
    let room_arg = std::env::args().nth(2).expect("Need to provide a room to join. Example: #test_room");

    let (tx, rx) = channel();

    let sender = Arc::new(Mutex::new(tx));

    let server = format!("{}:{}", server_arg, DEFAUL_PORT);

    let reader_thread: JoinHandle<std::result::Result<(), Box<std::io::Error>>> =
        thread::spawn(move || {
            let mut stream = TcpStream::connect(server)?;
            let mut buf = [0u8; 1024];

            let mut messages: Vec<Message> = Vec::new();

            loop {
                println!("Waiting for data");
                match stream.read(&mut buf) {
                    Ok(length) => {
                        let data = String::from_utf8_lossy(&buf[0..length]);
                        let message = Message::parse(&data);
                        match message {
                            Ok(message) => {
                                match message.command() {
                                    Command::Ping => {
                                        messages.push(Message::ping());
                                    },
                                    _ => {
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data");
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Unable to parse message: {}", data);
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => return Err(Box::new(e)),
                }

                messages.iter().for_each(|message| {
                    match stream.write(message) {
                        Ok()
                    }
                });
                thread::sleep(Duration::from_secs(1))
            }
        });

    // UI loop
    let ui_thread = thread::spawn(move || loop {
        match rx.recv() {
            Ok(message) => {
                println!("{:?}", message)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        thread::sleep(Duration::from_secs(1))
    });

    match reader_thread.join() {
        Ok(result) => match result {
            Ok(_) => {
                println!("Reader thread exited without incident")
            }
            Err(e) => {
                eprintln!("Reader thread exited due to error: {}", e)
            }
        },
        Err(e) => {
            eprintln!("IRC listener thread unable to start")
        }
    }

    ui_thread.join().expect("UI Thread unable to be started");

    Ok(())
}