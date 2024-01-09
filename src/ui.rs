use crate::Event;
use crossterm::event::KeyCode;

#[warn(unused_imports)]
use crossterm::event::KeyEvent;

#[warn(unused_imports)]
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame
};

use crate::app::Editor;

// NOTE:: follow tasks in order of significance
pub fn ui(f: &mut Frame<'_>, _editor: &mut Editor){
    // get size of terminal
        // find intersection between lines and screen
    // append the lines in the intersection
            // figure out how to show cursor
    // and how to iter over lines
                // check if a wiget can be defined for row count
                // bottom widget for commands and status
    let _size = f.size();

    f.render_widget(Paragraph::new("Text object here"), f.size());
    f.render_widget(Paragraph::new("Text object 2 here"), f.size());
}

// NOTE: modify the editor based off the event
pub fn update(editor: &mut Editor, event: Event){
    match event {
        Event::Init => {println!("init found");},
        Event::Quit => {println!("quit found");},
        Event::Error => {println!("Error found");},
        Event::Closed => {println!("Closed found");},
        Event::Tick => {println!("Tick found");},
        Event::Render => {},
        Event::FocusGained => {println!("FocusGained found");},
        Event::FocusLost => {println!("FocusLost found");},
        Event::Paste(_) => {println!("Paste found");},
        Event::Key(key) => {
            // FIX: fix this to allow for more flexability
            match key.code {
                KeyCode::Char(value) => {
                    // FIX: change to ctrl + q
                    if value == 'Q'{
                        editor.should_quit = true;
                    } else {
                        println!("{}", value);
                    }
                },
                _ => {}
            }
        },
        Event::Mouse(_) => {println!("Mouse found");},
        Event::Resize(_, _) => {println!("Resize found");},
    }
}
