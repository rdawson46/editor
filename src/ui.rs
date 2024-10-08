use std::rc::Rc;
use crate::{
    editor::Editor, motion::MotionHandler, Event, Tui, X_OFFSET
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    prelude::Style,
    style::Stylize,
    Frame,
};


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


// TODO: fix this to allow motions
pub fn ui(f: &mut Frame<'_>, editor: &mut Editor, motion: &mut MotionHandler){
    let (wrapper_layout, num_text_layout) = get_layouts(f);
    editor.resize((num_text_layout[1].width, num_text_layout[1].height));

    let status = editor.mode_display();
    let motion = motion.get_text();

    match motion {
        Some(motion) => {

            let motion = Paragraph::new(motion)
                .block(Block::default()
                       .borders(Borders::TOP)
                       .border_style(Style::new().blue()));
            let status_motion = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                             Constraint::Percentage(50),
                             Constraint::Percentage(50)
                ])
                .split(wrapper_layout[1]);

            f.render_widget(status.to_owned(), status_motion[0]);
            f.render_widget(motion, status_motion[1]);
        },
        None => {
            f.render_widget(status.to_owned(), wrapper_layout[1]);
        }
    }

    let (line_par, text_par) = editor.buffer_display();

    f.render_widget(line_par, num_text_layout[0]);
    f.render_widget(text_par, num_text_layout[1]);
}


pub fn update(editor: &mut Editor, event: Event, tui: &mut Tui){
    match event {
        Event::Init => {
            editor.send("application initialized".to_string());
        },
        Event::Quit => {
            editor.send("application quiting".to_string());
        },
        Event::Error => {
            editor.send("error encountered".to_string());
        },
        Event::Closed => {},
        Event::Tick => {},
        Event::FocusGained => {
            editor.send("focus gained".to_string());
        },
        Event::FocusLost => {
            editor.send("focus lost".to_string());
        },
        Event::Paste(text) => {
            editor.paste(text)
        },
        Event::Key(key) => {
            editor.key_press(key);
        },
        Event::Mouse(mouse_event) => {
            editor.handle_mouse(mouse_event);
        },
        Event::Resize(x, y) => {
            tui.size = (x, y);
        },
        _ => {}
    }
}
