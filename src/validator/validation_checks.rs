use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ValidationChecks: u32 {
        const Signature             = 1;
        const Permissions           = 2;
        const KeyUsage              = 4;
        const ValidityPeriod        = 8;
        const Timestamp             = 16;
        const All = Self::Signature.bits()
                  | Self::Permissions.bits()
                  | Self::KeyUsage.bits()
                  | Self::ValidityPeriod.bits()
                  | Self::Timestamp.bits();
    }
}
