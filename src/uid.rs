use std::{fmt, num::NonZeroU32};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Uid(NonZeroU32);

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}""#, self.0)
    }
}

impl Uid {
    pub const fn raw(self) -> NonZeroU32 {
        self.0
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
