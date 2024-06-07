use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub enum EditorCmd {
    MoveCursor(i16),
    JumpToStart,
    JumpToEnd,
    Scroll(isize),
    Resize(u16, u16),
    Newline,
    DeleteNewlineBefore,
    DeleteNewlineAfter,
    Save,
    Exit,
}

impl EditorCmd {
    pub fn from(event: Event) -> Option<Self> {
        match event {
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                modifiers,
                state: _,
            }) => Self::from_key(code, modifiers),
            Event::Resize(w, h) => Some(Self::Resize(w, h)),
            _ => None,
        }
    }

    pub fn from_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Self> {
        match modifiers {
            KeyModifiers::CONTROL => match code {
                KeyCode::Char(c) => match c {
                    's' => Some(Self::Save),
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
