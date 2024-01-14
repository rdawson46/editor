use core::panic;

use crate::editor::Mode;

#[test]
fn test_editor(){
    use crate::Editor;

    let filename = "./Cargo.toml";
    let filename = std::path::Path::new(&filename);

    let mut editor = Editor::new(filename).unwrap();
    assert_eq!(editor.cursor.current.1, 0);
    
    editor.move_down();
    assert_eq!(editor.cursor.current.1, 1);
    
    editor.move_up();
    assert_eq!(editor.cursor.current.1, 0);

    editor.move_up();
    assert_eq!(editor.cursor.current.1, 0);

    match &editor.mode {
        Mode::Normal => {},
        _ => panic!("starting mode not Normal")
    }

    editor.change_mode(Mode::Insert);

    match &editor.mode {
        Mode::Insert => {},
        _ => panic!("mode didn't change properly")
    }

    editor.change_mode(Mode::Normal);

    match &editor.mode {
        Mode::Normal => {},
        _ => panic!("mode didn't change properly")
    }
}

#[test]
fn test_start_and_close() {
    use crate::Tui;
    let tui = Tui::new().unwrap();
    
    tui.enter().unwrap();
    assert!(crossterm::terminal::is_raw_mode_enabled().unwrap());

    tui.exit().unwrap();
    assert!(!(crossterm::terminal::is_raw_mode_enabled().unwrap()));
}
