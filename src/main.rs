mod editor;
mod ui;
mod tui;
mod word;
use crate::editor::Editor;
use crate::tui::{Tui, Event};
use crate::ui::{ui, update};

use color_eyre::eyre::Result;

async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    if filename.is_none() {
        println!("no file provided");
        return Ok(());
    }

    let filename = filename.unwrap();
    let filename = std::path::Path::new(&filename);

    //let mut tui = Tui::new()?.tick_rate(1.0).frame_rate(30.0);
    let mut tui = Tui::new()?.tick_rate(1.0).frame_rate(0.5);
    let mut editor = Editor::new(filename)?;

    tui.enter()?; 

    tui.start();

    loop {
        // check for event
        // update editor
        // render file information
        let event = tui.next().await?;
        
        tui.terminal.show_cursor()?;

        if let Event::Render = event.clone() {
            tui.terminal.draw(|f| {
                ui(f, &mut editor);
            })?;
        }
        
        update(&mut editor, event, &mut tui);

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
