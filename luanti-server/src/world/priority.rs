use glam::{I16Vec3, Vec3};
use luanti_core::{MapBlockPos, MapNodePos};

/// Represents a value describing how important something (e.g. a map block) is to the player.
///
/// This value may be used as key for a priority queue, where higher values mean higher priority.
///
/// Priorities are coarse-grained and a conversion from float values is supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct Priority(u16);

impl Priority {
    /// This is the maximum achievable priority
    pub(crate) const MAX: Self = Self(u16::MAX);

    /// This is the minimum achievable priority (before being `NONE`)
    pub(crate) const MIN: Self = Self(Self::NONE.0 + 1);

    /// This value means that the associated object shall no longer being considered at all.
    ///
    /// The meaning is equivalent to `Option::<Priority>::None`, but without requiring an extra byte
    /// and it automatically causes `Ord` to be implemented correctly.
    pub(crate) const NONE: Self = Self(u16::MIN);

    pub(crate) fn is_none(self) -> bool {
        self == Self::NONE
    }

    #[expect(dead_code, reason = "might come in handy later")]
    pub(crate) fn is_some(self) -> bool {
        !self.is_none()
    }

    /// Uses the euclidean distance between two positions as priority.
    /// Distances exceeding `max_distance` will be mapped to `Priority::NONE`. Set the limit to
    /// `u32::MAX` to disable this.
    /// Distances exceeding the limit of `u16` will be clamped to a priority of `Priority::MIN`.
    pub(crate) fn from_vec_distance(pos_a: I16Vec3, pos_b: I16Vec3, max_distance: u32) -> Self {
        // TODO(kawogi) find a more performant solution; a very coarse approximation would be sufficient
        // note: do not use `distance_squared` because the decimation for lower distances will cause
        // all low distances to be mapped to `Priority::MAX`
        let distance = Vec3::distance(pos_a.as_vec3(), pos_b.as_vec3());
        #[expect(
            clippy::cast_precision_loss,
            reason = "the expected range is precise enough"
        )]
        if distance < max_distance as f32 {
            #[expect(clippy::cast_possible_truncation, reason = "truncation is on purpose")]
            #[expect(clippy::cast_sign_loss, reason = "distance is always positive")]
            Priority(Self::MAX.0.saturating_sub(distance as u16).max(Self::MIN.0))
        } else {
            Priority::NONE
        }
    }

    pub(crate) fn from_node_distance(
        pos_a: MapNodePos,
        pos_b: MapNodePos,
        max_distance: u32,
    ) -> Self {
        Self::from_vec_distance(pos_a.into(), pos_b.into(), max_distance)
    }

    pub(crate) fn from_block_distance(
        pos_a: MapBlockPos,
        pos_b: MapBlockPos,
        max_distance: u32,
    ) -> Self {
        Self::from_node_distance(pos_a.into(), pos_b.into(), max_distance)
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::NONE
    }
}

/// Converts float values in the range of `0.0..=1.0` to priorities `MIN..=MAX`.
///
/// Smaller numbers are being interpreted as lower priority, with 0.0 being the lowest and 1.0 being
/// the highest.
/// Negative values are being clamped to 0.0.
/// `NAN` is mapped to `Self::NONE`.
impl From<f32> for Priority {
    fn from(value: f32) -> Self {
        if value.is_nan() {
            Self::NONE
        } else {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the value is clamped to be in range"
            )]
            #[expect(
                clippy::cast_sign_loss,
                reason = "the value is clamped to be non-negative"
            )]
            Self(
                (value.clamp(0.0, 1.0) * f32::from(Self::MAX.0 - Self::MIN.0)).round() as u16
                    + Self::MIN.0,
            )
        }
    }
}
