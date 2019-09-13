use crate::text_buffer::{ConsoleChar, TextBuffer};
use alloc::vec::Vec;

/// Cache layer for [`TextBuffer`]
pub struct TextBufferCache<T: TextBuffer> {
    buf: Vec<Vec<ConsoleChar>>,
    row_offset: usize,
    inner: T,
}

impl<T: TextBuffer> TextBufferCache<T> {
    /// Create a cache layer for `inner` text buffer
    pub fn new(inner: T) -> Self {
        TextBufferCache {
            buf: vec![vec![ConsoleChar::default(); inner.width()]; inner.height()],
            row_offset: 0,
            inner,
        }
    }
    /// Get real row of inner buffer
    fn real_row(&self, row: usize) -> usize {
        (self.row_offset + row) % self.inner.height()
    }
    /// Clear line at `row`
    fn clear_line(&mut self, row: usize) {
        for col in 0..self.width() {
            self.buf[row][col] = ConsoleChar::default();
            self.inner.write(row, col, ConsoleChar::default());
        }
    }
}

impl<T: TextBuffer> TextBuffer for TextBufferCache<T> {
    fn width(&self) -> usize {
        self.inner.width()
    }
    fn height(&self) -> usize {
        self.inner.height()
    }
    fn read(&self, row: usize, col: usize) -> ConsoleChar {
        let row = self.real_row(row);
        self.buf[row][col]
    }
    fn write(&mut self, row: usize, col: usize, ch: ConsoleChar) {
        let row = self.real_row(row);
        self.buf[row][col] = ch;
        self.inner.write(row, col, ch);
    }
    fn new_line(&mut self) {
        self.row_offset = (self.row_offset + 1) % self.inner.height();
        self.clear_line(self.row_offset);
    }
}
