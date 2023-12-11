//! Parsing utility for shell settings on Ukagaka.

use nom::error::VerboseError;

pub mod charset;

/// All-purpose Error type.
pub type ShellParseError<'a> = VerboseError<&'a str>;
