#[cfg(test)]
mod test;

mod buffer;
mod colors;
mod command;
mod editor;
mod motion;
mod tui;
mod ui;
mod widgets;
mod window;
mod word;

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
    - establish a widget for:
        - editor
        - buffer
        - status line
    - figure out to get text from motionbuffer in MotionHandler 

  After:
    - change tui channels and connect reciever to editor ✅
    - make channel between motion and editor, and then in reverse ✅

    handle ui {
        draw()
        color() // color cells for syntax highlighting
    }

  Finally:
    - remove commented out code
        - main.rs
        - tui.rs
        - editor.rs?

   ⭐ don't render ui on updates but by frame rate, maybe ✅
==================== */


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    let filename = filename.unwrap_or(String::from("."));

    let mut tui = Tui::new()?.tick_rate(1.0);

    let mut editor = Editor::new()?;
    editor.new_buffer(&filename);

    tui.enter()?; 
    tui.start();

    loop {
        select! {
            event = tui.next() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Render => {
                                tui.terminal.draw(|f| {
                                    editor.set_cursor(f);
                                    ui(f, &mut editor);
                                    // color(f)???  // might be helpful with highlighting
                                })?;
                            },
                            _ => update(&mut editor, event, &mut tui),
                        }
                    },
                    Err(err) => {
                        editor.send("Hit crossterm error".to_string());
                        editor.send(err.to_string());

                        editor.should_quit = true;
                    }
                }
            },

            motion_buffer = editor.next_motion() => {
                match motion_buffer {
                    Ok(motion_buffer) => {
                        let _ = editor.parse(motion_buffer);
                    },
                    Err(err) => {
                        editor.send("motion error recv".to_string());
                        editor.send(err.to_string());
                        editor.should_quit = true;
                    }
                }
            }
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
