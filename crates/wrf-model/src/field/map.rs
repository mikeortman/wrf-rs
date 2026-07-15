use std::fmt;

/// Registry-backed horizontal map factors and terrain height.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum ArwMapField {
    /// West-east velocity X map factor `msfux`.
    WestEastVelocityX,
    /// West-east velocity Y map factor `msfuy`.
    WestEastVelocityY,
    /// South-north velocity X map factor `msfvx`.
    SouthNorthVelocityX,
    /// Inverse south-north velocity X map factor `msfvx_inv`.
    InverseSouthNorthVelocityX,
    /// South-north velocity Y map factor `msfvy`.
    SouthNorthVelocityY,
    /// Mass-point X map factor `msftx`.
    MassPointX,
    /// Mass-point Y map factor `msfty`.
    MassPointY,
    /// Terrain height `ht`.
    TerrainHeight,
}

impl ArwMapField {
    pub(crate) const COUNT: usize = 8;
    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::WestEastVelocityX,
        Self::WestEastVelocityY,
        Self::SouthNorthVelocityX,
        Self::InverseSouthNorthVelocityX,
        Self::SouthNorthVelocityY,
        Self::MassPointX,
        Self::MassPointY,
        Self::TerrainHeight,
    ];

    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::WestEastVelocityX => "msfux",
            Self::WestEastVelocityY => "msfuy",
            Self::SouthNorthVelocityX => "msfvx",
            Self::InverseSouthNorthVelocityX => "msfvx_inv",
            Self::SouthNorthVelocityY => "msfvy",
            Self::MassPointX => "msftx",
            Self::MassPointY => "msfty",
            Self::TerrainHeight => "ht",
        }
    }
}

impl fmt::Display for ArwMapField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.registry_name())
    }
}
