use std::time::Duration;

use crate::app::types::Timestamp;
use crate::util::bit::bits;
use crate::util::bit::BitMask;
use crate::util::bit::Bitfield;

/// Enumeration modeling two stables states and an in-transit state
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DoubleBit {
    /// Transitioning between end conditions
    Intermediate,
    /// Determined to be OFF
    DeterminedOff,
    /// Determined to be ON
    DeterminedOn,
    /// Abnormal or custom condition
    Indeterminate,
}

/// A DNP3 time value that may be Synchronized or NotSynchronized
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Time {
    /// The timestamp is UTC synchronized at the remote device
    Synchronized(Timestamp),
    /// The device indicates the timestamp may be not be synchronized
    NotSynchronized(Timestamp),
}

impl Time {
    /// test if the `Time` is synchronized
    pub fn is_synchronized(&self) -> bool {
        std::matches!(self, Self::Synchronized(_))
    }

    /// created a synchronized `Time` from a u64
    pub fn synchronized(ts: u64) -> Time {
        Self::Synchronized(Timestamp::new(ts))
    }

    /// created a unsynchronized `Time` from a u64
    pub fn not_synchronized(ts: u64) -> Time {
        Self::NotSynchronized(Timestamp::new(ts))
    }
}

/// Flags as defined in the specification where each bit has a type-specific meaning
///
/// Not every bit is used for every type (Binary, Analog, etc). Users
/// should refer to the standard to determine what flag values
/// correspond to each type.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Flags {
    /// underlying bitmask
    pub value: u8,
}

impl Flags {
    /// Object value is 'good' / 'valid' / 'nominal'
    pub const ONLINE: Flags = Flags::new(bits::BIT_0.value);
    /// Object value has not been updated since device restart
    pub const RESTART: Flags = Flags::new(bits::BIT_1.value);
    /// Object value represents the last value available before a communication failure occurred
    pub const COMM_LOST: Flags = Flags::new(bits::BIT_2.value);
    /// Object value is overridden in a downstream reporting device
    pub const REMOTE_FORCED: Flags = Flags::new(bits::BIT_3.value);
    /// object value is overridden by the device reporting this flag
    pub const LOCAL_FORCED: Flags = Flags::new(bits::BIT_4.value);
    /// Object value is changing state rapidly (device dependent meaning)
    pub const CHATTER_FILTER: Flags = Flags::new(bits::BIT_5.value);
    /// Object value exceeds the measurement range of the reported variation
    pub const OVER_RANGE: Flags = Flags::new(bits::BIT_5.value);
    /// reported counter value cannot be compared against a prior value to obtain the correct count difference
    pub const DISCONTINUITY: Flags = Flags::new(bits::BIT_6.value);
    /// Object value might not have the expected level of accuracy
    pub const REFERENCE_ERR: Flags = Flags::new(bits::BIT_6.value);

    /// Create a `Flags` struct from a `u8` bitmask
    pub const fn new(value: u8) -> Self {
        Self { value }
    }

    /// Return true if all of the flags in 'other' are set in this Flags
    pub fn is_set(&self, other: Flags) -> bool {
        (self.value & other.value) == other.value
    }
}

pub(crate) trait ToVariation<V> {
    fn to_variation(&self) -> V;
}

pub(crate) trait WireFlags {
    fn get_wire_flags(&self) -> u8;
}

impl From<Option<Time>> for Time {
    fn from(x: Option<Time>) -> Self {
        x.unwrap_or_else(|| Time::NotSynchronized(Timestamp::new(0)))
    }
}

impl From<Option<Time>> for Timestamp {
    fn from(x: Option<Time>) -> Self {
        Time::from(x).timestamp()
    }
}

impl Time {
    pub(crate) fn checked_add(self, x: u16) -> Option<Self> {
        match self {
            Time::Synchronized(ts) => ts
                .checked_add(Duration::from_millis(x as u64))
                .map(Time::Synchronized),
            Time::NotSynchronized(ts) => ts
                .checked_add(Duration::from_millis(x as u64))
                .map(Time::NotSynchronized),
        }
    }

    /// extract a `Timestamp` from a `Time` discarding synchronization information
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Time::Synchronized(ts) => *ts,
            Time::NotSynchronized(ts) => *ts,
        }
    }
}

