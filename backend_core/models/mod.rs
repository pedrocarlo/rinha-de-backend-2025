pub mod backend;
pub mod processor;

use std::ops::{Deref, DerefMut};

use chrono::{DateTime, SubsecRound, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct RoundedDateTime(pub DateTime<Utc>);

impl Deref for RoundedDateTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RoundedDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for RoundedDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rounded = self.0.round_subsecs(6);
        rounded.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RoundedDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dt = DateTime::<Utc>::deserialize(deserializer)?;
        Ok(RoundedDateTime(dt.round_subsecs(6)))
    }
}

impl From<DateTime<Utc>> for RoundedDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}
