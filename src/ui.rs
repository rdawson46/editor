use crate::Event;
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
pub fn ui(f: &mut Frame<'_>, editor: &Editor){
    
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
        Event::Key(_) => {},
        Event::Mouse(_) => {},
        Event::Resize(_, _) => {},
    }
}
