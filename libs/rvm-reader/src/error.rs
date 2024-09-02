use std::error::Error;

use eyre::Report;
use nom::error::{ErrorKind, FromExternalError, ParseError};

type ExternalError = Box<dyn Error + Send + Sync + 'static>;
pub struct ParsingError<'a> {
	parts: Vec<(&'a [u8], ParsingErrorPart)>,
}

impl<'a> ParsingError<'a> {
	pub fn format(self, source: &[u8]) -> Report {
		let mut report: Option<Report> = None;
		for (remaining, part) in self.parts.into_iter() {
			let location = source.len() - remaining.len();

			let string = part.format(location);

			// Root element and external
			if let ParsingErrorPart::External(external) = part
				&& report.is_none()
			{
				report = Some(external.wrap_err(format!("[{location}..] External Error:")));
				continue;
			}

			match report {
				Some(old_report) => report = Some(old_report.wrap_err(string)),
				None => {
					report = Some(Report::msg(string));
				}
			};
		}

		report.unwrap_or(Report::msg("Unknown error"))
	}
}

pub enum ParsingErrorPart {
	External(Report),
	Context(&'static str),
	NomError(ErrorKind),
}

impl ParsingErrorPart {
	pub fn format(&self, location: usize) -> String {
		let mut output = format!("[{location}..] ");
		match self {
			ParsingErrorPart::External(error) => {
				output.push_str("External: ");
				output.push_str(&error.to_string());
			}
			ParsingErrorPart::Context(context) => {
				output.push_str("Context: ");
				output.push_str(context)
			}
			ParsingErrorPart::NomError(error) => {
				output.push_str("Nom: ");
				output.push_str(error.description())
			}
		}

		output
	}
}

impl<'a> ParseError<&'a [u8]> for ParsingError<'a> {
	fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
		ParsingError {
			parts: vec![(input, ParsingErrorPart::NomError(kind))],
		}
	}

	fn append(input: &'a [u8], kind: ErrorKind, mut other: Self) -> Self {
		other.parts.push((input, ParsingErrorPart::NomError(kind)));
		other
	}
}

impl<'a, E> FromExternalError<&'a [u8], E> for ParsingError<'a>
where
	E: Error + Send + Sync + 'static,
{
	fn from_external_error(input: &'a [u8], _: ErrorKind, e: E) -> Self {
		ParsingError {
			parts: vec![(input, ParsingErrorPart::External(Report::new(e)))],
		}
	}
}
impl<'a> nom::error::ContextError<&'a [u8]> for ParsingError<'a> {
	fn add_context(input: &'a [u8], ctx: &'static str, mut other: Self) -> Self {
		other.parts.push((input, ParsingErrorPart::Context(ctx)));
		other
	}
}
