use crate::escape_parser::CharacterAttribute;

/// A character with attribute on screen
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(4))]
pub struct ConsoleChar {
    pub ascii_char: u8,
    pub attr: CharacterAttribute,
}

/// Empty char
impl Default for ConsoleChar {
    fn default() -> Self {
        ConsoleChar {
            ascii_char: b' ',
            attr: CharacterAttribute::default(),
        }
    }
}

/// A 2D array of [`ConsoleChar`] to render on screen
pub trait TextBuffer {
    /// Columns
    fn width(&self) -> usize;

    /// Rows
    fn height(&self) -> usize;

    /// Read the character at `(row, col)`
    ///
    /// Avoid use this because it's usually very slow on real hardware.
    fn read(&self, row: usize, col: usize) -> ConsoleChar;

    /// Write a character `ch` at `(row, col)`
    fn write(&mut self, row: usize, col: usize, ch: ConsoleChar);

    /// Delete one character at `(row, col)`.
    fn delete(&mut self, row: usize, col: usize) {
        self.write(row, col, ConsoleChar::default());
    }

    /// Insert one blank line at the bottom, and scroll up one line.
    ///
    /// The default method does single read and write for each pixel.
    /// Usually it needs rewrite for better performance.
    fn new_line(&mut self) {
        for i in 1..self.height() {
            for j in 0..self.width() {
                self.write(i - 1, j, self.read(i, j));
            }
        }
        for j in 0..self.width() {
            self.delete(self.height() - 1, j);
        }
    }

    /// Clear the buffer
    fn clear(&mut self) {
        for i in 0..self.height() {
            for j in 0..self.width() {
                self.delete(i, j);
            }
        }
    }
}
