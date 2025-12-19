use anyhow::Context;
use serde_json::Value;
use std::str::FromStr;

/// string型である値を通して型変換を行うトレイト
pub trait FromJsonStringValue: Sized {
    /// JSONから指定されたキーでstring型の値を取得し、型変換を行う
    fn from_json_string_value(key: &str, value: &Value) -> anyhow::Result<Self>;
}

/// FromStrトレイトを実装している型に対する包括的な実装
impl<T> FromJsonStringValue for T
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug + std::fmt::Display,
{
    fn from_json_string_value(key: &str, value: &Value) -> anyhow::Result<Self> {
        let str_value = value[key]
            .as_str()
            .with_context(|| format!("value['{}'] cannot call with as_str()", key))?;

        T::from_str(str_value).map_err(|_| {
            anyhow::anyhow!("value['{}'] failed to call from_str('{}')", key, str_value)
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::shared::from_json_string_value::FromJsonStringValue;

    #[test]
    fn from_json_string_value_trait_successful_parsing_test() {
        let json_value = json!({
            "age": "25",
            "name": "John",
            "date": "2023-12-25",
            "price": "123.45",
            "active": "true",
            "disabled": "false",
            "empty": ""
        });

        // FromJsonKeyトレイトを直接使用 - 整数
        let age: Result<i32> = i32::from_json_string_value("age", &json_value);
        assert!(age.is_ok());
        assert_eq!(age.unwrap(), 25);

        // FromJsonKeyトレイトを直接使用 - 文字列
        let name: Result<String> = String::from_json_string_value("name", &json_value);
        assert!(name.is_ok());
        assert_eq!(name.unwrap(), "John");

        // FromJsonKeyトレイトを直接使用 - 日付
        let date: Result<NaiveDate> = NaiveDate::from_json_string_value("date", &json_value);
        assert!(date.is_ok());
        assert_eq!(
            date.unwrap(),
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
        );

        // FromJsonKeyトレイトを直接使用 - 浮動小数点数
        let price: Result<f64> = f64::from_json_string_value("price", &json_value);
        assert!(price.is_ok());
        assert_eq!(price.unwrap(), 123.45);

        // FromJsonKeyトレイトを直接使用 - boolean
        let active: Result<bool> = bool::from_json_string_value("active", &json_value);
        assert!(active.is_ok());
        assert_eq!(active.unwrap(), true);

        let disabled: Result<bool> = bool::from_json_string_value("disabled", &json_value);
        assert!(disabled.is_ok());
        assert_eq!(disabled.unwrap(), false);

        // FromJsonKeyトレイトを直接使用 - 空文字列
        let empty: Result<String> = String::from_json_string_value("empty", &json_value);
        assert!(empty.is_ok());
        assert_eq!(empty.unwrap(), "");
    }

    #[test]
    fn from_json_string_value_trait_error_cases_test() {
        // 存在しないキー
        let json_value = json!({
            "existing": "value"
        });
        let result: Result<String> = String::from_json_string_value("nonexistent", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['nonexistent'] cannot call with as_str()"));

        // 値が文字列ではない
        let json_value = json!({
            "number": 42,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        });

        let result: Result<i32> = i32::from_json_string_value("number", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['number'] cannot call with as_str()"));

        let result: Result<bool> = bool::from_json_string_value("boolean", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['boolean'] cannot call with as_str()"));

        let result: Result<String> = String::from_json_string_value("null", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['null'] cannot call with as_str()"));

        let result: Result<String> = String::from_json_string_value("array", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['array'] cannot call with as_str()"));

        let result: Result<String> = String::from_json_string_value("object", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['object'] cannot call with as_str()"));

        // パース失敗
        let json_value = json!({
            "invalid_int": "not_a_number",
            "invalid_float": "123.45.67",
            "invalid_bool": "maybe",
            "invalid_date": "2023-13-45"
        });

        let result: Result<i32> = i32::from_json_string_value("invalid_int", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_int'] failed to call from_str('not_a_number')")
        );

        let result: Result<f64> = f64::from_json_string_value("invalid_float", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_float'] failed to call from_str('123.45.67')")
        );

        let result: Result<bool> = bool::from_json_string_value("invalid_bool", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['invalid_bool'] failed to call from_str('maybe')"));

        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("invalid_date", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_date'] failed to call from_str('2023-13-45')")
        );
    }

    #[test]
    fn numeric_edge_cases_test() {
        let json_value = json!({
            "negative": "-42",
            "negative_float": "-123.456",
            "zero": "0",
            "zero_float": "0.0",
            "scientific": "1e10",
            "scientific_negative": "1.5e-10",
            "infinity": "inf",
            "neg_infinity": "-inf",
            "nan": "NaN",
            "max_u64": "18446744073709551615",
            "overflow_u64": "18446744073709551616",
            "negative_unsigned": "-42"
        });

        // 負の数
        let result: Result<i32> = i32::from_json_string_value("negative", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -42);

        let result: Result<f64> = f64::from_json_string_value("negative_float", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -123.456);

        // ゼロ
        let result: Result<i32> = i32::from_json_string_value("zero", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let result: Result<f64> = f64::from_json_string_value("zero_float", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);

        // 科学記数法
        let result: Result<f64> = f64::from_json_string_value("scientific", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1e10);

        let result: Result<f64> = f64::from_json_string_value("scientific_negative", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.5e-10);

        // 特殊な浮動小数点値
        let result: Result<f64> = f64::from_json_string_value("infinity", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), f64::INFINITY);

        let result: Result<f64> = f64::from_json_string_value("neg_infinity", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), f64::NEG_INFINITY);

        let result: Result<f64> = f64::from_json_string_value("nan", &json_value);
        assert!(result.is_ok());
        assert!(result.unwrap().is_nan());

        // u64の最大値
        let result: Result<u64> = u64::from_json_string_value("max_u64", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 18446744073709551615);

        // u64のオーバーフロー
        let result: Result<u64> = u64::from_json_string_value("overflow_u64", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message
                .contains("value['overflow_u64'] failed to call from_str('18446744073709551616')")
        );

        // 符号なし整数に負の値（エラー）
        let result: Result<u32> = u32::from_json_string_value("negative_unsigned", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['negative_unsigned'] failed to call from_str('-42')")
        );
    }

    #[test]
    fn boolean_edge_cases_test() {
        let json_value = json!({
            "upper_true": "TRUE",
            "upper_false": "FALSE",
            "mixed_case": "True",
            "numeric_true": "1",
            "numeric_false": "0",
            "empty_string": "",
            "whitespace": "   "
        });

        // Rustの標準的なbool::from_strは"true"と"false"のみを受け入れる
        let result: Result<bool> = bool::from_json_string_value("upper_true", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['upper_true'] failed to call from_str('TRUE')"));

        let result: Result<bool> = bool::from_json_string_value("upper_false", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['upper_false'] failed to call from_str('FALSE')"));

        let result: Result<bool> = bool::from_json_string_value("mixed_case", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['mixed_case'] failed to call from_str('True')"));

        let result: Result<bool> = bool::from_json_string_value("numeric_true", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['numeric_true'] failed to call from_str('1')"));

        let result: Result<bool> = bool::from_json_string_value("numeric_false", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['numeric_false'] failed to call from_str('0')"));

        let result: Result<bool> = bool::from_json_string_value("empty_string", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['empty_string'] failed to call from_str('')"));

        let result: Result<bool> = bool::from_json_string_value("whitespace", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['whitespace'] failed to call from_str('   ')"));
    }

    #[test]
    fn date_edge_cases_test() {
        let json_value = json!({
            "leap_year": "2020-02-29",
            "non_leap_year": "2021-02-29",
            "invalid_month": "2023-13-01",
            "invalid_day": "2023-02-30",
            "wrong_format": "12/25/2023",
            "partial_date": "2023-12",
            "extra_info": "2023-12-25 10:30:00"
        });

        // うるう年の有効な日付
        let result: Result<NaiveDate> = NaiveDate::from_json_string_value("leap_year", &json_value);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );

        // 非うるう年の無効な日付
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("non_leap_year", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['non_leap_year'] failed to call from_str('2021-02-29')")
        );

        // 無効な月
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("invalid_month", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_month'] failed to call from_str('2023-13-01')")
        );

        // 無効な日
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("invalid_day", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_day'] failed to call from_str('2023-02-30')")
        );

        // 間違った形式
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("wrong_format", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['wrong_format'] failed to call from_str('12/25/2023')")
        );

        // 部分的な日付
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("partial_date", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['partial_date'] failed to call from_str('2023-12')"));

        // 時間情報付きの日付
        let result: Result<NaiveDate> =
            NaiveDate::from_json_string_value("extra_info", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message
                .contains("value['extra_info'] failed to call from_str('2023-12-25 10:30:00')")
        );
    }

    #[test]
    fn empty_json_test() {
        // 空のJSONオブジェクト
        let json_value = json!({});
        let result: Result<String> = String::from_json_string_value("any_key", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['any_key'] cannot call with as_str()"));
    }

    #[test]
    fn unicode_test() {
        // Unicode文字列
        let json_value = json!({
            "japanese": "こんにちは",
            "emoji": "🎉",
            "mixed": "Hello 世界 🌍",
            "korean": "안녕하세요",
            "arabic": "مرحبا",
            "chinese": "你好",
            "russian": "Привет"
        });

        let result: Result<String> = String::from_json_string_value("japanese", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "こんにちは");

        let result: Result<String> = String::from_json_string_value("emoji", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "🎉");

        let result: Result<String> = String::from_json_string_value("mixed", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello 世界 🌍");

        let result: Result<String> = String::from_json_string_value("korean", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "안녕하세요");

        let result: Result<String> = String::from_json_string_value("arabic", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "مرحبا");

        let result: Result<String> = String::from_json_string_value("chinese", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "你好");

        let result: Result<String> = String::from_json_string_value("russian", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Привет");
    }

    #[test]
    fn various_integer_types_test() {
        let json_value = json!({
            "i8_max": "127",
            "i8_min": "-128",
            "u8_max": "255",
            "i16_max": "32767",
            "i16_min": "-32768",
            "u16_max": "65535",
            "i32_max": "2147483647",
            "i32_min": "-2147483648",
            "u32_max": "4294967295",
            "i64_max": "9223372036854775807",
            "i64_min": "-9223372036854775808",
            "u64_max": "18446744073709551615",
            "isize_test": "42",
            "usize_test": "42"
        });

        // i8
        let result: Result<i8> = i8::from_json_string_value("i8_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 127);

        let result: Result<i8> = i8::from_json_string_value("i8_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -128);

        // u8
        let result: Result<u8> = u8::from_json_string_value("u8_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 255);

        // i16
        let result: Result<i16> = i16::from_json_string_value("i16_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 32767);

        let result: Result<i16> = i16::from_json_string_value("i16_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -32768);

        // u16
        let result: Result<u16> = u16::from_json_string_value("u16_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 65535);

        // i32
        let result: Result<i32> = i32::from_json_string_value("i32_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2147483647);

        let result: Result<i32> = i32::from_json_string_value("i32_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -2147483648);

        // u32
        let result: Result<u32> = u32::from_json_string_value("u32_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4294967295);

        // i64
        let result: Result<i64> = i64::from_json_string_value("i64_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 9223372036854775807);

        let result: Result<i64> = i64::from_json_string_value("i64_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -9223372036854775808);

        // u64
        let result: Result<u64> = u64::from_json_string_value("u64_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 18446744073709551615);

        // isize
        let result: Result<isize> = isize::from_json_string_value("isize_test", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // usize
        let result: Result<usize> = usize::from_json_string_value("usize_test", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn various_float_types_test() {
        let json_value = json!({
            "f32_test": "123.45",
            "f64_test": "123.456789",
            "f32_max": "3.4028235e38",
            "f64_max": "1.7976931348623157e308",
            "f32_min": "1.17549435e-38",
            "f64_min": "2.2250738585072014e-308"
        });

        // f32
        let result: Result<f32> = f32::from_json_string_value("f32_test", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123.45);

        // f64
        let result: Result<f64> = f64::from_json_string_value("f64_test", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123.456789);

        // f32 max
        let result: Result<f32> = f32::from_json_string_value("f32_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3.4028235e38);

        // f64 max
        let result: Result<f64> = f64::from_json_string_value("f64_max", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.7976931348623157e308);

        // f32 min
        let result: Result<f32> = f32::from_json_string_value("f32_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.17549435e-38);

        // f64 min
        let result: Result<f64> = f64::from_json_string_value("f64_min", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.2250738585072014e-308);
    }

    #[test]
    fn parsing_failure_edge_cases_test() {
        let json_value = json!({
            "empty_for_int": "",
            "whitespace_only": "   ",
            "mixed_alphanumeric": "123abc",
            "leading_whitespace": "  123",
            "trailing_whitespace": "123  ",
            "multiple_dots": "123.45.67",
            "invalid_scientific": "1e",
            "invalid_hex": "0xGG",
            "special_chars": "!@#$%"
        });

        // 空文字列のint変換
        let result: Result<i32> = i32::from_json_string_value("empty_for_int", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['empty_for_int'] failed to call from_str('')"));

        // 空白のみのint変換
        let result: Result<i32> = i32::from_json_string_value("whitespace_only", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['whitespace_only'] failed to call from_str('   ')"));

        // 混在する文字列
        let result: Result<i32> = i32::from_json_string_value("mixed_alphanumeric", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['mixed_alphanumeric'] failed to call from_str('123abc')")
        );

        // 先頭空白
        let result: Result<i32> = i32::from_json_string_value("leading_whitespace", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['leading_whitespace'] failed to call from_str('  123')")
        );

        // 末尾空白
        let result: Result<i32> = i32::from_json_string_value("trailing_whitespace", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['trailing_whitespace'] failed to call from_str('123  ')")
        );

        // 複数のドット
        let result: Result<f64> = f64::from_json_string_value("multiple_dots", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['multiple_dots'] failed to call from_str('123.45.67')")
        );

        // 不正な科学記数法
        let result: Result<f64> = f64::from_json_string_value("invalid_scientific", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("value['invalid_scientific'] failed to call from_str('1e')")
        );

        // 不正な16進数
        let result: Result<i32> = i32::from_json_string_value("invalid_hex", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['invalid_hex'] failed to call from_str('0xGG')"));

        // 特殊文字
        let result: Result<i32> = i32::from_json_string_value("special_chars", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['special_chars'] failed to call from_str('!@#$%')"));
    }

    #[test]
    fn nested_json_access_test() {
        let json_value = json!({
            "level1": {
                "level2": {
                    "value": "42"
                }
            }
        });

        // 存在しないキーでの多重アクセス
        let result: Result<String> = String::from_json_string_value("level1", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['level1'] cannot call with as_str()"));
    }

    #[test]
    fn custom_string_types_test() {
        let json_value = json!({
            "text": "Hello World",
            "empty": "",
            "newline": "Line1\nLine2",
            "tab": "Column1\tColumn2",
            "quote": "He said \"Hello\"",
            "backslash": "Path\\to\\file"
        });

        // 通常の文字列
        let result: Result<String> = String::from_json_string_value("text", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");

        // 空文字列
        let result: Result<String> = String::from_json_string_value("empty", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");

        // 改行を含む文字列
        let result: Result<String> = String::from_json_string_value("newline", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Line1\nLine2");

        // タブを含む文字列
        let result: Result<String> = String::from_json_string_value("tab", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Column1\tColumn2");

        // 引用符を含む文字列
        let result: Result<String> = String::from_json_string_value("quote", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He said \"Hello\"");

        // バックスラッシュを含む文字列
        let result: Result<String> = String::from_json_string_value("backslash", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Path\\to\\file");
    }

    #[test]
    fn char_type_test() {
        let json_value = json!({
            "single_char": "A",
            "empty_char": "",
            "multi_char": "ABC",
            "unicode_char": "あ",
            "emoji_char": "🎉"
        });

        // 単一文字
        let result: Result<char> = char::from_json_string_value("single_char", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 'A');

        // 空文字列（エラー）
        let result: Result<char> = char::from_json_string_value("empty_char", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['empty_char'] failed to call from_str('')"));

        // 複数文字（エラー）
        let result: Result<char> = char::from_json_string_value("multi_char", &json_value);
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("value['multi_char'] failed to call from_str('ABC')"));

        // Unicode文字
        let result: Result<char> = char::from_json_string_value("unicode_char", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 'あ');

        let result: Result<char> = char::from_json_string_value("emoji_char", &json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), '🎉');
    }
}
