use std::io::{Write, Result as Rs};

use crossterm::{
    execute, queue,
    style, terminal, cursor,
    //event::KeyCode,
    ExecutableCommand,
};

pub struct Buffer {
    lines: Vec<String>,
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

    pub fn get_line<'a>(&'a mut self) -> &'a mut String {
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        return &mut self.lines[i];
    }

    pub fn queue_reprint<W>(&self, w: &mut W) -> Rs<()>
    where W: Write {
        queue!(
            w,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;
        let scroll_bottom = (self.lines.len() - self.scroll_pos).min(self.scroll_pos + self.term_size.1 as usize);
        let wp = self.lines.get(self.scroll_pos..scroll_bottom).expect("Not enough lines in the buffer");
        for line in wp {
            queue!(
                w,
                style::Print(line),
                cursor::MoveToNextLine(1),
            )?;
        }
        Ok(())
    }
    
    pub fn scroll<W>(&mut self, w: &mut W, delta: isize) -> Rs<()>
    where W: Write {
        if delta == 0 { return Ok(()); }
        w.execute(cursor::SavePosition)?;

        if delta < 0 {
            // Scroll up
            let new_scroll_pos = (self.scroll_pos as isize + delta as isize).max(0) as usize;
            if self.scroll_pos != new_scroll_pos {
                self.scroll_pos = new_scroll_pos;
                self.queue_reprint(w)?;
                w.flush()?;
            }
        } else if self.lines.len() - 1 > self.term_size.1 as usize {
            // Scroll down
            let new_scroll_pos = (self.scroll_pos + delta as usize).min(self.lines.len() - 1 - self.term_size.1 as usize);
            if self.scroll_pos != new_scroll_pos {
                self.scroll_pos = new_scroll_pos;
                self.queue_reprint(w)?;
                w.flush()?;
            }
        }

        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        execute!(
            w,
            cursor::RestorePosition,
            cursor::MoveToColumn((self.cursor_pos.0 as u16).min(self.lines[i].len() as u16)),
        )?;

        Ok(())
    }

    pub fn save_cursor_pos(&mut self) -> Rs<()> {
        self.cursor_pos = cursor::position()?;
        Ok(())
    }

    pub fn move_cursor_v<W>(&mut self, w: &mut W, delta_row: i16) -> Rs<()>
    where W: Write {
        if delta_row == 0 { return Ok(()); }
        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        if i as i16 + delta_row < 0 || (i as i16 + delta_row) as usize > self.lines.len() - 1 {
            return Ok(());
        }

        let new_pos = self.cursor_pos.1 as i16 + delta_row;
        if new_pos < 0 {
            //w.execute(cursor::MoveToRow(0))?;
            self.cursor_pos = (self.cursor_pos.0, 0);
            self.scroll(w, new_pos as isize)?;
        } else if new_pos as u16 > self.term_size.1 - 1 {
            //w.execute(cursor::MoveToRow(self.term_size.1 - 1))?;
            self.cursor_pos = (self.cursor_pos.0, self.term_size.1 - 1);
            self.scroll(w, new_pos as isize - self.term_size.1 as isize + 1)?;
        } else {
            self.cursor_pos = (self.cursor_pos.0, new_pos as u16);
            w.execute(cursor::MoveToRow(new_pos as u16))?;
        }

        let i = self.scroll_pos + self.cursor_pos.1 as usize;
        w.execute(cursor::MoveToColumn((self.cursor_pos.0 as u16).min(self.lines[i].len() as u16)))?;
        Ok(())
    }

    pub fn newline_after<W>(&mut self, w: &mut W, new_line: String) -> Rs<()>
    where W: Write {
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
    where W: Write {
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
    where W: Write {
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
    where W: Write {
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
}
