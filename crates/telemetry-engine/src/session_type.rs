use serde::{Deserialize, Serialize};

/// Session type enum representing different iRacing session modes.
/// Maps iRacing's session type strings to normalized internal types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    #[default]
    Practice,
    Qualifying,
    Race,
    Warmup,
    Testing,
}

impl SessionType {
    /// Map an iRacing session type string to the SessionType enum.
    ///
    /// # Arguments
    /// * `raw` - The raw session type string from IRSDK session info
    ///
    /// # Returns
    /// The mapped SessionType. Defaults to Practice for unrecognized values.
    ///
    /// # Examples
    /// ```
    /// use telemetry_engine::SessionType;
    ///
    /// assert_eq!(SessionType::from_irsdk("Practice"), SessionType::Practice);
    /// assert_eq!(SessionType::from_irsdk("Qualify"), SessionType::Qualifying);
    /// assert_eq!(SessionType::from_irsdk("Lone Qualify"), SessionType::Qualifying);
    /// assert_eq!(SessionType::from_irsdk("Race"), SessionType::Race);
    /// assert_eq!(SessionType::from_irsdk("UnknownType"), SessionType::Practice);
    /// ```
    pub fn from_irsdk(raw: &str) -> Self {
        match raw {
            "Practice" => SessionType::Practice,
            "Qualify" | "Lone Qualify" | "Open Qualify" => SessionType::Qualifying,
            "Race" => SessionType::Race,
            "Warmup" => SessionType::Warmup,
            "Testing" => SessionType::Testing,
            _ => {
                tracing::warn!(
                    "Unrecognized session type '{}', defaulting to Practice",
                    raw
                );
                SessionType::Practice
            }
        }
    }

    /// Get the display label for the session type (for UI rendering).
    ///
    /// # Returns
    /// A human-readable display string.
    ///
    /// # Examples
    /// ```
    /// use telemetry_engine::SessionType;
    ///
    /// assert_eq!(SessionType::Practice.display_label(), "Practice");
    /// assert_eq!(SessionType::Qualifying.display_label(), "Qualifying");
    /// assert_eq!(SessionType::Race.display_label(), "Race");
    /// ```
    pub fn display_label(&self) -> &str {
        match self {
            SessionType::Practice => "Practice",
            SessionType::Qualifying => "Qualifying",
            SessionType::Race => "Race",
            SessionType::Warmup => "Warmup",
            SessionType::Testing => "Testing",
        }
    }

    /// Convert the session type to a lowercase string for database storage.
    /// This matches the serde serialization format.
    ///
    /// # Examples
    /// ```
    /// use telemetry_engine::SessionType;
    ///
    /// assert_eq!(SessionType::Practice.as_str(), "practice");
    /// assert_eq!(SessionType::Qualifying.as_str(), "qualifying");
    /// assert_eq!(SessionType::Race.as_str(), "race");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            SessionType::Practice => "practice",
            SessionType::Qualifying => "qualifying",
            SessionType::Race => "race",
            SessionType::Warmup => "warmup",
            SessionType::Testing => "testing",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_irsdk_practice() {
        assert_eq!(SessionType::from_irsdk("Practice"), SessionType::Practice);
    }

    #[test]
    fn test_from_irsdk_race() {
        assert_eq!(SessionType::from_irsdk("Race"), SessionType::Race);
    }

    #[test]
    fn test_from_irsdk_qualify() {
        assert_eq!(SessionType::from_irsdk("Qualify"), SessionType::Qualifying);
    }

    #[test]
    fn test_from_irsdk_lone_qualify() {
        assert_eq!(
            SessionType::from_irsdk("Lone Qualify"),
            SessionType::Qualifying
        );
    }

    #[test]
    fn test_from_irsdk_open_qualify() {
        assert_eq!(
            SessionType::from_irsdk("Open Qualify"),
            SessionType::Qualifying
        );
    }

    #[test]
    fn test_from_irsdk_warmup() {
        assert_eq!(SessionType::from_irsdk("Warmup"), SessionType::Warmup);
    }

    #[test]
    fn test_from_irsdk_testing() {
        assert_eq!(SessionType::from_irsdk("Testing"), SessionType::Testing);
    }

    #[test]
    fn test_from_irsdk_unknown() {
        // Should default to Practice for unknown values
        assert_eq!(
            SessionType::from_irsdk("UnknownType"),
            SessionType::Practice
        );
        assert_eq!(SessionType::from_irsdk(""), SessionType::Practice);
        assert_eq!(
            SessionType::from_irsdk("SomeOtherSessionType"),
            SessionType::Practice
        );
    }

    #[test]
    fn test_display_label() {
        assert_eq!(SessionType::Practice.display_label(), "Practice");
        assert_eq!(SessionType::Qualifying.display_label(), "Qualifying");
        assert_eq!(SessionType::Race.display_label(), "Race");
        assert_eq!(SessionType::Warmup.display_label(), "Warmup");
        assert_eq!(SessionType::Testing.display_label(), "Testing");
    }

    #[test]
    fn test_as_str() {
        assert_eq!(SessionType::Practice.as_str(), "practice");
        assert_eq!(SessionType::Qualifying.as_str(), "qualifying");
        assert_eq!(SessionType::Race.as_str(), "race");
        assert_eq!(SessionType::Warmup.as_str(), "warmup");
        assert_eq!(SessionType::Testing.as_str(), "testing");
    }

    #[test]
    fn test_serialization() {
        let session_type = SessionType::Qualifying;
        let json = serde_json::to_string(&session_type).unwrap();
        assert_eq!(json, "\"qualifying\"");

        let deserialized: SessionType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SessionType::Qualifying);
    }

    #[test]
    fn test_all_variants_serialize_as_lowercase() {
        assert_eq!(
            serde_json::to_string(&SessionType::Practice).unwrap(),
            "\"practice\""
        );
        assert_eq!(
            serde_json::to_string(&SessionType::Qualifying).unwrap(),
            "\"qualifying\""
        );
        assert_eq!(
            serde_json::to_string(&SessionType::Race).unwrap(),
            "\"race\""
        );
        assert_eq!(
            serde_json::to_string(&SessionType::Warmup).unwrap(),
            "\"warmup\""
        );
        assert_eq!(
            serde_json::to_string(&SessionType::Testing).unwrap(),
            "\"testing\""
        );
    }

    #[test]
    fn test_default() {
        let default_type: SessionType = Default::default();
        assert_eq!(default_type, SessionType::Practice);
    }
}
