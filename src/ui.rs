use std::rc::Rc;
use std::usize;

use crate::Event;
use crate::Tui;
use crate::X_OFFSET;
use crate::command::CommandKey;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crate::editor::{Editor, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

// TODO: replace editor.ptr with y_ptr and x_ptr
    // for horizontal scrolling


fn get_layouts(f: &mut Frame<'_>) -> (Rc<[Rect]>, Rc<[Rect]>) {
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

    return (wrapper_layout, num_text_layout);
}



pub fn ui(f: &mut Frame<'_>, editor: &mut Editor){
    let (wrapper_layout, num_text_layout) = get_layouts(f);
    
    editor.size = (num_text_layout[1].width, num_text_layout[1].height);

    // loop to make text for line nums and file text
    let mut line_nums = "".to_string();
    let mut text_string = "".to_string();

    for (i, line) in editor.lines.lines.iter().skip(editor.ptr_y.into()).enumerate() {
        if i > usize::from(editor.ptr_y + editor.size.1) {
            break;
        }

        let mut i_str: String;
        let current_line = usize::from(editor.ptr_y + editor.cursor.current.1);

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

        text_string.push('\r');
        text_string.push('\n');
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

            // TODO: add movable cursor with arrow keys
            match editor.mode {
                Mode::Insert => {
                    editor.insert_key(key);
                },

                Mode::Command => {
                    match key.code {
                        KeyCode::Char(value) => {
                            if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                editor.change_mode(Mode::Normal);
                            } else {
                                editor.command.text.push(value);
                            }
                        },
                        KeyCode::Esc => {
                            editor.change_mode(Mode::Normal);
                        }
                        KeyCode::Enter => {
                            let command = editor.command.confirm();

                            match command {
                                Some(command) => {
                                    match command {
                                        CommandKey::Save => editor.save(),
                                        CommandKey::Quit => editor.should_quit = true,
                                        CommandKey::Line(number) => {
                                            editor.go_to_line(number);
                                        },
                                        CommandKey::SaveAndQuit => {
                                            editor.save();
                                            editor.should_quit = true;
                                        },
                                        CommandKey::History => todo!(),
                                    }
                                },
                                None => {}
                            }
                            editor.change_mode(Mode::Normal);
                        },
                        KeyCode::Backspace => {
                            if editor.command.text.len() > 0 {
                                // TODO: add movable cursor
                                editor.command.text.pop();
                            } else {
                                editor.change_mode(Mode::Normal);
                            }
                        },
                        _ => {}
                    }
                },

                Mode::Normal => {
                    match key.code {
                        KeyCode::Char(value) => {
                            if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                editor.should_quit = true;
                            } else if value == 's' && key.modifiers == KeyModifiers::CONTROL {
                                editor.save();
                            }
                            match value {
                                ':' => editor.change_mode(Mode::Command),
                                'j' => editor.move_down(),
                                'k' => editor.move_up(),
                                'h' => editor.move_left(),
                                'l' => editor.move_right(),
                                'i' => editor.change_mode(Mode::Insert),
                                'a' => {
                                    editor.change_mode(Mode::Insert);
                                    editor.move_right();
                                },
                                'w' => editor.move_next_word(),
                                'b' => editor.move_back_word(),
                                'e' => editor.move_end_word(),
                                '0' => editor.move_begin_of_line(),
                                '$' => editor.move_end_of_line(),
                                'I' => {
                                    editor.change_mode(Mode::Insert);
                                    editor.move_begin_of_line();
                                },
                                'A' => {
                                    editor.change_mode(Mode::Insert);
                                    editor.move_end_of_line();
                                },
                                val => {
                                    editor.motion.push(val)
                                }
                            }
                        },
                        KeyCode::Esc => {
                            editor.should_quit = true;
                        }
                        _ => {}
                    }
                },
            }
        },
        Event::Mouse(_) => {},
        Event::Resize(x, y) => {
            tui.size = (x, y);
        },
    }
}