/// Measurement type corresponding to groups 1 and 2
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Binary {
    /// value of the type
    pub value: bool,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl Binary {
    /// construct a `Binary` from its fields
    pub fn new(value: bool, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Measurement type corresponding to groups 3 and 4
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DoubleBitBinary {
    /// value of the type
    pub value: DoubleBit,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl DoubleBitBinary {
    /// construct a `DoubleBitBinary` from its fields
    pub fn new(value: DoubleBit, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Measurement type corresponding to groups 10 and 11
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BinaryOutputStatus {
    /// value of the type
    pub value: bool,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl BinaryOutputStatus {
    /// construct a `BinaryOutputStatus` from its fields
    pub fn new(value: bool, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Measurement type corresponding to groups 20 and 22
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Counter {
    /// value of the type
    pub value: u32,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl Counter {
    /// construct a `Counter` from its fields
    pub fn new(value: u32, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Measurement type corresponding to groups 21 and 23
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FrozenCounter {
    /// value of the type
    pub value: u32,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl FrozenCounter {
    /// construct a `FrozenCounter` from its fields
    pub fn new(value: u32, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Measurement type corresponding to groups 30 and 32
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Analog {
    /// value of the type
    pub value: f64,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl Analog {
    /// construct an `Analog` from its fields
    pub fn new(value: f64, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

impl std::ops::BitOr<Flags> for Flags {
    type Output = Flags;

    fn bitor(self, rhs: Flags) -> Self::Output {
        Flags::new(self.value | rhs.value)
    }
}

impl std::ops::BitOrAssign<Flags> for Flags {
    fn bitor_assign(&mut self, rhs: Flags) {
        self.value |= rhs.value
    }
}

// some crate only helpers
impl Flags {
    /// test a `Flags` struct to see if the `STATE` bit is set
    pub(crate) fn state(self) -> bool {
        self.value.bit_7()
    }

    /// extract the `DoubleBit` value from a flags struct
    pub(crate) fn double_bit_state(self) -> DoubleBit {
        DoubleBit::from(self.value.bit_7(), self.value.bit_6())
    }

    pub(crate) fn with_bits_set_to(&self, mask: BitMask, value: bool) -> Flags {
        if value {
            self.with_bits_set(mask)
        } else {
            self.with_bits_cleared(mask)
        }
    }

    pub(crate) fn with_bits_cleared(&self, mask: BitMask) -> Flags {
        Flags::new(self.value & !mask.value)
    }

    pub(crate) fn with_bits_set(&self, mask: BitMask) -> Flags {
        Flags::new(self.value | mask.value)
    }

    pub(crate) fn without(&self, mask: BitMask) -> Flags {
        Flags::new(self.value & !mask.value)
    }
}

struct FlagFormatter {
    prev: bool,
}

impl FlagFormatter {
    fn new() -> Self {
        Self { prev: false }
    }

    fn push(
        &mut self,
        is_set: bool,
        text: &'static str,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        if is_set {
            if self.prev {
                f.write_str(", ")?;
            }
            self.prev = true;
            f.write_str(text)?;
        }
        Ok(())
    }

    fn begin(flags: Flags, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:02X} [", flags.value)
    }

    fn end(f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("]")
    }

    fn format_binary_flags_0_to_4(
        &mut self,
        flags: Flags,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        self.push(flags.is_set(Flags::ONLINE), "ONLINE", f)?;
        self.push(flags.is_set(Flags::RESTART), "RESTART", f)?;
        self.push(flags.is_set(Flags::COMM_LOST), "COMM_LOST", f)?;
        self.push(flags.is_set(Flags::REMOTE_FORCED), "REMOTE_FORCED", f)?;
        self.push(flags.is_set(Flags::LOCAL_FORCED), "LOCAL_FORCED", f)?;
        Ok(())
    }

    fn format_binary_flags_0_to_5(
        &mut self,
        flags: Flags,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        self.format_binary_flags_0_to_4(flags, f)?;
        self.push(flags.is_set(Flags::CHATTER_FILTER), "CHATTER_FILTER", f)?;
        Ok(())
    }

    fn push_debug_item<T>(
        &mut self,
        name: &'static str,
        item: T,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result
    where
        T: std::fmt::Debug,
    {
        if self.prev {
            f.write_str(", ")?;
        }
        self.prev = true;
        write!(f, "{} = {:?}", name, item)
    }
}

pub(crate) struct BinaryFlagFormatter {
    flags: Flags,
}

impl BinaryFlagFormatter {
    pub(crate) fn new(value: u8) -> Self {
        Self {
            flags: Flags::new(value),
        }
    }
}

impl std::fmt::Display for BinaryFlagFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut formatter = FlagFormatter::new();
        FlagFormatter::begin(self.flags, f)?;
        formatter.format_binary_flags_0_to_5(self.flags, f)?;
        formatter.push(self.flags.value.bit_6(), "RESERVED(6)", f)?;
        formatter.push(self.flags.value.bit_7(), "STATE", f)?;
        FlagFormatter::end(f)
    }
}

pub(crate) struct DoubleBitBinaryFlagFormatter {
    flags: Flags,
}

impl DoubleBitBinaryFlagFormatter {
    pub(crate) fn new(value: u8) -> Self {
        Self {
            flags: Flags::new(value),
        }
    }
}

impl std::fmt::Display for DoubleBitBinaryFlagFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut formatter = FlagFormatter::new();
        FlagFormatter::begin(self.flags, f)?;
        formatter.format_binary_flags_0_to_5(self.flags, f)?;
        formatter.push_debug_item("state", self.flags.double_bit_state(), f)?;
        FlagFormatter::end(f)
    }
}

pub(crate) struct BinaryOutputStatusFlagFormatter {
    flags: Flags,
}

impl BinaryOutputStatusFlagFormatter {
    pub(crate) fn new(value: u8) -> Self {
        Self {
            flags: Flags::new(value),
        }
    }
}

impl std::fmt::Display for BinaryOutputStatusFlagFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut formatter = FlagFormatter::new();
        FlagFormatter::begin(self.flags, f)?;
        formatter.format_binary_flags_0_to_4(self.flags, f)?;
        formatter.push(self.flags.value.bit_5(), "RESERVED(5)", f)?;
        formatter.push(self.flags.value.bit_6(), "RESERVED(6)", f)?;
        formatter.push(self.flags.value.bit_7(), "STATE", f)?;
        FlagFormatter::end(f)
    }
}

pub(crate) struct CounterFlagFormatter {
    flags: Flags,
}

impl CounterFlagFormatter {
    pub(crate) fn new(value: u8) -> Self {
        Self {
            flags: Flags::new(value),
        }
    }
}

impl std::fmt::Display for CounterFlagFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut formatter = FlagFormatter::new();
        FlagFormatter::begin(self.flags, f)?;
        formatter.format_binary_flags_0_to_4(self.flags, f)?;
        formatter.push(self.flags.value.bit_5(), "ROLLOVER", f)?;
        formatter.push(self.flags.value.bit_6(), "DISCONTINUITY", f)?;
        formatter.push(self.flags.value.bit_7(), "RESERVED(7)", f)?;
        FlagFormatter::end(f)
    }
}

pub(crate) struct AnalogFlagFormatter {
    flags: Flags,
}

impl AnalogFlagFormatter {
    pub(crate) fn new(value: u8) -> Self {
        Self {
            flags: Flags::new(value),
        }
    }
}

impl std::fmt::Display for AnalogFlagFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut formatter = FlagFormatter::new();
        FlagFormatter::begin(self.flags, f)?;
        formatter.format_binary_flags_0_to_4(self.flags, f)?;
        formatter.push(self.flags.value.bit_5(), "OVER_RANGE", f)?;
        formatter.push(self.flags.value.bit_6(), "REFERENCE_ERR", f)?;
        formatter.push(self.flags.value.bit_7(), "RESERVED(7)", f)?;
        FlagFormatter::end(f)
    }
}

pub(crate) trait AnalogConversions {
    const OVER_RANGE: BitMask = bits::BIT_5;

    fn get_value(&self) -> f64;
    fn get_flags(&self) -> Flags;

    fn to_i16(&self) -> (Flags, i16) {
        if self.get_value() < i16::MIN.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), i16::MIN);
        }

        if self.get_value() > i16::MAX.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), i16::MAX);
        }

        (self.get_flags(), self.get_value() as i16)
    }

    fn to_i32(&self) -> (Flags, i32) {
        if self.get_value() < i32::MIN.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), i32::MIN);
        }

        if self.get_value() > i32::MAX.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), i32::MAX);
        }

        (self.get_flags(), self.get_value() as i32)
    }

    fn to_f32(&self) -> (Flags, f32) {
        if self.get_value() < f32::MIN.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), f32::MIN);
        }

        if self.get_value() > f32::MAX.into() {
            return (self.get_flags().with_bits_set(Self::OVER_RANGE), f32::MAX);
        }

        (self.get_flags(), self.get_value() as f32)
    }
}

