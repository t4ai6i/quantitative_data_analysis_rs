use num_traits::{Float, FromPrimitive, PrimInt, ToPrimitive};

pub fn min<T: Float + FromPrimitive>(v: &[T]) -> T {
    v.iter().copied().fold(T::infinity(), T::min)
}

pub fn max<T: Float + FromPrimitive>(v: &[T]) -> T {
    v.iter().copied().fold(T::neg_infinity(), T::max)
}

pub fn percentage<T, U>(dividend: T, divisor: T) -> U
where
    T: PrimInt + ToPrimitive,
    U: Float + FromPrimitive,
{
    let dividend = U::from(dividend).unwrap_or(U::zero());
    let divisor = U::from(divisor).unwrap_or(U::zero());
    if dividend.is_nan() || divisor.is_nan() || divisor.is_zero() {
        U::zero()
    } else {
        dividend / divisor * U::from_f64(100.0).unwrap_or(U::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_test() {
        let v = vec![1.0, 2.0, 3.0];
        let result = min(&v);
        assert_eq!(result, 1.0);

        let v = vec![-1.0, -2.0, -3.0];
        let result = min(&v);
        assert_eq!(result, -3.0);

        let v: Vec<f64> = vec![];
        let result = min(&v);
        assert_eq!(result, f64::INFINITY);
    }

    #[test]
    fn max_test() {
        let v = vec![1.0, 2.0, 3.0];
        let result = max(&v);
        assert_eq!(result, 3.0);

        let v = vec![-1.0, -2.0, -3.0];
        let result = max(&v);
        assert_eq!(result, -1.0);

        let v: Vec<f64> = vec![];
        let result = max(&v);
        assert_eq!(result, f64::NEG_INFINITY);
    }

    #[test]
    fn percentage_test() {
        let dividend = 50;
        let divisor = 200;
        let result: f64 = percentage(dividend, divisor);
        assert_eq!(result, 25.0);

        let dividend = 0;
        let divisor = 0;
        let result: f64 = percentage(dividend, divisor);
        assert_eq!(result, 0.0);

        let dividend = 50;
        let divisor = 0;
        let result: f64 = percentage(dividend, divisor);
        assert_eq!(result, 0.0);
    }
}
