#[cfg(test)]
mod test;

mod ui;
mod tui;
mod editor;
mod buffer;
mod window;
mod motion;
mod command;
mod word;
mod colors;
use crate::editor::Editor;
use crate::buffer::Mode;
use crate::tui::{
    Tui,
    Event
};
use crate::ui::{ui, update};
use color_eyre::eyre::Result;


static X_OFFSET: usize = 5;


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    let filename = filename.unwrap_or(String::from("."));

    let mut tui = Tui::new()?.tick_rate(1.0);
    let mut editor = Editor::new()?;
    editor.new_buffer(&filename);

    tui.enter()?; 
    tui.start();

    loop {
        tui.terminal.show_cursor()?;
        
        match &editor.buffers[editor.buf_ptr].mode {
            Mode::Command => {
                tui.terminal.set_cursor((editor.command.text.len() + 1).try_into().unwrap(), tui.size.1)?;
            },
            _ => {
                tui.terminal.set_cursor(
                    (editor.buffers[editor.buf_ptr].cursor.current.0 + X_OFFSET).try_into().unwrap(),
                    (editor.buffers[editor.buf_ptr].cursor.current.1).try_into().unwrap()
                )?;
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
