// Copyright 2026 Palantir Technologies, Inc.
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
//! Logic to extract fields for logging from a JWT.
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use conjure_object::Uuid;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

/// Represents the parsed form of a JWT but does not verify the token signature.
///
/// The information provided by this struct should not be used for any security-sensitive
/// application unless verified through some other process (e.g. by querying another
/// service known to perform validation).
///
/// An anticipated use of this type is making a best-effort extraction of its fields for
/// logging.
#[derive(PartialEq, Eq, Debug, Deserialize, Clone)]
pub struct UnverifiedJwt {
    #[serde(deserialize_with = "de_uuid")]
    sub: Uuid,
    #[serde(default, deserialize_with = "de_opt_uuid")]
    sid: Option<Uuid>,
    #[serde(default, deserialize_with = "de_opt_uuid")]
    jti: Option<Uuid>,
    #[serde(default, deserialize_with = "de_opt_uuid")]
    org: Option<Uuid>,
}

impl UnverifiedJwt {
    /// Returns the unverified user id, i.e., the "sub" claim, of the JWT.
    pub fn unverified_user_id(&self) -> Uuid {
        self.sub
    }

    /// Returns the unverified session id, i.e. the "sid" claim, of the JWT
    /// or absent if the JWT does not contain the "sid" claim.
    pub fn unverified_session_id(&self) -> Option<Uuid> {
        self.sid
    }

    /// Returns the unverified token id, i.e. the "jti" claim, of the JWT
    /// or absent if the JWT does not contain the "jti" claim.
    pub fn unverified_token_id(&self) -> Option<Uuid> {
        self.jti
    }

    /// Returns the unverified organization id, i.e. the "org" claim, of the JWT
    /// or absent if the JWT does not contain the "org" claim.
    pub fn unverified_organization_id(&self) -> Option<Uuid> {
        self.org
    }

    /// Attempts to create an [`UnverifiedJwt`] from a provided bearer token.
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.split('.').skip(1);
        let payload = it.next()?;
        if it.count() != 1 {
            return None;
        }

        let payload = URL_SAFE_NO_PAD.decode(payload).ok()?;

        serde_json::from_slice(&payload).ok()
    }
}

// To save space, we serialize UUIDs as base64 bytes rather than the normal hex format.
fn de_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl Visitor<'_> for V {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("base64 encoded UUID")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let bytes = STANDARD
                .decode(v)
                .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))?;

            Uuid::from_slice(&bytes).map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_str(V)
}

fn de_opt_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl<'de2> Visitor<'de2> for V {
        type Value = Option<Uuid>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("option")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de2>,
        {
            de_uuid(deserializer).map(Some)
        }
    }

    deserializer.deserialize_option(V)
}

#[cfg(test)]
mod test {
    use crate::jwt::UnverifiedJwt;

    #[test]
    fn parse() {
        let token = "header.\
            eyJzdWIiOiJ3NVAyV1FNQlEwNnB5WEl3U2xCLy9BPT0iLCJzaWQiOiJQOFpqMUQ1SVRlMjZUdGVLK1l1RFl3PT0\
            iLCJqdGkiOiJwRm0wb1ZDSlQrQ0dWZFhmMmJLMy9RPT0iLCJvcmciOiJGQlMycTgvbFQvMnNBRktxZ09pUW13PT\
            0iLCJleHAiOiAxNTc3ODY1NjAwfQ\
            .signature";

        let parsed = UnverifiedJwt::parse(token).unwrap();

        let expected = UnverifiedJwt {
            sub: "c393f659-0301-434e-a9c9-72304a507ffc".parse().unwrap(),
            sid: Some("3fc663d4-3e48-4ded-ba4e-d78af98b8363".parse().unwrap()),
            jti: Some("a459b4a1-5089-4fe0-8655-d5dfd9b2b7fd".parse().unwrap()),
            org: Some("1414b6ab-cfe5-4ffd-ac00-52aa80e8909b".parse().unwrap()),
        };

        assert_eq!(expected, parsed);
    }
}
