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
    // TODO: fix new function
    let mut tui = Tui::new()?.tick_rate(1.0).frame_rate(30.0);
    tui.enter()?; 

    tui.start();

    let filename = std::env::args().nth(0).unwrap();
    let filename = std::path::Path::new(&filename);

    let mut editor = Editor::new(filename)?;

    loop {
        let event = tui.next().await?;

        if let Event::Render = event.clone() {
            tui.terminal.draw(|f| {
                // sets up ui, who knows where it will go
                ui(f, &editor);
            })?;
        }

        // use event to update the editor
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
