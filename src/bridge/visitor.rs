use tracing_core::field::{Field, Visit};

use crate::Fields;

// tracing's Visit has no JSON case, so a nested value arrives through
// record_debug and is stored as its Debug string. That limit applies to the
// tracing input path only, not to our own macros or the plugin.

// Third-party spans instrument fields with whatever they hold, and some of those
// Debug renderings run to thousands of lines — a Tauri AppHandle is the usual
// offender. Uncapped, one such field lands in every event inside that span and
// drowns the log. Truncation keeps the signal without the flood.
const MAX_DEBUG_LEN: usize = 512;
#[derive(Default)]
pub struct FieldVisitor {
    pub fields: Fields,
    pub message: Option<String>,
}

impl Visit for FieldVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name(), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(field.name(), value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = Self::truncate(format!("{value:?}"));
        if field.name() == "message" {
            self.message = Some(rendered);
        } else {
            self.fields.insert(field.name(), rendered);
        }
    }
}

impl FieldVisitor {
    fn truncate(rendered: String) -> String {
        if rendered.len() <= MAX_DEBUG_LEN {
            return rendered;
        }

        // Cut on a char boundary; a Debug rendering may hold multi-byte text
        let mut end = MAX_DEBUG_LEN;
        while end > 0 && !rendered.is_char_boundary(end) {
            end -= 1;
        }

        format!(
            "{}… [{} more bytes truncated]",
            &rendered[..end],
            rendered.len() - end
        )
    }
}
