#[cfg(test)]
mod test;

mod editor;
mod motion;
mod command;
mod ui;
mod tui;
mod word;
mod buffer;
use crate::editor::{
    Editor,
    Mode
};
use crate::tui::{
    Tui,
    Event
};
use crate::ui::{ui, update};

use color_eyre::eyre::Result;


static X_OFFSET: u16 = 5;


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);

    let filename = filename.unwrap_or(String::from("."));

    //let filename = filename.unwrap();
    let filename = std::path::Path::new(&filename);

    let mut tui = Tui::new()?.tick_rate(1.0);
    let mut editor = Editor::new(filename)?;

    tui.enter()?; 

    tui.start();

    loop {
        tui.terminal.show_cursor()?;
        
        match &editor.buffers[editor.buf_ptr].mode {
            Mode::Command => {
                tui.terminal.set_cursor((editor.command.text.len() + 1).try_into().unwrap(), tui.size.1)?;
            },
            _ => {
                tui.terminal.set_cursor(editor.buffers[editor.buf_ptr].cursor.current.0 + X_OFFSET, editor.buffers[editor.buf_ptr].cursor.current.1)?;
            }
        };

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
