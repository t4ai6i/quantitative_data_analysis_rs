use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(f: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if f.is_nan() {
        s.serialize_none()
    } else {
        s.serialize_f64(*f)
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<f64>::deserialize(d).map(|opt| opt.unwrap_or(f64::NAN))
}
