use num_traits::{Float, FromPrimitive};

pub fn min<T: Float + FromPrimitive>(v: &[T]) -> T {
    v.iter().copied().fold(T::infinity(), T::min)
}

pub fn max<T: Float + FromPrimitive>(v: &[T]) -> T {
    v.iter().copied().fold(T::neg_infinity(), T::max)
}
