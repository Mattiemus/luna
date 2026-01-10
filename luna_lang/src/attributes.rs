use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Attributes: u32 {
        const HOLD_ALL     = 0b0001;
        const HOLD_FIRST   = 0b0010;
        const HOLD_REST    = 0b0100;
        const PROTECTED    = 0b1000;

        const FLAT         = 0b0001_0000;
        const ORDERLESS    = 0b0010_0000;
        const ONE_IDENTITY = 0b0100_0000;
        const LOCKED       = 0b1000_0000;

        const SEQUENCE_HOLD    = 0b0001_0000_0000;
        const LISTABLE         = 0b0010_0000_0000;
        const NUMERIC_FUNCTION = 0b0100_0000_0000;



        /// Symbol is read only and cannot be changed.
        const READ_ONLY = 0b0001_0000_0000_0000;

        /// Attributes of the symbol cannot be changed.
        const ATTRIBUTES_READ_ONLY = 0b0010_0000_0000_0000;
    }
}
