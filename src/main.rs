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
    - change cursor interface ✅
    - clear command line on ctrl+c ✅
    - establish a widget for editor
    - figure out to get text from motionbuffer in MotionHandler

  After:
    - change tui channels and connect reciever to editor ✅
    - make channel between motion and editor, and then in reverse ✅
    
  Goal in main:
    select! {
        event = channel from tui { handle event }, ✅
        motion/action = channel from motion { handle funcs to modify buffer }, ✅
    }

    handle ui {
        draw()
        color() // color cells for syntax highlighting
    }

  Finally:
    - remove commented out code
        - main.rs
        - tui.rs
        - editor.rs?

   ⭐ don't render ui on updates but by frame rate, maybe
==================== */


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    let filename = filename.unwrap_or(String::from("."));

    // return tui and a recv channel
    // let mut (tui, event_recv) = Tui::new()?.tick_rate(1.0);
    let mut tui = Tui::new()?.tick_rate(1.0);

    let mut editor = Editor::new()?;
    editor.new_buffer(&filename);

    tui.enter()?; 
    tui.start();


    loop {
        // TODO: Switch cursor interface used
        // won't break the cursor

        tui.terminal.draw(|f| {
            editor.set_cursor(f);
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

            _action = editor.next_action() => {
                editor.send("motion reciever".to_string());
                // editor.handle_action()
            }
            /*
            event = tui.next() => {
                match event {
                    Ok(event) => update(&mut editor, event, &mut tui),
                    Err(_) => {}
                }
            },
            */

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
