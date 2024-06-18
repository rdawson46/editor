use ratatui::{
    prelude::{Alignment, Style},
    style::Stylize,
    widgets::{
        Block,
        Borders,
        Paragraph,
        Widget
    },
    Frame,
};
use crate::{
    editor::Editor,
    buffer::Buffer,
};

struct BufferWidget {
    text: String,
}

// might be too much data
impl BufferWidget {
    fn new(b: &Buffer) -> Self {
        return BufferWidget {
            text: b.lines.rope.to_string()
        };
    }
}

// text within view of current buffer
impl Widget for BufferWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {

    }
}

// what classifies this?
struct EditorWidget {
    status_line: StatuslineWidget,
    buffer: Option<BufferWidget>,
}

impl EditorWidget {
    fn new(e: &Editor) -> Self {
        return EditorWidget {
            status_line: StatuslineWidget::new(),
            buffer: None,
        };
    }
}


impl Widget for EditorWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        
    }
}

struct StatuslineWidget {
    command: String,
    motion: String,
    location: (u16, u16),
}

impl StatuslineWidget {
    pub fn new() -> Self {
        return StatuslineWidget {
            command: "".to_string(),
            motion: "".to_string(),
            location: (0,0),
        }
    }
}

impl Widget for StatuslineWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        
    }
}
