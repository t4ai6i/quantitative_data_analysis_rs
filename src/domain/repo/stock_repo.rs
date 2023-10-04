use crate::domain::entity::stock::Stock;

pub trait StockRepo {
    fn vec_from_csv<R: std::io::Read>(&self, rdr: R, has_headers: bool) -> Vec<Stock>;
}
