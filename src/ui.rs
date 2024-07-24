use std::rc::Rc;
use crate::{
    Event,
    Tui,
    X_OFFSET,
    editor::Editor
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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

    let (line_par, text_par) = editor.buffer_display();

    f.render_widget(line_par, num_text_layout[0]);
    f.render_widget(text_par, num_text_layout[1]);
}


pub fn update(editor: &mut Editor, event: Event, tui: &mut Tui){
    match event {
        Event::Init => {},
        Event::Quit => {},
        Event::Error => {},
        Event::Closed => {},
        Event::Tick => {},
        Event::FocusGained => {},
        Event::FocusLost => {},
        Event::Paste(text) => {
            editor.paste(text)
        },
        Event::Key(key) => {
            editor.key_press(key);
        },
        Event::Mouse(_) => {},
        Event::Resize(x, y) => {
            tui.size = (x, y);
        },
        _ => {}
    }
}
