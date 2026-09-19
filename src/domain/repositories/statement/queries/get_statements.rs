#[derive(Debug, Default)]
pub struct Query<'a> {
    pub code: Option<&'a str>,
}
