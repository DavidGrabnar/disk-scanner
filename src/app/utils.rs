use std::fmt::{Display, Formatter, Write};

pub enum Size {
    Base,
    Kilo,
    Mega,
    Giga,
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Size::Base => "",
            Size::Kilo => "k",
            Size::Mega => "M",
            Size::Giga => "G",
        })
    }
}

pub struct Bytes {
    base: f32,
    size: Size,
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:.1} {}B", self.base, self.size).as_str())
    }
}

impl Bytes {
    pub fn new(value: u64) -> Bytes {
        let mut base: f32 = value as f32;
        let mut size = Size::Base;
        if base > 1024.0 {
            base /= 1024.0;
            size = Size::Kilo;
        }
        if base > 1024.0 {
            base /= 1024.0;
            size = Size::Mega;
        }
        if base > 1024.0 {
            base /= 1024.0;
            size = Size::Giga;
        }
        Self { base, size }
    }
}
