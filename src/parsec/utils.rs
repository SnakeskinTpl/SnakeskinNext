use std::ops::{Bound, RangeBounds};

/// Returns inclusive count bounds, or None for an empty or unrepresentable range.
pub(super) fn count_bounds(range: impl RangeBounds<usize>) -> Option<(usize, usize)> {
    let min = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&value) => value,
        Bound::Excluded(&value) => value.checked_add(1)?,
    };

    let max = match range.end_bound() {
        Bound::Unbounded => usize::MAX,
        Bound::Included(&value) => value,
        Bound::Excluded(&value) => value.checked_sub(1)?,
    };

    (min <= max).then_some((min, max))
}
