/// Side of a destination patch receiving a halo.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExchangeDirection {
    /// Lower-index side: west or south.
    Lower,
    /// Upper-index side: east or north.
    Upper,
}
