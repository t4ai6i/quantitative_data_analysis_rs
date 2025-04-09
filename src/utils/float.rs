use num_traits::{Float, FromPrimitive, PrimInt, ToPrimitive};
use rayon::prelude::*;

pub fn min<T: Float + Send + Sync>(v: &[T]) -> T {
    if v.is_empty() {
        return T::nan(); // 空のベクタの場合はNaNを返す
    }
    v.par_iter() // 並列イテレーションを開始
        .cloned() // `par_iter` では参照を扱うため値に変換
        .filter(|&x| !x.is_nan()) // NaN を除外
        .reduce(|| T::max_value(), T::min) // 並列化された要素を統合しながら最小値を計算
}

pub fn max<T: Float + Send + Sync>(v: &[T]) -> T {
    if v.is_empty() {
        return T::nan(); // 空のベクタの場合はNaNを返す
    }
    v.par_iter() // 並列イテレーションを開始
        .cloned() // `par_iter` では参照を扱うため値に変換
        .filter(|&x| !x.is_nan()) // NaN を除外
        .reduce(|| T::min_value(), T::max) // 並列化された要素を統合しながら最小値を計算
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

        let v = vec![1.0];
        let result = min(&v);
        assert_eq!(result, 1.0);

        let v: Vec<f64> = vec![];
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
