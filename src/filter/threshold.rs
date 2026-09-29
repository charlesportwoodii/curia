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

    // How many levels this threshold lets through: 0 for Off, 5 for Trace. A level is
    // admitted exactly when its rank is below this, which is what lets `Logger` hold a
    // threshold in one atomic and test it with one compare.
    pub(crate) fn admitted(&self) -> u8 {
        match self {
            Self::Off => 0,
            Self::At(ceiling) => ceiling.rank() + 1,
        }
    }

    // The more verbose of the two.
    pub fn widest(self, other: Self) -> Self {
        if other.admitted() > self.admitted() {
            other
        } else {
            self
        }
    }

    // The less verbose of the two.
    pub fn narrowest(self, other: Self) -> Self {
        if other.admitted() < self.admitted() {
            other
        } else {
            self
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
