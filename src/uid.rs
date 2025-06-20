use serde::Serialize;
use std::{fmt, num::NonZeroU32};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Uid(NonZeroU32);

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Uid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

pub struct Generator {
    counter: NonZeroU32,
}

impl Generator {
    pub const fn new_uid(&mut self) -> Uid {
        let counter = self.counter;
        self.counter = counter.checked_add(1).expect("ran out of UIDs");
        Uid(counter)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            counter: NonZeroU32::MIN,
        }
    }
}
