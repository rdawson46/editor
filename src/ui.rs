use crate::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame
};

use crate::app::Editor;

// NOTE:: not sure where I'm going with this
// maybe just wrap a border around the terminal
// then insert each line of text from the editor
pub fn ui(f: &mut Frame<'_>, _editor: &Editor){
    
}

// NOTE: modify the editor based off the event
pub fn update(editor: &mut Editor, event: Event){
    match event {
        Event::Init => {},
        Event::Quit => {},
        Event::Error => {},
        Event::Closed => {},
        Event::Tick => {},
        Event::Render => {},
        Event::FocusGained => {},
        Event::FocusLost => {},
        Event::Paste(_) => {},
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
        Event::Mouse(_) => {},
        Event::Resize(_, _) => {},
    }
}
