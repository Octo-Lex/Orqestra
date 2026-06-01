//! Duration newtype — stores minutes as `u32`.
//!
//! YAML frontmatter uses human-readable strings like `"8h"`, `"30m"`, `"1h30m"`.
//! JSON (and the TypeScript contract) expects a plain integer (`number | null`).
//!
//! Custom serde:
//! - **Deserialize (YAML):** Accepts strings (`"8h"`, `"30m"`, `"1h30m"`) and bare integers (`480`).
//! - **Serialize (JSON):** Emits a plain `u32` integer.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Duration in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub u32);

impl Duration {
    /// Return the number of minutes.
    pub fn minutes(self) -> u32 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Serialization: always a plain integer
// ---------------------------------------------------------------------------

impl Serialize for Duration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0)
    }
}

// ---------------------------------------------------------------------------
// Deserialization: accept string patterns OR bare integers
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, Visitor};

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    fmt,
                    "a duration string like \"8h\", \"30m\", \"1h30m\" or a bare integer (minutes)"
                )
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Duration, E> {
                Ok(Duration(v as u32))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Duration, E> {
                if v < 0 {
                    return Err(E::custom("duration cannot be negative"));
                }
                Ok(Duration(v as u32))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Duration, E> {
                parse_duration_string(v).map_err(|msg| E::custom(msg))
            }
        }

        deserializer.deserialize_any(DurationVisitor)
    }
}

/// Parse duration strings: `"8h"`, `"30m"`, `"1h30m"`, `"1d4h"`.
///
/// Supported units: `d` (days = 480 min), `h` (hours = 60 min), `m` (minutes = 1 min).
fn parse_duration_string(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".into());
    }

    // If it's purely numeric, treat as minutes
    if let Ok(minutes) = s.parse::<u32>() {
        return Ok(Duration(minutes));
    }

    let mut total_minutes: u32 = 0;
    let mut remaining = s;
    let mut parsed_something = false;

    while !remaining.is_empty() {
        // Find the next unit suffix
        let (number_part, rest) = if let Some(pos) = remaining.find(|c: char| c.is_alphabetic()) {
            let (num, r) = remaining.split_at(pos);
            // Find where the digits end in rest (the unit might be multi-char)
            let unit_end = r.find(|c: char| c.is_ascii_digit()).unwrap_or(r.len());
            let (unit, next) = r.split_at(unit_end);
            (num, (unit, next))
        } else {
            break;
        };

        let value: u32 = number_part
            .parse()
            .map_err(|_| format!("invalid number '{}' in duration '{}'", number_part, s))?;

        let multiplier = match rest.0 {
            "d" => 480,
            "h" => 60,
            "m" => 1,
            other => return Err(format!("unknown unit '{}' in duration '{}'", other, s)),
        };

        total_minutes += value * multiplier;
        parsed_something = true;
        remaining = rest.1;
    }

    if !parsed_something {
        return Err(format!("could not parse duration '{}'", s));
    }

    Ok(Duration(total_minutes))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hours() {
        assert_eq!(parse_duration_string("8h").unwrap(), Duration(480));
    }

    #[test]
    fn parse_single_hour() {
        assert_eq!(parse_duration_string("1h").unwrap(), Duration(60));
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(parse_duration_string("30m").unwrap(), Duration(30));
    }

    #[test]
    fn parse_hours_minutes() {
        assert_eq!(parse_duration_string("1h30m").unwrap(), Duration(90));
    }

    #[test]
    fn parse_days() {
        assert_eq!(parse_duration_string("1d").unwrap(), Duration(480));
    }

    #[test]
    fn parse_days_hours() {
        assert_eq!(parse_duration_string("2d4h").unwrap(), Duration(1200));
    }

    #[test]
    fn parse_bare_integer_from_yaml() {
        // serde_yaml deserializes a bare YAML integer to visit_u64
        let dur: Duration = serde_yaml::from_str("120").unwrap();
        assert_eq!(dur, Duration(120));
    }

    #[test]
    fn parse_string_hours_from_yaml() {
        let dur: Duration = serde_yaml::from_str("\"8h\"").unwrap();
        assert_eq!(dur, Duration(480));
    }

    #[test]
    fn serialize_as_plain_integer() {
        let dur = Duration(480);
        let json = serde_json::to_string(&dur).unwrap();
        assert_eq!(json, "480");
    }

    #[test]
    fn serialize_null_duration() {
        let opt: Option<Duration> = None;
        let json = serde_json::to_string(&opt).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn reject_negative() {
        assert!(parse_duration_string("-5h").is_err());
    }

    #[test]
    fn reject_unknown_unit() {
        assert!(parse_duration_string("5x").is_err());
    }

    #[test]
    fn reject_empty() {
        assert!(parse_duration_string("").is_err());
    }
}
