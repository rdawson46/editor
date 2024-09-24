use std::io;

use crossterm::event::{
//    KeyCode,
//    self,
//    KeyEventKind,
    EnableMouseCapture,
    DisableMouseCapture,
    KeyEvent,
    MouseEvent
};
use color_eyre::eyre::Result;
use ratatui::Terminal;
use crossterm::{execute, ExecutableCommand};
use futures::{
    FutureExt,
    StreamExt
};
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc::unbounded_channel;
use tokio::{
    sync::mpsc,
    task::JoinHandle
};
use crossterm::terminal::{
    enable_raw_mode,
    EnterAlternateScreen,
    disable_raw_mode,
    LeaveAlternateScreen,
};

// combining the terminal and event
// TODO: add these events
#[warn(dead_code)]
#[derive(Clone, Debug)]
pub enum Event{
    Init,
    Quit,
    Error,
    Render,
    Closed,
    Tick,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<CrosstermBackend<std::io::Stderr>>,
    pub size: (u16, u16),
    pub task: Option<JoinHandle<()>>,
    pub event_rx: mpsc::UnboundedReceiver<Event>,
    pub event_tx: mpsc::UnboundedSender<Event>,
    pub tick_rate: f64,
    pub render_rate: f64,
}

impl Tui {
    // NOTE: creates new Tui
    pub fn new() -> Result<Tui> {
        let backend = CrosstermBackend::new(io::stderr());
        let terminal = Terminal::new(backend)?;

        let size = crossterm::terminal::size()?;
        let (tx, rx) = unbounded_channel::<Event>();

        let tick_rate: f64 = 0.0;
        let render_rate: f64 = 0.0;

        let task: Option<JoinHandle<()>> = None;

        Ok(Tui {
            terminal,
            size,
            task,
            event_rx: rx,
            event_tx: tx,
            tick_rate,
            render_rate
        })
    }

    
    // NOTE: kicks off tui usage
    pub fn start(&mut self) {
        // time to read events
        let tick_rate = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);

        // time to render screen
        let render_rate = std::time::Duration::from_secs_f64(1.0 / self.render_rate);
        let _event_tx = self.event_tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);
            let mut render_interval = tokio::time::interval(render_rate);
            
            loop{
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    crossterm::event::Event::Key(key) => {
                                        if key.kind == crossterm::event::KeyEventKind::Press{
                                            _event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    crossterm::event::Event::Resize(x,y) => {
                                        _event_tx.send(Event::Resize(x, y)).unwrap();
                                    },
                                    crossterm::event::Event::Mouse(mouse_event) => {
                                        _event_tx.send(Event::Mouse(mouse_event)).unwrap();
                                    },
                                    _ => {}
                                }
                            }
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            }
                            None => {},
                        }
                    },
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        });
        self.task = Some(task);
    }

    pub async fn next(&mut self) -> Result<Event> {
        let event = self.event_rx.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get event"));
        event
    }

    pub fn enter(&self) -> Result<()> {
        enable_raw_mode()?;
        execute!(std::io::stderr(), EnterAlternateScreen)?;
        std::io::stderr().execute(EnableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&self) -> Result<()> {
        std::io::stderr().execute(DisableMouseCapture)?;
        execute!(std::io::stderr(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn tick_rate(mut self, val: f64) -> Self{
        self.tick_rate = val;
        self
    }

    pub fn render_rate(mut self, val: f64) -> Self{
        self.render_rate = val;
        self
    }
}

