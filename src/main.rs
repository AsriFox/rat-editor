use std::io;

use crossterm::{
    execute,
    style, terminal, cursor,
    event::KeyCode,
    ExecutableCommand,
};

fn typing_synced<W>(w: &mut W, buf: &mut String) -> io::Result<KeyCode>
where W: io::Write,
{
    w.execute(cursor::MoveToColumn(buf.len() as u16))?;
    loop {
        match read_char()? {
            KeyCode::Char(c) => {
                buf.push(c);
                w.execute(style::Print(c))?;
            }
            KeyCode::Backspace => {
                buf.pop();
                execute!(
                    w,
                    cursor::MoveLeft(1),
                    style::Print(' '),
                    cursor::MoveLeft(1),
                )?;
            }
            KeyCode::Up => return Ok(KeyCode::Up),
            KeyCode::Down => return Ok(KeyCode::Down),
            KeyCode::Enter => return Ok(KeyCode::Enter),
            KeyCode::Esc => return Ok(KeyCode::Esc),
            _ => {}
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
