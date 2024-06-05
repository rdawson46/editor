use std::rc::Rc;
use crate::{
    Event,
    Tui,
    X_OFFSET,
    editor::Editor
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
/*
 * --> commented until ready
use tree_sitter_rust;
use tree_sitter_highlight::{
    HighlightEvent,
    Highlighter,
    HighlightConfiguration
};
*/

/* ======== ROADMAP =======

 - Thought Process:
    * gonna need to minorly refactor this file
        * getting lines from buffers/editor?

 - Actions:
    * Impl windows now to make life easier later

 - Improvements:
    * simplify ui func
        * make func else where for getting the text and line numbers 
    * give a size property to buffer
        * will make it possible for multi-buffered windows
            * could fix other issues with word jumping ⭐
    * functions for resizing buffers & editor

 - Current Functions:
    * get_layouts
    * ui
    * update
        
 - New Functions Ideas:
    * get_highlight
    * file_type (editor)

 - What Might Need To Happen:
    * make cells impl for highlighting instead of rendering full lines

======================== */


fn get_layouts(f: &mut Frame<'_>) -> (Rc<[Rect]>, Rc<[Rect]>) {
    // wrapper_layout[0] is for the text and line numbers
    // wrapper_layout[1] is for the command view
    let wrapper_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
                     Constraint::Min(1),
                     Constraint::Length(2)
        ])
        .split(f.size());

    let num_text_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
                     Constraint::Length((X_OFFSET - 1).try_into().unwrap()),
                     Constraint::Min(1)
        ])
        .split(wrapper_layout[0]);

    return (wrapper_layout, num_text_layout);
}


pub fn ui(f: &mut Frame<'_>, editor: &mut Editor){
    let (wrapper_layout, num_text_layout) = get_layouts(f);
    editor.resize((num_text_layout[1].width, num_text_layout[1].height));

    let (status, motion) = editor.mode_display();
    match motion {
        Some(motion) => {
            let status_motion = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                             Constraint::Percentage(50),
                             Constraint::Percentage(50)
                ])
                .split(wrapper_layout[1]);

            f.render_widget(status.to_owned(), status_motion[0]);
            f.render_widget(motion.to_owned(), status_motion[1]);
        },
        None => {
            f.render_widget(status.to_owned(), wrapper_layout[1]);
        }
    }

    let (line_nums, text_string) = editor.buffer_display();

    f.render_widget(Paragraph::new(line_nums)
                    .alignment(ratatui::layout::Alignment::Right)
                    .style(Style::default().fg(Color::DarkGray)),
                    num_text_layout[0]);
    
    f.render_widget(Paragraph::new(text_string)
                    .block(Block::default()
                    .padding(Padding::new(1, 0, 0, 0))),
                    num_text_layout[1]);
}


pub fn update(editor: &mut Editor, event: Event, tui: &mut Tui){
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
            editor.key_press(key);
        },
        Event::Mouse(_) => {},
        Event::Resize(x, y) => {
            tui.size = (x, y);
        },
    }
}
