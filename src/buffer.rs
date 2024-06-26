use std::io::{Result as Rs, Write};

use crossterm::{
    cursor,
    execute,
    queue,
    style,
    terminal,
    //event::KeyCode,
    ExecutableCommand,
};

use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
};

pub struct Buffer {
    pub lines: Vec<String>,
    /// Deprecated
    cursor_pos: (u16, u16),
    scroll_pos: usize,
    term_size: (u16, u16),
}

impl Buffer {
    pub fn new(lines: Vec<String>) -> Rs<Self> {
        Ok(Self {
            lines,
            cursor_pos: (0, 0),
            scroll_pos: 0,
            term_size: terminal::size()?,
        })
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.term_size = (width, height);
    }

    pub fn cursor(&self) -> (u16, u16) {
        let (x, y) = self.cursor_pos;
        let i = self.scroll_pos + y as usize;
        ((x as usize).min(self.lines[i].len()) as u16, y)
    }

    pub fn get_line<'a>(&'a mut self) -> &'a mut String {
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        return &mut self.lines[i];
    }

    pub fn queue_reprint<W>(&self, w: &mut W) -> Rs<()>
    where
        W: Write,
    {
        queue!(
            w,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;
        let scroll_bottom = (self.scroll_pos + self.term_size.1 as usize).min(self.lines.len());
        let wp = self
            .lines
            .get(self.scroll_pos..scroll_bottom)
            .expect("Not enough lines in the buffer");
        for line in wp {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1),)?;
        }
        Ok(())
    }

    pub fn scroll<W>(&mut self, _w: &mut W, delta: isize) -> Rs<()>
    where
        W: Write,
    {
        use std::cmp::Ordering::*;
        self.scroll_pos = match delta.cmp(&0) {
            Less => self.scroll_pos.saturating_add_signed(delta),
            Greater => {
                let scroll_max = self.lines.len().saturating_sub(self.term_size.1 as usize);
                (self.scroll_pos + delta as usize).min(scroll_max)
            }
            Equal => return Ok(()),
        };
        Ok(())
    }

    pub fn save_cursor_pos(&mut self) -> Rs<()> {
        self.cursor_pos = cursor::position()?;
        Ok(())
    }

    pub fn move_cursor_v<W>(&mut self, w: &mut W, delta_row: i16) -> Rs<()>
    where
        W: Write,
    {
        if delta_row == 0 {
            return Ok(());
        }
        let (x, y) = self.cursor_pos;

        let i = match (self.scroll_pos + y as usize).checked_add_signed(delta_row as isize) {
            Some(i) => {
                if i > self.lines.len() - 1 {
                    return Ok(());
                } else {
                    i
                }
            }
            None => return Ok(()),
        };
        let x = (x as usize).min(self.lines[i].len()) as u16;

        let y = y as i16 + delta_row;
        if y < 0 {
            self.cursor_pos = (x, 0);
            self.scroll(w, y as isize)?;
        } else if let Some(delta) = (y as u16).checked_sub(self.term_size.1 - 1) {
            self.cursor_pos = (x, self.term_size.1 - 1);
            self.scroll(w, delta as isize)?;
        } else {
            self.cursor_pos = (x, y as u16);
        }

        Ok(())
    }

    pub fn newline_after<W>(&mut self, w: &mut W, new_line: String) -> Rs<()>
    where
        W: Write,
    {
        let i = self.scroll_pos + self.cursor_pos.1 as usize + 1;
        self.lines.insert(i, new_line);
        self.cursor_pos = (0, self.cursor_pos.1);
        self.move_cursor_v(w, 1)?;
        if self.cursor_pos.1 + 1 < self.term_size.1 {
            self.queue_reprint(w)?;
            queue!(w, cursor::MoveTo(self.cursor_pos.0, self.cursor_pos.1))?;
            w.flush()?;
        }
        Ok(())
    }

    pub fn newline<W>(&mut self, w: &mut W) -> Rs<()>
    where
        W: Write,
    {
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        if self.cursor_pos.0 as usize >= self.lines[i].len() {
            // Append line
            self.newline_after(w, String::new())?;
        } else {
            // Split line
            let new_line = self.lines[i].split_off(self.cursor_pos.0 as usize);
            self.newline_after(w, new_line)?;
        }
        Ok(())
    }

    pub fn delete_newline_before<W>(&mut self, w: &mut W) -> Rs<()>
    where
        W: Write,
    {
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        if i == 0 {
            return Ok(());
        }
        let i = i - 1;
        let col = self.lines[i].len() as u16;
        let after = self.lines.remove(i + 1);
        self.lines[i].push_str(&after);
        self.queue_reprint(w)?;
        queue!(w, cursor::MoveTo(col, self.cursor_pos.1))?;
        w.flush()?;
        self.cursor_pos = (col, self.cursor_pos.1);
        self.move_cursor_v(w, -1)?;
        Ok(())
    }

    pub fn delete_newline_after<W>(&mut self, w: &mut W) -> Rs<()>
    where
        W: Write,
    {
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        if i >= self.lines.len() - 1 {
            return Ok(());
        }
        w.execute(cursor::SavePosition)?;
        let after = self.lines.remove(i + 1);
        self.lines[i].push_str(&after);
        self.queue_reprint(w)?;
        queue!(w, cursor::RestorePosition)?;
        w.flush()?;
        Ok(())
    }

    pub fn widget<'a>(&'a self) -> Renderer<'a> {
        Renderer(self)
    }
}

pub struct Renderer<'a>(&'a Buffer);

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let top_row = self.0.scroll_pos;
        let bottom_row = self.0.lines.len().min(top_row + area.height as usize);
        let text = Text::from_iter(
            self.0
                .lines
                .iter()
                .skip(top_row)
                .take(bottom_row - top_row)
                .map(|s| Line::raw(s)),
        );
        Paragraph::new(text).render(area, buf);
    }
}
