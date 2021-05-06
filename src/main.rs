use std::net::TcpStream;
use std::thread;
use std::thread::{JoinHandle};
use std::{error::Error, io};
use crate::util::{
    Event,
    Events,
    TabsState,
};
use io::Read;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Terminal,
};

mod util;
mod message;

struct App<'a> {
    tabs: TabsState<'a>,
}

const SERVER: &'static str = "192.168.33.10:6697";

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let tcp_thread: JoinHandle<std::result::Result<(), Box<std::io::Error>>> = thread::spawn(|| {
        // Start the TCP connection
        let mut stream = TcpStream::connect(SERVER)?;

        let mut buf = [0u8; 1024];


        loop {
            match stream.read(&mut buf) {
                Ok(length) => {
                    let data = &buf[0..length];
                    println!("data: {}", String::from_utf8_lossy(data));
                },
                Err(e) => {
                    return Err(Box::new(e))
                }
            }
        }
    });

    tcp_thread.join().ok().expect("Can't join the TCP thread");

    // App
    let mut app = App {
        tabs: TabsState::new(vec!["Tab0", "Tab1", "Tab2", "Tab3"]),
    };

    // Main UI loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .horizontal_margin(3)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let block = Block::default().style(Style::default().bg(Color::Rgb(21,21,21)).fg(Color::White));
            f.render_widget(block, size);
            let titles = app
                .tabs
                .titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(first, Style::default().fg(Color::White)),
                        Span::styled(rest, Style::default().fg(Color::Green)),
                    ])
                })
                .collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .select(app.tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                );
            f.render_widget(tabs, chunks[0]);
            let inner = match app.tabs.index {
                0 => Block::default().title("Inner 0").borders(Borders::ALL),
                1 => Block::default().title("Inner 1").borders(Borders::ALL),
                2 => Block::default().title("Inner 2").borders(Borders::ALL),
                3 => Block::default().title("Inner 3").borders(Borders::ALL),
                _ => unreachable!(),
            };
            f.render_widget(inner, chunks[1]);
        })?;

        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Alt(number) => app.tabs.at(number.to_digit(10).unwrap() as usize),
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                _ => {}
            }
        }
    }
    Ok(())
}