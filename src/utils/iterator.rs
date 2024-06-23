/// Retrieve the specified number of elements from the end
///
/// # Arguments
///
/// * `slice`: &[T]
/// * `number`: usize
///
/// returns: Vec<T, Global>
///
/// # Examples
///
/// ```
/// use quantitative_data_analysis_rs::utils::iterator::get_vec_containing_number_from_end_of_array;
///
/// let vec = vec![0, 1, 2, 3, 4];
/// let number = 2;
/// let actual = get_vec_containing_number_from_end_of_array(vec.as_slice(), number);
/// assert_eq!(actual, vec![3, 4]);
///
/// ```
pub fn get_vec_containing_number_from_end_of_array<T: Clone>(slice: &[T], number: usize) -> Vec<T> {
    let number = slice.len().saturating_sub(number);
    let (_, slice) = slice.split_at(number);
    slice.to_vec()
}

#[cfg(test)]
mod tests {
    use crate::utils::iterator::get_vec_containing_number_from_end_of_array;

    #[test]
    fn get_vec_containing_number_from_the_end_of_the_array_test() {
        let vec = vec![0, 1, 2, 3, 4];
        let number = 2;
        let actual = get_vec_containing_number_from_end_of_array(vec.as_slice(), number);
        assert_eq!(actual, vec![3, 4]);

        let number = 0;
        let actual = get_vec_containing_number_from_end_of_array(vec.as_slice(), number);
        assert_eq!(actual, Vec::<i32>::new());

        let number = 5;
        let actual = get_vec_containing_number_from_end_of_array(vec.as_slice(), number);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);

        let number = 6;
        let actual = get_vec_containing_number_from_end_of_array(vec.as_slice(), number);
        assert_eq!(actual, vec![0, 1, 2, 3, 4]);
    }
}
