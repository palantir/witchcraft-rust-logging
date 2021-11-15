// Copyright 2019 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use serde::de;
use serde::de::{DeserializeSeed, EnumAccess, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

static LOG_LEVEL_NAMES: [&str; 7] = ["OFF", "FATAL", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

/// The verbosity level of a log record.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// The "fatal" level.
    ///
    /// Designates an extremely serious error.
    // make the discriminants match up with LevelFilter's
    Fatal = 1,
    /// The "error" level.
    ///
    /// Designates an error.
    Error,
    /// The "warn" level.
    ///
    /// Designates a potentially concerning event.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}

impl FromStr for Level {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Level, FromStrError> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|level| s.eq_ignore_ascii_case(level))
            .and_then(Level::from_usize)
            .ok_or(FromStrError(()))
    }
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }

    #[inline]
    fn lt(&self, other: &LevelFilter) -> bool {
        (*self as usize) < (*other as usize)
    }

    #[inline]
    fn le(&self, other: &LevelFilter) -> bool {
        (*self as usize) <= (*other as usize)
    }

    #[inline]
    fn gt(&self, other: &LevelFilter) -> bool {
        (*self as usize) > (*other as usize)
    }

    #[inline]
    fn ge(&self, other: &LevelFilter) -> bool {
        (*self as usize) >= (*other as usize)
    }
}

impl Level {
    fn from_usize(n: usize) -> Option<Level> {
        match n {
            1 => Some(Level::Fatal),
            2 => Some(Level::Error),
            3 => Some(Level::Warn),
            4 => Some(Level::Info),
            5 => Some(Level::Debug),
            6 => Some(Level::Trace),
            _ => None,
        }
    }

    /// Returns the standard string name of the level.
    pub fn as_str(self) -> &'static str {
        LOG_LEVEL_NAMES[self as usize]
    }
}

