use itertools::Itertools;

pub trait FromEnd<T> {
    fn from_end(&self, reverse_index: isize) -> Vec<T>;
}

pub struct VecT<'a, T>(pub &'a [T]);

impl<'a, T> FromEnd<T> for VecT<'a, T>
where
    T: Copy,
{
    /// Specify reverse index and get Vec
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
    /// use quantitative_data_analysis_rs::utils::iterator::{FromEnd, VecT};
    /// let vec = vec![0, 1, 2, 3, 4];
    /// let vec = VecT(vec.as_slice());
    /// let actual = vec.from_end(0);
    /// assert_eq!(actual, Vec::<i32>::new());
    /// let actual = vec.from_end(-2);
    /// assert_eq!(actual, vec![3, 4]);
    /// let actual = vec.from_end(2);
    /// assert_eq!(actual, vec![3, 4]);
    ///
    /// ```
    fn from_end(&self, reverse_index: isize) -> Vec<T> {
        let index = self.0.len().checked_sub(reverse_index.unsigned_abs());
        let index = index.unwrap_or_default();
        let range = index..;
        self.0[range].iter().cloned().collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::iterator::{FromEnd, VecT};

    #[test]
    fn from_end_test() {
        let vec = vec![0, 1, 2, 3, 4];
        let vec = VecT(vec.as_slice());

        let actual = vec.from_end(0);
        assert_eq!(actual, Vec::<i32>::new());

        let actual = vec.from_end(-1);
        assert_eq!(actual, vec![4]);

        let actual = vec.from_end(-2);
        assert_eq!(actual, vec![3, 4]);

        let actual = vec.from_end(-4);
        assert_eq!(actual, vec![1, 2, 3, 4]);

        let actual = vec.from_end(-5);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = vec.from_end(-6);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = vec.from_end(6);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = vec.from_end(5);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let actual = vec.from_end(1);
        assert_eq!(actual, vec![4]);
    }
}
