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
    motion::MotionHandler,
};
use tokio::select;
use color_eyre::eyre::Result;


static X_OFFSET: usize = 5;

/* ====================
 Map to improve event loop: 
  Prep:
    - figure out to get text from motionbuffer in MotionHandler 
    - establish a widget for:
        - editor
        - buffer
        - status line
        - file tree

==================== */


async fn run() -> Result<()> {
    let filename = std::env::args().nth(1);
    let filename = filename.unwrap_or(String::from("."));

    let mut tui = Tui::new()?.tick_rate(1.0).render_rate(30.0);
    let (mut motion, motion_sender, clear_sender, motion_buffer_listener) = MotionHandler::new();
    let mut editor = Editor::new(motion_sender, clear_sender, motion_buffer_listener)?;

    editor.new_buffer(&filename);

    tui.enter()?; 
    tui.start();

    loop {
        select! {
            // get event from tui, draw on render, else update
            event = tui.next() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Render => {
                                tui.terminal.draw(|f| {
                                    editor.set_cursor(f);
                                    ui(f, &mut editor, &mut motion);
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

            // editor recv motion_buffer
            motion_buffer = editor.next_motion() => {
                match motion_buffer {
                    Ok(motion_buffer) => {
                        // IDEA: 
                        // use parse to create a function and push the function to a queue to run
                        // on update tick?
                        //
                        // TODO: 
                        // 1. edit the parsing function to return a function with param &editor
                        // 2. add an event queue to the editor
                        // 3. add an update tick
                        // 4. add a function to consume these fucntions
                        let _ = editor.parse(motion_buffer);
                    },
                    Err(err) => {
                        editor.send("motion error recv".to_string());
                        editor.send(err.to_string());
                        editor.should_quit = true;
                    }
                }
            },

            // recv char from editor
            c = motion.listener.recv() => {
                motion.handle_char(c);
            }

            _ = motion.clear.recv() => {
                motion.refresh();
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
