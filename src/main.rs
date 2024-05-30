mod buffer;
pub mod command;

use std::io::{self, prelude::*};

use crossterm::{
    execute, queue,
    style, terminal, cursor,
    event::KeyCode,
    ExecutableCommand,
};

use command::EditorCmd;

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
            return Ok(Self::CursorMode);
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

fn typing_synced<W>(w: &mut W, buf: &mut String) -> io::Result<EditorCmd>
where W: io::Write,
{
    use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
    let mut state = EditorState::CursorMode;
    loop {
        let (code, modifiers) = if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            state: _,
        }) = read()? {
            (code, modifiers)
        } else {
            continue;
        };

        if let Some(c) = EditorCmd::from_key(code, modifiers) {
            let _ = state.cursor_mode(buf)?;
            return Ok(c);
        }

        let (col, _) = cursor::position()?;
        state = match code {
            KeyCode::Char(c) => state.print(w, buf, c)?,
            KeyCode::Backspace => {
                if col == 0 {
                    let _ = state.cursor_mode(buf)?;
                    return Ok(EditorCmd::DeleteNewlineBefore);
                }
                state.erase_left(w, buf)?
            }
            KeyCode::Delete => match state.erase_right(w, buf)? {
                EditorState::InsertMode { after } => EditorState::InsertMode { after },
                EditorState::CursorMode => return Ok(EditorCmd::DeleteNewlineAfter),
            }
            KeyCode::Left => {
                w.execute(cursor::MoveLeft(1))?;
                state.cursor_mode(buf)?
            }
            KeyCode::Right => {
                let new_state = state.cursor_mode(buf)?;
                if (col as usize) < buf.len() {
                    w.execute(cursor::MoveRight(1))?;
                }
                new_state
            }
            _ => continue,
        };
    }
}

fn run<W>(w: &mut W, lines: Vec<String>) -> io::Result<()>
where W: io::Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut buffer = buffer::Buffer::new(lines)?;

    queue!(
        w,
        style::ResetColor,
        cursor::SetCursorStyle::BlinkingBar,
    )?;
    buffer.queue_reprint(w)?;
    queue!(w, cursor::MoveTo(0, 0))?;
    w.flush()?;

    loop {
        let edcmd = typing_synced(w, buffer.get_line())?;
        buffer.save_cursor_pos()?;
        match edcmd {
            EditorCmd::MoveCursor(i) => buffer.move_cursor_v(w, i)?,
            EditorCmd::Scroll(i) => buffer.scroll(w, i)?,
            EditorCmd::Newline => buffer.newline(w)?,
            EditorCmd::DeleteNewlineBefore => buffer.delete_newline_before(w)?,
            EditorCmd::DeleteNewlineAfter => buffer.delete_newline_after(w)?,
            EditorCmd::Exit => break,
        }
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

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let lines = if args.len() >= 2 {
        let file = std::fs::File::open(&args[1])?;
        let lines: Vec<String> =
            std::io::BufReader::new(file)
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();
        if lines.len() == 0 {
            return Err(
                io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("File {} is empty", &args[1])
                )
            );
        }
        lines
    } else {
        vec![String::new()]
    };

    std::panic::set_hook(Box::new(|info| {
        eprintln!("{:?}", info);
        terminal::disable_raw_mode().unwrap();
    }));

    let mut stdout = io::stdout();
    run(&mut stdout, lines)
}

