mod threshold;

pub use threshold::Threshold;

use crate::Level;

#[derive(Debug, Clone)]
pub struct Filter {
    global: Threshold,
    targets: Vec<(String, Threshold)>,
}

impl Filter {
    pub fn new(global: Threshold) -> Self {
        Self {
            global,
            targets: Vec::new(),
        }
    }

    // Directives are comma separated. One without `=` sets the global threshold;
    // one with `=` sets a threshold for every target whose name starts with the
    // prefix on the left. A string that names no global keeps Info, which is what
    // a consumer means by a directive list that only silences dependencies.
    pub fn from_directives(text: &str) -> Self {
        let mut filter = Self::new(Threshold::At(Level::Info));

        for directive in text.split(',').map(str::trim).filter(|d| !d.is_empty()) {
            match directive.split_once('=') {
                None => match Threshold::parse(directive) {
                    Some(threshold) => filter.set_global(threshold),
                    None => Self::report(directive),
                },
                Some((prefix, level)) => match Threshold::parse(level) {
                    Some(threshold) => filter.add_target(prefix, threshold),
                    None => Self::report(directive),
                },
            }
        }

        filter
    }

    pub fn set_global(&mut self, threshold: Threshold) {
        self.global = threshold;
    }

    // Sorted longest prefix first, so a specific module overrides a broad crate
    // rule whatever order the directives were written in.
    pub fn add_target(&mut self, prefix: impl Into<String>, threshold: Threshold) {
        self.targets.push((prefix.into(), threshold));
        self.targets
            .sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));
    }

    pub fn admits(&self, target: &str, level: Level) -> bool {
        self.targets
            .iter()
            .find(|(prefix, _)| target.starts_with(prefix.as_str()))
            .map(|(_, threshold)| *threshold)
            .unwrap_or(self.global)
            .admits(level)
    }

    // A dropped directive reads as a filter that does not work, so it is named
    // rather than swallowed. stderr, because a logger cannot log its own setup.
    fn report(directive: &str) {
        eprintln!("curia: ignoring unparseable log directive `{directive}`");
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new(Threshold::At(Level::Trace))
    }
}
