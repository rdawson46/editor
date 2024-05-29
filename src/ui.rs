use std::rc::Rc;
use std::usize;
use crate::Event;
use crate::Tui;
use crate::X_OFFSET;
use crate::editor::Editor;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph},
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
    
    editor.size = (num_text_layout[1].width, num_text_layout[1].height);

    // loop to make text for line nums and file text
    let mut line_nums = "".to_string();
    let mut text_string = "".to_string();

    // for (i, line) in editor.buffers[editor.buf_ptr].lines.lines.iter().skip(editor.buffers[editor.buf_ptr].ptr_y.into()).enumerate() {
    for (i, line) in editor.buffers[editor.buf_ptr].lines.rope.lines().skip(editor.buffers[editor.buf_ptr].ptr_y).enumerate() {
        if i > editor.buffers[editor.buf_ptr].ptr_y + usize::from(editor.size.1) {
            break;
        }

        if i == editor.buffers[editor.buf_ptr].lines.rope.len_lines() - 1 {
            break;
        }

        let mut i_str: String;
        let current_line = usize::from(editor.buffers[editor.buf_ptr].cursor.current.1);

        if current_line != i {
            if current_line > i {
                i_str = (current_line - i).to_string();
            } else{
                i_str = (i - current_line).to_string();
            }

        } else {
            i_str = (editor.buffers[editor.buf_ptr].ptr_y + editor.buffers[editor.buf_ptr].cursor.current.1 + 1).to_string();
            if i_str.len() <= 2 {
                i_str.push(' ');
            }
        }

        i_str.push_str("\n\r");

        for char in i_str.chars() {
            line_nums.push(char);
        }

        for char in line.chars() {
            text_string.push(char);
        }
    }

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

            f.render_widget(status, status_motion[0]);
            f.render_widget(motion, status_motion[1]);
        },
        None => {
            f.render_widget(status, wrapper_layout[1]);
        }
    }

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
