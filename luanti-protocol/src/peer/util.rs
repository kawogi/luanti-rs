// Luanti uses 16-bit sequence numbers that wrap around.
// To simplify reasoning about sequence numbers, translate
// them into 64-bit unique ids.
pub(crate) fn rel_to_abs(base: u64, seqnum: u16) -> u64 {
    let delta = relative_distance(base as u16, seqnum);
    ((base as i64) + delta) as u64
}

/// Determine the distance from sequence number a to b.
/// Sequence numbers are modulo 65536, so this is the
/// unique value d in the range -32768 < d <= 32768
/// with: a + d = b (mod 65536)
#[expect(clippy::min_ident_chars, reason = "names are generic on purpose")]
pub(crate) fn relative_distance(a: u16, b: u16) -> i64 {
    let distance: u16 = (std::num::Wrapping(b) - std::num::Wrapping(a)).0;
    if distance <= 0x8000 {
        distance as i64
    } else {
        (distance as i64) - 0x0001_0000
    }
}
