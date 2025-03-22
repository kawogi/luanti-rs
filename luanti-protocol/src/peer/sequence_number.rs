use std::ops::Add;

use crate::wire::sequence_number::WrappingSequenceNumber;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct SequenceNumber(u64);

impl SequenceNumber {
    pub(crate) fn init() -> Self {
        Self(u64::from(u16::from(WrappingSequenceNumber::INITIAL)))
    }

    /// Apply the (shortened) sequence number to this one by replacing the lower bits.
    ///
    /// Depending on the resulting delta, decide whether the sequence number shall be increased or
    /// decreased and adjust the high bits accordingly.
    pub(crate) fn goto(self, partial_sequence_number: impl Into<WrappingSequenceNumber>) -> Self {
        let partial_sequence_number = u16::from(partial_sequence_number.into());
        let distance = partial_sequence_number.wrapping_sub(self.0 as u16);
        // TODO(kawogi) this is an unusual boundary; if 0x7fff was used, this could be solved with a simple cast to i16
        let is_overflow = distance > 0x8000;
        let result = self.0 + u64::from(distance) - if is_overflow { 0x0001_0000 } else { 0 };

        Self(result)
    }

    #[inline]
    pub(crate) fn as_wrapping(self) -> WrappingSequenceNumber {
        (self.0 as u16).into()
    }

    pub(crate) const fn inc(&mut self) {
        self.0 += 1;
    }
}

impl Add<u16> for SequenceNumber {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + u64::from(rhs))
    }
}

#[cfg(test)]
mod tests {

    use crate::wire::sequence_number::WrappingSequenceNumber;

    use super::*;

    #[test]
    fn init() {
        assert_eq!(
            WrappingSequenceNumber::INITIAL,
            SequenceNumber::init().as_wrapping()
        );
    }

    #[test]
    fn goto() {
        assert_eq!(
            SequenceNumber(0x0001_0000),
            SequenceNumber(0x0001_0000).goto(0x0000)
        );
        assert_eq!(
            SequenceNumber(0x0000_ffff),
            SequenceNumber(0x0001_0000).goto(0xffff)
        );
        assert_eq!(
            SequenceNumber(0x0001_0001),
            SequenceNumber(0x0001_0000).goto(0x0001)
        );
        assert_eq!(
            SequenceNumber(0x0001_7fff),
            SequenceNumber(0x0001_0000).goto(0x7fff)
        );
        assert_eq!(
            SequenceNumber(0x0001_8000),
            SequenceNumber(0x0001_0000).goto(0x8000)
        );
        assert_eq!(
            SequenceNumber(0x0000_8001),
            SequenceNumber(0x0001_0000).goto(0x8001)
        );
    }
}
