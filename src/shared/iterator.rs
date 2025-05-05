use deref_derive::{Deref, DerefMut};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut)]
pub struct SliceWrapper<'a, T>(&'a [T]);

impl<'a, T> From<&'a [T]> for SliceWrapper<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self(value)
    }
}

pub trait FromEnd<T> {
    fn get_from_end(&self, reverse_index: isize) -> Vec<T>;
}

impl<'a, T> FromEnd<T> for SliceWrapper<'a, T>
where
    T: Copy,
{
    /// Specify reverse index and get Vec from end.
    ///
    /// # Arguments
    ///
    /// * `reverse_index`: isize
    ///
    /// returns: Vec<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use quantitative_data_analysis_rs::shared::iterator::{FromEnd, SliceWrapper}; // ここも変更
    ///
    /// let vec = vec![0, 1, 2, 3, 4];
    /// let wrapper = SliceWrapper::from(vec.as_slice());
    ///
    /// let actual = wrapper.get_from_end(0);
    /// assert_eq!(actual, Vec::<i32>::new());
    ///
    /// let actual = wrapper.get_from_end(-2);
    /// assert_eq!(actual, vec![3, 4]);
    ///
    /// let actual = wrapper.get_from_end(2);
    /// assert_eq!(actual, vec![3, 4]);
    ///
    /// ```
    fn get_from_end(&self, reverse_index: isize) -> Vec<T> {
        // スライスの長さから reverse_index の絶対値を引く。
        // 結果が負になる場合は 0 に飽和させる (saturating)。
        let index = self.0.len().saturating_sub(reverse_index.unsigned_abs());
        // 計算された index から末尾までの要素を収集する。
        self.0[index..].iter().cloned().collect_vec()
    }
}

#[cfg(test)]
mod tests {
    // 使用する構造体名を変更
    use crate::shared::iterator::{FromEnd, SliceWrapper};

    #[test]
    fn from_end_test() {
        let vec = vec![0, 1, 2, 3, 4];
        let slice_wrapper = SliceWrapper::from(vec.as_slice());

        let actual = slice_wrapper.get_from_end(0);
        assert_eq!(actual, Vec::<i32>::new());

        let actual = slice_wrapper.get_from_end(-1);
        assert_eq!(actual, vec![4]);

        let actual = slice_wrapper.get_from_end(-2);
        assert_eq!(actual, vec![3, 4]);

        let actual = slice_wrapper.get_from_end(-4);
        assert_eq!(actual, vec![1, 2, 3, 4]);

        let actual = slice_wrapper.get_from_end(-5);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = slice_wrapper.get_from_end(-6);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = slice_wrapper.get_from_end(6);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = slice_wrapper.get_from_end(5);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = slice_wrapper.get_from_end(1);
        assert_eq!(actual, vec![4]);
    }

    #[test]
    fn from_end_empty_slice_test() {
        let vec: Vec<i32> = vec![]; // 空のVec
        let slice_wrapper = SliceWrapper::from(vec.as_slice());

        // reverse_index が 0 の場合
        let actual = slice_wrapper.get_from_end(0);
        assert_eq!(actual, Vec::<i32>::new());

        // reverse_index が正の値の場合
        let actual = slice_wrapper.get_from_end(1);
        assert_eq!(actual, Vec::<i32>::new());

        // reverse_index が負の値の場合
        let actual = slice_wrapper.get_from_end(-1);
        assert_eq!(actual, Vec::<i32>::new());
    }
}
