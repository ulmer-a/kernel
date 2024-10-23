use core::{
    fmt::{Display, Formatter, Result},
    marker::PhantomData,
};

pub struct Fmt<T> {
    length: u64,
    phantom: PhantomData<T>,
}

impl<T> From<u64> for Fmt<T> {
    fn from(value: u64) -> Self {
        Self {
            length: value,
            phantom: PhantomData,
        }
    }
}

impl<T> Display for Fmt<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.length {
            b if b < 10 * 1024 => write!(f, "{b} B"),
            kb if kb < 10 * 1024u64.pow(2) => write!(f, "{} KiB", kb >> 10),
            mb if mb < 10 * 1024u64.pow(3) => write!(f, "{} MiB", mb >> 20),
            gb => write!(f, "{} GiB", gb >> 30),
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::Fmt;

//     fn fmt_test() {
//         assert_eq!(format!("{}", Fmt::from(4823)), "4823 B");
//     }
// }