/// A filter for log record verbosity levels.
///
/// The variants match `Level`'s, with the addition of `Off`, which filters out all log messages.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)] // we need to store this as a usize in MAX_LOG_LEVEL_FILTER
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Fatal` log level.
    Fatal,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl fmt::Display for LevelFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}

impl FromStr for LevelFilter {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<LevelFilter, FromStrError> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|level| s.eq_ignore_ascii_case(level))
            .and_then(LevelFilter::from_usize)
            .ok_or(FromStrError(()))
    }
}

impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd<Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }

    #[inline]
    fn lt(&self, other: &Level) -> bool {
        (*self as usize) < (*other as usize)
    }

    #[inline]
    fn le(&self, other: &Level) -> bool {
        (*self as usize) <= (*other as usize)
    }

    #[inline]
    fn gt(&self, other: &Level) -> bool {
        (*self as usize) > (*other as usize)
    }

    #[inline]
    fn ge(&self, other: &Level) -> bool {
        (*self as usize) >= (*other as usize)
    }
}

impl LevelFilter {
    fn from_usize(n: usize) -> Option<LevelFilter> {
        match n {
            0 => Some(LevelFilter::Off),
            1 => Some(LevelFilter::Fatal),
            2 => Some(LevelFilter::Error),
            3 => Some(LevelFilter::Warn),
            4 => Some(LevelFilter::Info),
            5 => Some(LevelFilter::Debug),
            6 => Some(LevelFilter::Trace),
            _ => None,
        }
    }
}

/// An error parsing a `Level` or `LevelFilter` from a string.
#[derive(Debug)]
pub struct FromStrError(());

impl fmt::Display for FromStrError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("invalid log level")
    }
}

impl Error for FromStrError {}

// The Deserialize impls are handwritten to be case insensitive using FromStr.
impl Serialize for Level {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Level::Fatal => serializer.serialize_unit_variant("Level", 0, "FATAL"),
            Level::Error => serializer.serialize_unit_variant("Level", 1, "ERROR"),
            Level::Warn => serializer.serialize_unit_variant("Level", 2, "WARN"),
            Level::Info => serializer.serialize_unit_variant("Level", 3, "INFO"),
            Level::Debug => serializer.serialize_unit_variant("Level", 4, "DEBUG"),
            Level::Trace => serializer.serialize_unit_variant("Level", 5, "TRACE"),
        }
    }
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LevelIdentifier;

        impl<'de> Visitor<'de> for LevelIdentifier {
            type Value = Level;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("log level")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Case insensitive.
                FromStr::from_str(s)
                    .map_err(|_| de::Error::unknown_variant(s, &LOG_LEVEL_NAMES[1..]))
            }
        }

        impl<'de> DeserializeSeed<'de> for LevelIdentifier {
            type Value = Level;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(LevelIdentifier)
            }
        }

        struct LevelEnum;

        impl<'de> Visitor<'de> for LevelEnum {
            type Value = Level;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("log level")
            }

            fn visit_enum<A>(self, value: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (level, variant) = value.variant_seed(LevelIdentifier)?;
                // Every variant is a unit variant.
                variant.unit_variant()?;
                Ok(level)
            }
        }

        deserializer.deserialize_enum("Level", &LOG_LEVEL_NAMES[1..], LevelEnum)
    }
}

impl Serialize for LevelFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            LevelFilter::Off => serializer.serialize_unit_variant("LevelFilter", 0, "OFF"),
            LevelFilter::Fatal => serializer.serialize_unit_variant("LevelFilter", 1, "FATAL"),
            LevelFilter::Error => serializer.serialize_unit_variant("LevelFilter", 2, "ERROR"),
            LevelFilter::Warn => serializer.serialize_unit_variant("LevelFilter", 3, "WARN"),
            LevelFilter::Info => serializer.serialize_unit_variant("LevelFilter", 4, "INFO"),
            LevelFilter::Debug => serializer.serialize_unit_variant("LevelFilter", 5, "DEBUG"),
            LevelFilter::Trace => serializer.serialize_unit_variant("LevelFilter", 6, "TRACE"),
        }
    }
}

impl<'de> Deserialize<'de> for LevelFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LevelFilterIdentifier;

        impl<'de> Visitor<'de> for LevelFilterIdentifier {
            type Value = LevelFilter;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("log level filter")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Case insensitive.
                FromStr::from_str(s).map_err(|_| de::Error::unknown_variant(s, &LOG_LEVEL_NAMES))
            }
        }

        impl<'de> DeserializeSeed<'de> for LevelFilterIdentifier {
            type Value = LevelFilter;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(LevelFilterIdentifier)
            }
        }

        struct LevelFilterEnum;

        impl<'de> Visitor<'de> for LevelFilterEnum {
            type Value = LevelFilter;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("log level filter")
            }

            fn visit_enum<A>(self, value: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (level_filter, variant) = value.variant_seed(LevelFilterIdentifier)?;
                // Every variant is a unit variant.
                variant.unit_variant()?;
                Ok(level_filter)
            }
        }

        deserializer.deserialize_enum("LevelFilter", &LOG_LEVEL_NAMES, LevelFilterEnum)
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    use crate::{Level, LevelFilter};

    fn level_token(variant: &'static str) -> Token {
        Token::UnitVariant {
            name: "Level",
            variant,
        }
    }

    fn level_filter_token(variant: &'static str) -> Token {
        Token::UnitVariant {
            name: "LevelFilter",
            variant,
        }
    }

    #[test]
    fn test_level_ser_de() {
        let cases = [
            (Level::Fatal, [level_token("FATAL")]),
            (Level::Error, [level_token("ERROR")]),
            (Level::Warn, [level_token("WARN")]),
            (Level::Info, [level_token("INFO")]),
            (Level::Debug, [level_token("DEBUG")]),
            (Level::Trace, [level_token("TRACE")]),
        ];

        for &(s, expected) in &cases {
            assert_tokens(&s, &expected);
        }
    }

    #[test]
    fn test_level_case_insensitive() {
        let cases = [
            (Level::Fatal, [level_token("fatal")]),
            (Level::Error, [level_token("error")]),
            (Level::Warn, [level_token("warn")]),
            (Level::Info, [level_token("info")]),
            (Level::Debug, [level_token("debug")]),
            (Level::Trace, [level_token("trace")]),
        ];

        for &(s, expected) in &cases {
            assert_de_tokens(&s, &expected);
        }
    }

    #[test]
    fn test_level_de_error() {
        let msg = "unknown variant `errorx`, expected one of \
                   `FATAL`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`";
        assert_de_tokens_error::<Level>(&[level_token("errorx")], msg);
    }

    #[test]
    fn test_level_filter_ser_de() {
        let cases = [
            (LevelFilter::Off, [level_filter_token("OFF")]),
            (LevelFilter::Fatal, [level_filter_token("FATAL")]),
            (LevelFilter::Error, [level_filter_token("ERROR")]),
            (LevelFilter::Warn, [level_filter_token("WARN")]),
            (LevelFilter::Info, [level_filter_token("INFO")]),
            (LevelFilter::Debug, [level_filter_token("DEBUG")]),
            (LevelFilter::Trace, [level_filter_token("TRACE")]),
        ];

        for &(s, expected) in &cases {
            assert_tokens(&s, &expected);
        }
    }

    #[test]
    fn test_level_filter_case_insensitive() {
        let cases = [
            (LevelFilter::Off, [level_filter_token("off")]),
            (LevelFilter::Fatal, [level_filter_token("fatal")]),
            (LevelFilter::Error, [level_filter_token("error")]),
            (LevelFilter::Warn, [level_filter_token("warn")]),
            (LevelFilter::Info, [level_filter_token("info")]),
            (LevelFilter::Debug, [level_filter_token("debug")]),
            (LevelFilter::Trace, [level_filter_token("trace")]),
        ];

        for &(s, expected) in &cases {
            assert_de_tokens(&s, &expected);
        }
    }

    #[test]
    fn test_level_filter_de_error() {
        let msg = "unknown variant `errorx`, expected one of \
                   `OFF`, `FATAL`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`";
        assert_de_tokens_error::<LevelFilter>(&[level_filter_token("errorx")], msg);
    }
}