/// Measurement type corresponding to groups 40 and 42
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AnalogOutputStatus {
    /// value of the type
    pub value: f64,
    /// associated flags
    pub flags: Flags,
    /// associated time
    pub time: Option<Time>,
}

impl AnalogOutputStatus {
    /// construct an `AnalogOutputStatus` from its fields
    pub fn new(value: f64, flags: Flags, time: Time) -> Self {
        Self {
            value,
            flags,
            time: Some(time),
        }
    }
}

/// Octet string point type corresponding to groups 110 and 111
///
/// Octet strings can only hold from 1 to 255 octets. Zero-length
/// octet strings are prohibited by the standard.
///
/// The default value is `[0x00]`, corresponding to an empty
/// C-style string.
#[allow(missing_copy_implementations)]
#[derive(Clone, PartialEq, Debug)]
pub struct OctetString {
    value: [u8; Self::MAX_SIZE],
    len: u8,
}

#[allow(clippy::len_without_is_empty)]
impl OctetString {
    const MAX_SIZE: usize = 255;

    /// Creates a new octet string.
    ///
    /// The `value` parameter must have a length of [1, 255],
    /// otherwise it will return an error.
    pub fn new(value: &[u8]) -> Result<Self, OctetStringError> {
        let len = value.len();
        if len == 0 {
            return Err(OctetStringError::ZeroLength);
        }

        if len > 255 {
            return Err(OctetStringError::MoreThan255Octets);
        }

        let mut result = Self {
            value: [0u8; 255],
            len: len as u8,
        };
        result.value[..len].copy_from_slice(value);
        Ok(result)
    }

