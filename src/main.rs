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
use crate::{
    editor::Editor,
    tui::{Tui, Event},
    ui::{ui, update},
};
use tokio::select;
use color_eyre::eyre::Result;


static X_OFFSET: usize = 5;

/* ====================
 Map to improve event loop: 
  Prep:
    - change cursor interface
    - establish a widget for editor

  After:
    - change tui channels and connect reciever to editor
    - make channel between motion and editor, and then in reverse
    
  Goal in main:
    select! {
        event = channel from tui { handle event },
        motion/action = channel from motion { handle funcs to modify buffer },
    }

    handle ui {
        draw()
        color() // color cells for syntax highlighting
    }

   ⭐ don't render ui on updates but by frame rate ⭐
==================== */


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    let filename = filename.unwrap_or(String::from("."));

    let mut tui = Tui::new()?.tick_rate(1.0);

    // TODO: assign an output to listen for actions to consume
    let mut editor = Editor::new()?;
    editor.new_buffer(&filename);

    tui.enter()?; 
    tui.start();


    loop {
        // TODO: Switch cursor interface used
        // won't break the cursor
        editor.set_cursor(&mut tui);

        tui.terminal.draw(|f| {
            ui(f, &mut editor);
            // color(f)???  // might be helpful with highlighting
        })?;
        
        select! {
            event = tui.next() => {
                match event {
                    Ok(event) => update(&mut editor, event, &mut tui),
                    Err(_) => {}
                }
            },

            /*
            motion_event = editor.next_motion() => {
                match motion_event {
                    Ok(action) => {},
                    Err(_) => {}
                }
            },
            */
        }

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
