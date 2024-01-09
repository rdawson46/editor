mod app;
mod ui;
mod tui;
use crate::app::Editor;
use crate::tui::{Tui, Event};
use crate::ui::{ui, update};

use color_eyre::eyre::Result;


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    if filename.is_none() {
        return Ok(());
    }

    let filename = filename.unwrap();
    let filename = std::path::Path::new(&filename);

    let mut tui = Tui::new()?.tick_rate(1.0).frame_rate(30.0);
    tui.enter()?; 

    tui.start();


    let mut editor = Editor::new(filename)?;

    loop {
        let event = tui.next().await?;

        let _ = tui.terminal.show_cursor()?;

        if let Event::Render = event.clone() {
            tui.terminal.draw(|f| {
                // sets up ui, who knows where it will go
                ui(f, &mut editor);
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
