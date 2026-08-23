use crate::Level;

// `Off` is a threshold, not a Level. Adding an Off variant to Level would change
// the meaning of `event.level <= sink.level()`, which every Sink implementation
// depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Threshold {
    Off,
    At(Level),
}

impl Threshold {
    pub fn admits(&self, level: Level) -> bool {
        match self {
            Self::Off => false,
            Self::At(ceiling) => level <= *ceiling,
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "error" => Some(Self::At(Level::Error)),
            "warn" => Some(Self::At(Level::Warn)),
            "info" => Some(Self::At(Level::Info)),
            "debug" => Some(Self::At(Level::Debug)),
            "trace" => Some(Self::At(Level::Trace)),
            _ => None,
        }
    }
}
