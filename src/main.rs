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
    let channel_name = std::env::args().nth(2).expect("Need to provide a room to join. Example: #test_room");

    let (tx, rx) = channel();

    let sender = Arc::new(Mutex::new(tx));

    let server = format!("{}:{}", server_arg, DEFAUL_PORT);

    let reader_thread: JoinHandle<std::result::Result<(), Box<std::io::Error>>> =
        thread::spawn(move || {
            let mut stream = TcpStream::connect(server)?;
            let mut buf = [0u8; 1024];

            let mut messages: Vec<Message> = Vec::new();
            let mut need_to_register = true;
            let mut can_join = false;

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
                                        let server = message.get_param(0).unwrap();
                                        messages.push(Message::pong(server.clone()));
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data");
                                    },
                                    Command::RplWelcome => {
                                        messages.append(&mut client::join(&channel_name));
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data");
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

                if need_to_register {
                    messages.append(&mut client::register());
                    need_to_register = false;
                }

                if can_join {
                    messages.append(&mut client::join(&channel_name));
                }

                messages.iter().for_each(|message| {
                    match stream.write(&message.as_bytes()) {
                        Ok(0) => {
                            println!("Sent nothing, server connection might be closed")
                        },
                        Ok(n) => {
                            println!("Sent {} bytes", n)
                        },
                        Err (e) => {
                            eprintln!("Unable to send message: {}", e);
                        }
                    }
                });
                messages.clear();

                thread::sleep(Duration::from_secs(1))
            }
        });

    // UI loop
    let ui_thread = thread::spawn(move || loop {
        let mut input_buffer = String::new();

        match io::stdin().read_to_string(&mut input_buffer) {
            Ok(length) => {
                println!("Read {} bytes from input", length);
            },
            Err(e) => {
                eprintln!("An error occurred reading input: {}", e)
            }
        }

        // Data from server TCP stream
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