use std::io;

use crossterm::{
    execute,
    queue,
    style,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal,
    cursor,
    Command,
};

const TEXT: &str = r#"Interactive test

press 'q' to exit
"#;

fn run<W>(w: &mut W) -> io::Result<()>
where W: io::Write, {
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1),
        )?;

        for line in TEXT.split('\n') {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }
        
        w.flush()?;

        match read_char()? {
            'q' => {
                execute!(w, cursor::SetCursorStyle::DefaultUserShape).unwrap();
                break;
            }
            _ => {}
        };
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )?;

    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn read_char() -> io::Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        })) = event::read() {
            return Ok(c);
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)
}
