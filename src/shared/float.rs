use num_traits::{Float, FromPrimitive, PrimInt, ToPrimitive};
use rayon::prelude::*;

fn reduce_with_operation<T: Float + Send + Sync, F>(v: &[T], op: F, identity: T) -> T
where
    F: Fn(T, T) -> T + Send + Sync,
{
    if v.is_empty() {
        return T::nan(); // 空のベクタの場合はNaNを返す
    }

    let filtered = v
        .par_iter() // 並列イテレーションを開始
        .cloned() // `par_iter` では参照を扱うため値に変換
        .filter(|x| !x.is_nan())
        .collect::<Vec<T>>(); // NaN を除外
    if filtered.is_empty() {
        return T::nan(); // 要素がすべてNaNで空のベクタになった場合、NaNを返す
    }
    filtered
        .par_iter() // 再度並列イテレーションを開始
        .cloned() // `par_iter` では参照を扱うため値に変換
        .reduce(|| identity, op) // 並列化された要素を統合しながら計算
}

pub fn min<T: Float + Send + Sync>(v: &[T]) -> T {
    reduce_with_operation(v, T::min, T::max_value())
}

pub fn max<T: Float + Send + Sync>(v: &[T]) -> T {
    reduce_with_operation(v, T::max, T::min_value())
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

// NaN または無限大の値を検出する補助関数
pub fn validate_value<T: Float>(value: T) -> Option<T> {
    if value.is_nan() || value.is_infinite() {
        None
    } else {
        Some(value)
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

        let v = vec![1.0];
        let result = min(&v);
        assert_eq!(result, 1.0);

        let v: Vec<f64> = vec![];
        let result = min(&v);
        assert!(result.is_nan());

        let v = vec![f64::NAN, 1.0];
        let result = min(&v);
        assert_eq!(result, 1.0);

        let v = vec![f64::NAN];
        let result = min(&v);
        assert!(result.is_nan());
    }

    #[test]
    fn max_test() {
        let v = vec![1.0, 2.0, 3.0];
        let result = max(&v);
        assert_eq!(result, 3.0);

        let v = vec![-1.0, -2.0, -3.0];
        let result = max(&v);
        assert_eq!(result, -1.0);

        let v = vec![1.0];
        let result = max(&v);
        assert_eq!(result, 1.0);

        let v: Vec<f64> = vec![];
        let result = max(&v);
        assert!(result.is_nan());

        let v = vec![f64::NAN, 1.0];
        let result = max(&v);
        assert_eq!(result, 1.0);

        let v = vec![f64::NAN];
        let result = max(&v);
        assert!(result.is_nan());
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

    #[test]
    fn validate_value_test() {
        let value = 1.0;
        let result = validate_value(value);
        assert_eq!(result, Some(1.0));

        let value = f64::NAN;
        let result = validate_value(value);
        assert_eq!(result, None);

        let value = f64::INFINITY;
        let result = validate_value(value);
        assert_eq!(result, None);

        let value = f64::NEG_INFINITY;
        let result = validate_value(value);
        assert_eq!(result, None);
    }
}
