use crate::text_buffer::{ConsoleChar, TextBuffer};
use alloc::collections::VecDeque;
use alloc::vec::Vec;

pub struct TextBufferCache<T: TextBuffer> {
    buf: VecDeque<Vec<ConsoleChar>>,
    inner: T,
}

impl<T: TextBuffer> TextBufferCache<T> {
    pub fn new(inner: T) -> Self {
        TextBufferCache {
            buf: VecDeque::from(vec![
                vec![ConsoleChar::default(); inner.width()];
                inner.height()
            ]),
            inner,
        }
    }
    fn flush(&mut self) {
        for i in 0..self.height() {
            for j in 0..self.width() {
                self.inner.write(i, j, self.buf[i][j]);
            }
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
        self.buf[row][col]
    }
    fn write(&mut self, row: usize, col: usize, ch: ConsoleChar) {
        self.buf[row][col] = ch;
        self.inner.write(row, col, ch);
    }
    fn new_line(&mut self) {
        let mut new_line = self.buf.pop_front().unwrap();
        for c in new_line.iter_mut() {
            *c = ConsoleChar::default();
        }
        self.buf.push_back(new_line);
        self.flush();
    }
}
