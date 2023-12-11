//! # `Charset`
//!
//! enum `Charset` and parser for it.
//! It can decode bytes of the type it represents into `Cow<'a, str>`.
//!
//! ## Example
//!
//! ```
//! use encoding_rs::SHIFT_JIS;
//!
//! use shell_parser_common_rs::charset::parse_charset;
//!
//! let case_raw = "Shift_JIS\r\n...";
//! let (case, _, _) = SHIFT_JIS.encode(case_raw);
//!
//! let case_temp = String::from_utf8_lossy(&case);
//! let (_remain, charset) = match parse_charset(&case_temp) {
//!     Ok(v) => v,
//!     Err(e) => {
//!         eprintln!("{:?}", e);
//!         return;
//!     }
//! };
//!
//! let result = match charset.decode(&case) {
//!     Ok(v) => v,
//!     Err(e) => {
//!         eprintln!("{:?}", e);
//!         return;
//!     }
//! };
//!
//! assert_eq!(result, case_raw);
//! ```

use std::borrow::Cow;

use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};

use crate::ShellParseError;

/// `Charset` type.
#[derive(Debug, PartialEq, Clone)]
pub enum Charset {
    ASCII,
    ShiftJIS,
    ISO2022JP,
    EUCJP,
    UTF8,
    Default,
}

/// parser for [`Charset`].
///
/// [`Charset`]: crate::charset::Charset
pub fn parse_charset<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    alt((
        charset_ascii,
        charset_shift_jis,
        charset_iso_2022_jp,
        charset_euc_jp,
        charset_utf_8,
    ))(input)
}

fn charset_ascii<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    map(tag("ASCII"), |_| Charset::ASCII)(input)
}

fn charset_shift_jis<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    map(tag("Shift_JIS"), |_| Charset::ShiftJIS)(input)
}

fn charset_iso_2022_jp<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    map(tag("ISO-2022-JP"), |_| Charset::ISO2022JP)(input)
}

fn charset_euc_jp<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    map(tag("EUC-JP"), |_| Charset::EUCJP)(input)
}

fn charset_utf_8<'a>(input: &'a str) -> IResult<&'a str, Charset, ShellParseError> {
    map(tag("UTF-8"), |_| Charset::UTF8)(input)
}

impl Charset {
    /// Decodes bytes of the type it represents into `Cow<'a, str>`.
    pub fn decode<'a>(&self, input: &'a [u8]) -> Result<Cow<'a, str>, ()> {
        let decoder = match self {
            Charset::ASCII => encoding_rs::UTF_8,
            Charset::ShiftJIS => encoding_rs::SHIFT_JIS,
            Charset::ISO2022JP => encoding_rs::ISO_2022_JP,
            Charset::EUCJP => encoding_rs::EUC_JP,
            Charset::UTF8 => encoding_rs::UTF_8,
            Charset::Default => {
                let os_str = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(input) };
                return Ok(os_str.to_string_lossy());
            }
        };

        let (cow, encoding_used, had_errors) = decoder.decode(input);
        if had_errors || encoding_used != decoder {
            Err(())
        } else {
            Ok(cow)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_charset {
        use super::*;

        #[test]
        fn sucess_when_valid_str_ascii() {
            let case = "ASCII\r\n";
            let (remain, result) = parse_charset(case).unwrap();
            assert_eq!(remain, "\r\n");
            assert_eq!(result, Charset::ASCII);
        }

        #[test]
        fn sucess_when_valid_str_shift_jis() {
            let case = "Shift_JIS\r\n";
            let (remain, result) = parse_charset(case).unwrap();
            assert_eq!(remain, "\r\n");
            assert_eq!(result, Charset::ShiftJIS);
        }

        #[test]
        fn sucess_when_valid_str_iso_2022_jp() {
            let case = "ISO-2022-JP\r\n";
            let (remain, result) = parse_charset(case).unwrap();
            assert_eq!(remain, "\r\n");
            assert_eq!(result, Charset::ISO2022JP);
        }

        #[test]
        fn sucess_when_valid_str_euc_jp() {
            let case = "EUC-JP\r\n";
            let (remain, result) = parse_charset(case).unwrap();
            assert_eq!(remain, "\r\n");
            assert_eq!(result, Charset::EUCJP);
        }

        #[test]
        fn sucess_when_valid_str_utf_8() {
            let case = "UTF-8\r\n";
            let (remain, result) = parse_charset(case).unwrap();
            assert_eq!(remain, "\r\n");
            assert_eq!(result, Charset::UTF8);
        }

        #[test]
        fn failed_when_invalid_str() {
            let case = "x76";
            assert!(parse_charset(case).is_err());
        }
    }

    mod charset {
        use super::*;

        #[test]
        fn basic_behavior() {
            let charset = Charset::ASCII;
            println!("{:?}", charset.clone());
        }

        #[test]
        fn success_when_valid_str() {
            let case = "abcdefg".as_bytes();
            let result = Charset::ASCII.decode(case).unwrap();
            assert_eq!(result, "abcdefg");

            let case_raw = "あいうえお";
            let (case, _, _) = encoding_rs::SHIFT_JIS.encode(case_raw);
            let result = Charset::ShiftJIS.decode(&case).unwrap();
            assert_eq!(result, case_raw);

            let case_raw = "あいうえお";
            let (case, _, _) = encoding_rs::ISO_2022_JP.encode(case_raw);
            let result = Charset::ISO2022JP.decode(&case).unwrap();
            assert_eq!(result, case_raw);

            let case_raw = "あいうえお";
            let (case, _, _) = encoding_rs::EUC_JP.encode(case_raw);
            let result = Charset::EUCJP.decode(&case).unwrap();
            assert_eq!(result, case_raw);

            let case_raw = "あいうえお";
            let (case, _, _) = encoding_rs::UTF_8.encode(case_raw);
            let result = Charset::UTF8.decode(&case).unwrap();
            assert_eq!(result, case_raw);

            let case_raw = "あいうえお";
            let case_os_str = std::ffi::OsStr::new(case_raw);
            let case = case_os_str.as_encoded_bytes();
            let result = Charset::Default.decode(case).unwrap();
            assert_eq!(result, case_raw);
        }

        #[test]
        fn failed_when_invalid_str() {
            let case_raw = "あいうえお";
            let (case, _, _) = encoding_rs::SHIFT_JIS.encode(case_raw);
            assert!(Charset::UTF8.decode(&case).is_err());
        }
    }
}
