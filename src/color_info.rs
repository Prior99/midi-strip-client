use std::fmt::{Debug, Result, Formatter};

pub struct ColorInfo {
    pub note: u8,
    pub velocity: u8,
}

impl Debug for ColorInfo {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ColorInfo {{ note: {}, velocity: {} }}", self.note, self.velocity)
    }
}
