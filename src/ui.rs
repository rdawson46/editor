use crate::Event;
use crate::Tui;
use crate::X_OFFSET;
use crossterm::event::KeyCode;
use crate::editor::{Editor, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};


pub fn ui(f: &mut Frame<'_>, editor: &mut Editor){
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
                     Constraint::Length(X_OFFSET - 1),
                     Constraint::Min(1)
        ])
        .split(wrapper_layout[0]);
    
    editor.size = (num_text_layout[1].width, num_text_layout[1].height);

    // loop to make text for line nums and file text
    let mut line_nums = "".to_string();
    let mut text_string = "".to_string();


    // TODO: use i to determine if something should be rendered
    for (i, line) in editor.lines.lines.iter().enumerate() {
        if i >= editor.ptr.into() && i <= usize::from(editor.ptr + editor.size.1)  {
            let mut i_str: String;
            let current_line = usize::from(editor.ptr + editor.cursor.current.1);

            if current_line != i {
                if current_line > i {
                    i_str = (current_line - i).to_string();
                } else{
                    i_str = (i - current_line).to_string();
                }

            } else {
                i_str = (i + 1).to_string();
                if i_str.len() <= 2 {
                    i_str.push(' ');
                }
            }

            i_str.push('\n');
            i_str.push('\r');

            for char in i_str.chars() {
                line_nums.push(char);
            }

            for char in line.text.chars() {
                text_string.push(char);
            }

            text_string.push('\n');
            text_string.push('\r');
        }
    }

    f.render_widget(editor.mode_display(), wrapper_layout[1]);

    f.render_widget(Paragraph::new(line_nums)
                    .alignment(ratatui::layout::Alignment::Right)
                    .style(Style::default().fg(Color::DarkGray)),
                    num_text_layout[0]);
    
    f.render_widget(Paragraph::new(text_string)
                    .block(Block::default()
                       .padding(Padding::new(1, 0, 0, 0))),
                    num_text_layout[1]);
}



//  TODO: fix how modes switch
//  TODO: move cursor management to editor
pub fn update(editor: &mut Editor, event: Event, tui: &mut Tui){
    match event {
        Event::Init => {println!("init found");},
        Event::Quit => {println!("quit found");},
        Event::Error => {println!("Error found");},
        Event::Closed => {println!("Closed found");},
        Event::Tick => {},
        Event::Render => {},
        Event::FocusGained => {println!("FocusGained found");},
        Event::FocusLost => {println!("FocusLost found");},
        Event::Paste(_) => {println!("Paste found");},
        Event::Key(key) => {
            match editor.mode {
                Mode::Insert => {
                    match key.code {
                        KeyCode::Char(value) => {
                            match value {
                                'Q' => editor.change_mode(Mode::Normal),
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },

                Mode::Normal => {
                    match key.code {
                        KeyCode::Char(value) => {
                            // FIX: change to ctrl + q
                            match value {
                                'Q' => editor.should_quit = true,
                                'j' => editor.move_down(),
                                'k' => editor.move_up(),
                                'h' => editor.move_left(),
                                'l' => editor.move_right(),
                                'i' => editor.change_mode(Mode::Insert),
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
            }
        },
        Event::Mouse(_) => {println!("Mouse found");},
        Event::Resize(x, y) => {
            println!("Resize found");
            tui.size = (x, y);
        },
    }
}
