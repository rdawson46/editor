#[tokio::test]
async fn test_editor(){
    use crate::editor::Editor;
    use editor_motion::MotionHandler;

    let (_motion, motion_sender, clear_sender, motion_buffer_listener) = MotionHandler::new();
    let editor_res = Editor::new(motion_sender, clear_sender, motion_buffer_listener);

    assert!(editor_res.is_ok());

    let mut editor = editor_res.unwrap();

    println!("testing logger");
    assert!(editor.logger.is_none());

    println!("testing buffer length 0");
    assert_eq!(editor.buffers.len(), 0);

    editor.new_buffer(&"./src/main.rs".to_string());
    println!("testing buffer length 1");
    assert_eq!(editor.buffers.len(), 1);
}

#[test]
fn test_buffer() {
    use editor_core::buffer::{Buffer, Mode, BufferType};

    let b_res = Buffer::new(&"./src/main.rs".to_string(), (10, 10));
    assert!(b_res.is_ok());

    let mut b = b_res.unwrap();

    assert_eq!(b.cursor.current.0, 0);
    assert_eq!(b.cursor.current.1, 0);
    b.move_down((10, 10));
    b.move_down((10, 10));
    assert_eq!(b.cursor.current.1, 2);

    assert_eq!(b.mode, Mode::Normal);
    b.change_mode(Mode::Insert);
    assert_eq!(b.mode, Mode::Insert);
    b.change_mode(Mode::Normal);
    assert_eq!(b.mode, Mode::Normal);

    assert_eq!(b.buffer_type, BufferType::File);

    let b_res = Buffer::new(&".".to_string(), (5, 5));

    assert!(b_res.is_ok());

    let b = b_res.unwrap();
    assert_eq!(b.buffer_type, BufferType::Directory);
}


#[test]
fn test_start_and_close() {
    /*
    use crate::Tui;
    let tui = Tui::new().unwrap();
    
    tui.enter().unwrap();
    assert!(crossterm::terminal::is_raw_mode_enabled().unwrap());

    tui.exit().unwrap();
    assert!(!(crossterm::terminal::is_raw_mode_enabled().unwrap()));
    */
}
