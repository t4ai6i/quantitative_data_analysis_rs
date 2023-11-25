use crate::infrastructure::data_format::DataFormat;
use chrono::NaiveDate;
use yahoo_finance_api::time::OffsetDateTime;
use yahoo_finance_api::YahooConnector;

pub struct YahooFinanceAPI<'a> {
    pub provider: &'a YahooConnector,
    pub data_format: DataFormat,
}

impl<'a> YahooFinanceAPI<'a> {
    pub fn new(provider: &'a YahooConnector, data_format: DataFormat) -> Self {
        Self {
            provider,
            data_format,
        }
    }
}

pub struct OffsetDateTimeWrapper(pub(crate) OffsetDateTime);

impl From<NaiveDate> for OffsetDateTimeWrapper {
    fn from(value: NaiveDate) -> Self {
        let timestamp = value.and_hms_opt(0, 0, 0).unwrap().timestamp();
        let offset_date_time = OffsetDateTime::from_unix_timestamp(timestamp).unwrap();
        OffsetDateTimeWrapper(offset_date_time)
    }
}
