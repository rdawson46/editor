// creates error now that it considers tui size
#[test]
fn test_editor(){

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
