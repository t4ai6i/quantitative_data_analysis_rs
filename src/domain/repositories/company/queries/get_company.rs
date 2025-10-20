#[derive(Debug, Default)]
pub struct Query<'a> {
    pub code: &'a str,
    pub market: Option<&'a str>,
}
