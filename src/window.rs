use crate::buffer::Buffer;
use color_eyre::eyre::Result;

pub struct Window {
    pub size: (u16, u16),
    pub buffers: Vec<Buffer>,
    pub buf_ptr: usize, // will point to the buffer to be typed in
}

impl Window {
    #[inline]
    pub fn new() -> Self {
        // FIX: return empty vec with ptr 0 is bad
        return Window {
            size: (0, 0),
            buffers: vec![],
            buf_ptr: 0
        };
    }

    #[inline]
    pub fn add_buffer(&mut self, new_buffer: Buffer) -> Result<()> {
        self.buffers.push(new_buffer);
        Ok(())
    }

    pub fn remove_buffer(&mut self) -> Result<()> {

        Ok(())
    }

    // TODO: will need more work for multi buffering
    pub fn resize(&mut self, size: (u16, u16)) -> Result<()> {
        self.size = size;

        for buffer in self.buffers.iter_mut() {
            buffer.resize(size)
        }

        Ok(())
    }

    // ui function, get view port of all buffers in window
    pub fn get_view() {}

    pub fn move_buffer() {}

    pub fn move_buffer_to_new_window() {}

    pub fn key_press() {}
}
