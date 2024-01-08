use crossterm::event::Event;
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

pub fn update(editor: &mut Editor, event: Event){

}
