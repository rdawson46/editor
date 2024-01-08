mod app;
mod ui;
mod tui;
use crate::app::Editor;
use crate::tui::{Tui, Event};
use crate::ui::{ui, update};

use color_eyre::eyre::Result;
use crossterm::event::{
    EnableMouseCapture,
    DisableMouseCapture,
    KeyCode,
    self,
    KeyEventKind
};
use ratatui::Terminal;
use ratatui::prelude::{
    CrosstermBackend,
    Backend
};


async fn run() -> Result<()> {
    let mut tui = Tui::new()?.tick_rate(1.0).frame_rate(30.0);
    tui.enter()?; 

    let filename = std::env::args().nth(0).unwrap();
    let filename = std::path::Path::new(&filename);

    let mut editor = Editor::new(filename)?;

    loop {
        let event = tui.next().await?;

        if let Event::Render = event.clone() {
            tui.draw(|f| {
                ui(f, &editor);
            })?;
        }

        update(&mut editor, event);

        if editor.should_quit {
            break;
        }
    }

    tui.exit()?; 

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let result = run().await;

    result?;

    Ok(())
}