    /// Returns the value of the octet string
    pub fn value(&self) -> &[u8] {
        &self.value[..self.len() as usize]
    }

    /// Returns the length of the octet string
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Allocates a new slice with the exact size of the string
    /// and copies the content to it.
    pub(crate) fn as_boxed_slice(&self) -> Box<[u8]> {
        self.value().into()
    }
}

/// Errors when creating an octet string
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OctetStringError {
    /// Zero-length octet strings are explicitely disallowed
    /// by the standard.
    ZeroLength,
    /// Octet strings can only hold up to 255 octets.
    MoreThan255Octets,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octet_string_methods() {
        let octet_string = OctetString::new(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(5, octet_string.len());
        assert_eq!(&[0, 1, 2, 3, 4], octet_string.value());
        assert_eq!(&[0, 1, 2, 3, 4], &*octet_string.as_boxed_slice());
    }

    #[test]
    fn new_octet_string_zero_length() {
        assert_eq!(Err(OctetStringError::ZeroLength), OctetString::new(&[]));
    }

    #[test]
    fn new_octet_string_greater_size() {
        assert_eq!(
            Err(OctetStringError::MoreThan255Octets),
            OctetString::new(&[0; 500])
        );
    }

    #[test]
    fn octet_string_default_value() {
        assert_eq!(&[0x00], OctetString::default().value());
    }

    #[test]
    fn flag_bit_or_works() {
        let flags = Flags::ONLINE | Flags::LOCAL_FORCED;
        assert_eq!(flags.value, 0b0001_0001);
    }

    #[test]
    fn flag_bit_or_assign_works() {
        let mut flags = Flags::ONLINE;
        flags |= Flags::LOCAL_FORCED;
        assert_eq!(flags.value, 0b0001_0001);
    }

    #[test]
    fn formats_binary_flags() {
        assert_eq!(format!("{}", BinaryFlagFormatter::new(0)), "0x00 []");
        assert_eq!(
            format!("{}", BinaryFlagFormatter::new(0b1100_0001)),
            "0xC1 [ONLINE, RESERVED(6), STATE]"
        );
    }

    #[test]
    fn formats_double_flags() {
        assert_eq!(
            format!("{}", DoubleBitBinaryFlagFormatter::new(0)),
            "0x00 [state = Intermediate]"
        );
        assert_eq!(
            format!("{}", DoubleBitBinaryFlagFormatter::new(0b1100_0001)),
            "0xC1 [ONLINE, state = Indeterminate]"
        );
    }
}
