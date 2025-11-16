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
use crate::editor::Editor;
use editor_core::buffer::{Buffer, Mode};

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

struct EditorWidget {
    status_line: StatuslineWidget,
    buffer: BufferWidget,
}

impl EditorWidget {
    fn new(e: &Editor) -> Self {
        return EditorWidget {
            status_line: StatuslineWidget::new(e),
            buffer: BufferWidget::new(&e.buffers[e.buf_ptr]),
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
    mode: Mode,
    location: (u16, u16),
}

impl StatuslineWidget {
    pub fn new(e: &Editor) -> Self {
        return StatuslineWidget {
            command: "".to_string(),
            motion: "".to_string(),
            mode: e.buffers[e.buf_ptr].mode,
            location: (0,0),
        }
    }
}

impl Widget for StatuslineWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        buf.set_string(area.left(), area.top(), self.command, Style::default());
        
        let location = format!("[{}:{}]", self.location.0, self.location.1);
        buf.set_string(area.right(), area.top(), location, Style::default());
    }
}
