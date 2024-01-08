use color_eyre::eyre::Result;
use crossterm::execute;
use crossterm::event::{KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use tokio::{sync::mpsc, task::JoinHandle};
use crossterm::terminal::{
    enable_raw_mode,
    EnterAlternateScreen,
    disable_raw_mode,
    LeaveAlternateScreen
};

// combining the terminal and event
#[derive(Clone, Debug)]
pub enum Event{
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<CrosstermBackend<std::io::Stderr>>,
    pub task: Option<JoinHandle<()>>,
    pub event_rx: mpsc::UnboundedReceiver<Event>,
    pub event_tx: mpsc::UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

impl Tui {
    
    // TODO: yes
    pub fn new() -> Result<()> {
        Ok(())
    }

    pub fn start(&mut self) {
        let tick_rate = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let redner_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        let _event_tx = self.event_tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);
            let mut render_interval = tokio::time::interval(redner_delay);
            
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
                                    _ => {},
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
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.event_rx.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }

    pub async fn enter(&self) -> Result<()> {
        enable_raw_mode()?;
        execute!(std::io::stderr(), EnterAlternateScreen)?;
        Ok(())
    }

    pub async fn exit(&self) -> Result<()> {
        execute!(std::io::stderr(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn frame_rate(&mut self, val: f64){
        self.frame_rate = val;
    }

    pub fn tick_rate(&mut self, val: f64){
        self.tick_rate = val;
    }
}
