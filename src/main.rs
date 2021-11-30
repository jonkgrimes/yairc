use io::Read;
use std::io::{stdin, Write};
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::{channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{error::Error, io};

use termion::{color, style};

mod client;
mod message;

use message::{Command, Message};

const DEFAUL_PORT: &'static str = "6697";

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Validate format of server
    let server_arg = std::env::args()
        .nth(1)
        .expect("Need to provide a host as the first argument. Example: irc.example.com");
    // TODO: Validate format of room
    let channel_name = std::env::args()
        .nth(2)
        .expect("Need to provide a room to join. Example: test_room");

    let nick = std::env::args()
        .nth(3)
        .expect("Need to provide a nick for the server. Example: somename");

    let (tx, rx) = channel();

    let sender = Arc::new(Mutex::new(tx));
    let receiver = Arc::new(Mutex::new(rx));

    let server = format!("{}:{}", server_arg, DEFAUL_PORT);

    let reader_thread: JoinHandle<std::result::Result<(), Box<std::io::Error>>> =
        thread::spawn(move || {
            let mut stream = TcpStream::connect(server)?;
            let mut buf = [0u8; 2048];

            let mut reply_messages: Vec<Message> = Vec::new();
            let mut need_to_register = true;
            let mut can_join = false;

            loop {
                println!("Reading from server");
                match stream.read(&mut buf) {
                    Ok(length) => {
                        let data = String::from_utf8_lossy(&buf[0..length]);
                        let messages: Vec<Result<Message, Box<dyn Error>>> = data.split("\r\n").map(|raw_message| Message::parse(raw_message)).collect();
                        for message in messages {
                            match message {
                                Ok(message) => match message.command() {
                                    Command::Ping => {
                                        let server = message.get_param(0).unwrap();
                                        reply_messages.push(Message::pong(server.clone()));
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data to UI thread");
                                    }
                                    Command::RplWelcome => {
                                        reply_messages.push(Message::motd());
                                        reply_messages.append(&mut client::join(&channel_name));
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data to UI thread");
                                    }
                                    _ => {
                                        sender
                                            .lock()
                                            .unwrap()
                                            .send(message)
                                            .expect("Unable to send data to UI thread");
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Unable to parse message: {}", data);
                                    eprintln!("Error: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => return Err(Box::new(e)),
                }

                if need_to_register {
                    println!("Registering...");
                    reply_messages.append(&mut client::register(&nick));
                    need_to_register = false;
                }

                if can_join {
                    println!("Joining...");
                    reply_messages.append(&mut client::join(&channel_name));
                }

                reply_messages
                    .iter()
                    .for_each(|message| match stream.write(&message.as_bytes()) {
                        Ok(0) => {
                            println!("Sent nothing, server connection might be closed")
                        }
                        Ok(n) => {
                            println!("Sent {} bytes", n)
                        }
                        Err(e) => {
                            eprintln!("Unable to send message: {}", e)
                        }
                    });
                reply_messages.clear();

                thread::sleep(Duration::from_secs(1));
            }
        });

    // Initiailize output
    let ui_thread = thread::spawn(move || {

        loop {
            let receiver = receiver.lock().unwrap();

            // Data from server TCP stream
            match receiver.recv() {
                Ok(message) => match message.command() {
                    Command::Notice => {
                        println!(
                            "{}{}{}",
                            color::Fg(color::Yellow),
                            message,
                            color::Fg(color::Reset)
                        );
                    }
                    Command::RplWelcome | Command::RplMyInfo | Command::RplYourHost | Command::RplCreated => {
                        println!(
                            "{}{}{}{}{}",
                            style::Bold,
                            color::Fg(color::LightBlue),
                            message,
                            color::Fg(color::Reset),
                            style::Reset
                        );
                    }
                    Command::MessageOfTheDay | Command::RplMotd | Command::RplMotdStart | Command::RplEndOfMotd => {
                        println!(
                            "{}{}{}",
                            style::Italic,
                            message,
                            style::Reset
                        );
                    }
                    Command::PrivMsg => {
                        dbg!(&message);
                        let name = match message.source() {
                            Some(name) => name.to_string(),
                            None => "Unknown".to_string(),
                        };
                        let message = message.get_param(1).unwrap();

                        println!(
                            "{}{}<{}>{}:{} {}",
                            style::Bold,
                            color::Fg(color::Green),
                            name,
                            color::Fg(color::Reset),
                            style::Reset,
                            message
                        );
                    }
                    _ => {
                        println!("{}", message);
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }

        }
    });

    let input_thread = thread::spawn(|| {
        let stdin = stdin();
        let stdin = stdin.lock();

        let mut bytes = stdin.bytes();
        loop {
            let b = bytes.next().unwrap().unwrap();
            println!(
                "{}{}<{}>{}:{} {}",
                style::Bold,
                color::Fg(color::White),
                "User",
                color::Fg(color::Reset),
                style::Reset,
                b
            );
        }
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

    Ok(())
}
