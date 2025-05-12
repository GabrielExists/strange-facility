use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Amount {
    Gain(i64),
    Spend(i64),
    Catalyst(i64),
    GainX(i64),
    SpendX(i64),
    CatalystX(i64),
    Set(i64),
}

impl Amount {
    pub fn multiply(self, factor: i64) -> Self {
        match self {
            Amount::Gain(delta) => Amount::Gain(delta * factor),
            Amount::Spend(delta) => Amount::Spend(delta * factor),
            Amount::Catalyst(delta) => Amount::Catalyst(delta * factor),
            Amount::GainX(delta) => Amount::GainX(delta * factor),
            Amount::SpendX(delta) => Amount::SpendX(delta * factor),
            Amount::CatalystX(delta) => Amount::CatalystX(delta * factor),
            Amount::Set(target) => Amount::Set(target * factor),
        }
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Gain(number) => f.write_fmt(format_args!("+{}", *number)),
            Amount::Spend(number) => f.write_fmt(format_args!("-{}", *number)),
            Amount::Catalyst(number) => f.write_fmt(format_args!("require {}", *number)),
            Amount::GainX(number) => f.write_fmt(format_args!("+{}X", *number)),
            Amount::SpendX(number) => f.write_fmt(format_args!("-{}X", *number)),
            Amount::CatalystX(number) => f.write_fmt(format_args!("require {}X", *number)),
            Amount::Set(number) => f.write_fmt(format_args!("set to {}", *number)),
        }
    }
}
