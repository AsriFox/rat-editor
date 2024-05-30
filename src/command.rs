use crossterm::event::{KeyCode, KeyModifiers};

pub enum EditorCmd {
    MoveCursor(i16),
    JumpToStart,
    JumpToEnd,
    Scroll(isize),
    Newline,
    DeleteNewlineBefore,
    DeleteNewlineAfter,
    Exit,
}

impl EditorCmd {
    pub fn from_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Self> {
        match modifiers {
            KeyModifiers::CONTROL => match code {
                KeyCode::Char(c) => match c {
                    'q' => Some(Self::Exit),
                    _ => None,
                },
                KeyCode::Up => Some(Self::Scroll(-1)),
                KeyCode::Down => Some(Self::Scroll(1)),
                KeyCode::Home => Some(Self::JumpToStart),
                KeyCode::End => Some(Self::JumpToEnd),
                _ => None,
            },
            KeyModifiers::NONE => match code {
                KeyCode::Up => Some(Self::MoveCursor(-1)),
                KeyCode::Down => Some(Self::MoveCursor(1)),
                KeyCode::PageUp => {
                    let (_, h) = crossterm::terminal::size().unwrap();
                    Some(Self::Scroll(-(h as isize) / 2))
                }
                KeyCode::PageDown => {
                    let (_, h) = crossterm::terminal::size().unwrap();
                    Some(Self::Scroll((h as isize) / 2))
                }
                KeyCode::Enter => Some(Self::Newline),
                _ => None,
            },
            _ => None,
        }
    }
}
