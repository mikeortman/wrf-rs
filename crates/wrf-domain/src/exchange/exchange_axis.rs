/// Horizontal axis exchanged during one WRF halo phase.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExchangeAxis {
    /// South-north halos, exchanged first by WRF.
    SouthNorth,
    /// West-east halos, exchanged after south-north corners are available.
    WestEast,
}
