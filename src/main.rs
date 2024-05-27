use std::io;

use crossterm::{
    execute,
    style, terminal, cursor,
    event::KeyCode,
    ExecutableCommand,
};

enum EditorState {
    CursorMode,
    InsertMode { after: String },
}

impl EditorState
{
    fn split_buf(&self, buf: &mut String) -> io::Result<String> {
        let after = match self {
            Self::CursorMode => {
                let (col, _) = cursor::position()?;
                buf.split_off((col as usize).min(buf.len()))
            }
            Self::InsertMode { after } => after.to_string(),
        };
        Ok(after)
    }

    pub fn print<W>(self, w: &mut W, buf: &mut String, c: char) -> io::Result<EditorState> 
    where W: io::Write,
    {
        let after = self.split_buf(buf)?;
        buf.push(c);
        if after.len() > 0 {
            execute!(
                w,
                style::Print(c),
                style::Print(&after),
                cursor::MoveLeft(after.len() as u16),
            )?;
        } else {
            w.execute(style::Print(c))?;
        }
        Ok(Self::InsertMode { after })
    }

    pub fn erase_left<W>(self, w: &mut W, buf: &mut String) -> io::Result<EditorState>
    where W: io::Write,
    {
        let after = self.split_buf(buf)?;
        buf.pop();
        if after.len() > 0 {
            execute!(
                w,
                cursor::MoveLeft(1),
                style::Print(&after),
                style::Print(' '),
                cursor::MoveLeft(after.len() as u16 + 1),
            )?;
        } else {
            execute!(
                w,
                cursor::MoveLeft(1),
                style::Print(' '),
                cursor::MoveLeft(1),
            )?;
        }
        Ok(Self::InsertMode { after })
    }

    pub fn erase_right<W>(self, w: &mut W, buf: &mut String) -> io::Result<EditorState>
    where W: io::Write,
    {
        let after = match self {
            Self::CursorMode => self.split_buf(buf)?,
            Self::InsertMode { after } => after,
        };
        if after.len() == 0 {
            return Ok(Self::InsertMode { after });
        }
        let (_, after) = after.split_at(1);
        execute!(
            w,
            style::Print(after),
            style::Print(' '),
            cursor::MoveLeft(after.len() as u16 + 1),
        )?;
        Ok(Self::InsertMode { after: String::from(after) })
    }

    pub fn cursor_mode(self, buf: &mut String) -> io::Result<EditorState> {
        match self {
            Self::CursorMode => {}
            Self::InsertMode { after } => {
                buf.push_str(&after);
            }
        }
        Ok(Self::CursorMode)
    }
}

fn typing_synced<W>(w: &mut W, buf: &mut String) -> io::Result<KeyCode>
where W: io::Write,
{
    w.execute(cursor::MoveToColumn(buf.len() as u16))?;
    let mut state = EditorState::CursorMode;
    loop {
        state = match read_char()? {
            KeyCode::Char(c) => state.print(w, buf, c)?,
            KeyCode::Backspace => state.erase_left(w, buf)?,
            KeyCode::Delete => state.erase_right(w, buf)?,
            KeyCode::Left => {
                w.execute(cursor::MoveLeft(1))?;
                state.cursor_mode(buf)?
            }
            KeyCode::Right => {
                let new_state = state.cursor_mode(buf)?;
                if (cursor::position()?.0 as usize) < buf.len() {
                    w.execute(cursor::MoveRight(1))?;
                }
                new_state
            }
            KeyCode::Up => return Ok(KeyCode::Up),
            KeyCode::Down => return Ok(KeyCode::Down),
            KeyCode::Enter => return Ok(KeyCode::Enter),
            KeyCode::Esc => return Ok(KeyCode::Esc),
            _ => continue,
        };
    }
}

fn run<W>(w: &mut W) -> io::Result<()>
where W: io::Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    execute!(
        w,
        style::ResetColor,
        terminal::Clear(terminal::ClearType::All),
        cursor::SetCursorStyle::BlinkingBar,
        cursor::MoveTo(1, 1),
    )?;

    let mut buffer = vec![String::new()];
    let mut i = 0;

    loop {
        match typing_synced(w, &mut buffer[i])? {
            KeyCode::Esc => break,
            KeyCode::Up => {
                if i > 0 {
                    w.execute(cursor::MoveToPreviousLine(1))?;
                    i -= 1;
                }
            }
            KeyCode::Down => {
                if i < buffer.len() - 1 {
                    w.execute(cursor::MoveToNextLine(1))?;
                    i += 1;
                }
            }
            KeyCode::Enter => {
                if i == buffer.len() - 1 {
                    buffer.push(String::new());
                    w.execute(cursor::MoveToNextLine(1))?;
                    i += 1;
                }
            }
            _ => {}
        };
    }

    execute!(
        w,
        style::ResetColor,
        terminal::LeaveAlternateScreen,
        cursor::SetCursorStyle::DefaultUserShape,
    )?;

    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn read_char() -> io::Result<KeyCode> {
    use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
    loop {
        if let Ok(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        })) = read() {
            return Ok(code);
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)
}
