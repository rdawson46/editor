#[cfg(test)]
mod test;

mod editor;
mod ui;
mod tui;
mod word;
use crate::editor::Editor;
use crate::tui::{Tui, Event};
use crate::ui::{ui, update};

use color_eyre::eyre::Result;


static X_OFFSET: u16 = 5;


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    if filename.is_none() {
        // TODO: open an empty buffer when no file is provided
        println!("no file provided");
        return Ok(());
    }

    let filename = filename.unwrap();
    let filename = std::path::Path::new(&filename);

    let mut tui = Tui::new()?.tick_rate(1.0);
    let mut editor = Editor::new(filename)?;

    tui.enter()?; 

    tui.start();

    loop {
        tui.terminal.show_cursor()?;
        tui.terminal.set_cursor(editor.cursor.current.0 + X_OFFSET, editor.cursor.current.1)?;

        if tui.update {
            tui.update = false;
            tui.terminal.draw(|f| {
                ui(f, &mut editor);
            })?;
        }
        
        let event = tui.next().await?;
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
