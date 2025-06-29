use jsonschema::paths::{Location, LocationSegment};
use serde_json::Value;

/// Extension trait to get the parent of a JSON pointer.
pub trait LocationExtensions: Sized {
    /// Return the pointers parent there is one.
    fn parent(&self) -> Option<Self>;

    /// Return the last segment.
    fn last(&self) -> Option<LocationSegment<'_>>;

    /// Returns what the location is pointing at, this is either the final property or an index on a property.
    fn pointing_at(&self) -> String;

    /// Try reconstruct the value
    fn reconstruct(&self, value: &Value) -> String;
}

impl LocationExtensions for Location {
    fn parent(&self) -> Option<Self> {
        let mut segments: Vec<_> = self.into_iter().collect();
        if segments.is_empty() {
            return None;
        }
        segments.pop();
        Some(Self::from_iter(segments))
    }

    fn pointing_at(&self) -> String {
        let Some(last) = self.last() else {
            return "[root]".to_string();
        };

        match last {
            LocationSegment::Property(property) => property.to_string(),
            LocationSegment::Index(index) => {
                if let Some(parent) = self.parent() {
                    format!("{}[{}]", parent.pointing_at(), index)
                } else {
                    format!("[{index}]")
                }
            }
        }
    }

    fn last(&self) -> Option<LocationSegment<'_>> {
        self.into_iter().last()
    }

    fn reconstruct(&self, value: &Value) -> String {
        let value = serde_json::to_string_pretty(&value).unwrap_or_default();

        let key = match self.last() {
            Some(LocationSegment::Property(property)) => format!("\"{property}\": "),
            Some(LocationSegment::Index(_)) => "".to_string(),
            None => "".to_string(),
        };

        format!("{key}{value}")
    }
}
