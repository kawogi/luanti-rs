use std::ops::Add;

const SEQUENCE_NUMBER_INITIAL: u16 = 0xffdc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct SequenceNumber(u64);

impl SequenceNumber {
    pub(crate) const fn init() -> Self {
        Self(SEQUENCE_NUMBER_INITIAL as u64)
    }

    /// Apply the (shortened) sequence number to this one by replacing the lower bits.
    ///
    /// Depending on the resulting delta, decide whether the sequence number shall be increased or
    /// decreased and adjust the high bits accordingly.
    pub(crate) fn goto(self, partial_sequence_number: u16) -> Self {
        let distance = partial_sequence_number.wrapping_sub(self.partial());
        // TODO(kawogi) this is an unusual boundary; if 0x7fff was used, this could be solved with a simple cast to i16
        let is_overflow = distance > 0x8000;
        let result = self.0 + u64::from(distance) - if is_overflow { 0x0001_0000 } else { 0 };

        Self(result)
    }

    #[inline]
    pub(crate) fn partial(self) -> u16 {
        self.0 as u16
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

    use super::*;

    #[test]
    fn init() {
        assert_eq!(SEQUENCE_NUMBER_INITIAL, SequenceNumber::init().partial());
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

    // #[test]
    // fn test_relative_distance() {
    //     use super::SequenceNumber::relative_distance;

    //     assert_eq!(0, relative_distance(0x0000, 0x0000));
    //     assert_eq!(0, relative_distance(0x7fff, 0x7fff));
    //     assert_eq!(0, relative_distance(0x8000, 0x8000));
    //     assert_eq!(0, relative_distance(0xffff, 0xffff));

    //     assert_eq!(1, relative_distance(0x0000, 0x0001));
    //     assert_eq!(1, relative_distance(0xfffe, 0xffff));
    //     assert_eq!(1, relative_distance(0xffff, 0x0000));
    //     assert_eq!(1, relative_distance(0x7fff, 0x8000));
    //     assert_eq!(-1, relative_distance(0x0000, 0xffff));
    //     assert_eq!(i64::from(i16::MIN), relative_distance(0x0000, 0x8000));
    //     assert_eq!(i64::from(i16::MAX), relative_distance(0x0000, 0x7fff));
    //     assert_eq!(i64::from(i16::MIN), relative_distance(0x8000, 0x0000));
    //     assert_eq!(-i64::from(i16::MAX), relative_distance(0x7fff, 0x0000));
    //     assert_eq!(i64::from(i16::MIN), relative_distance(0xffff, 0x7fff));
    //     assert_eq!(-i64::from(i16::MAX), relative_distance(0xffff, 0x8000));
    // }
}
