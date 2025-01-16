use std::fmt::Display;

use wasm_bindgen::JsValue;

#[derive(Debug, PartialEq)]
pub enum Error {
	SyntaxError(String),
	TypeError(String),
	MalformedType(String),
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::SyntaxError(s) => write!(f, "Syntax error: {s}"),
			Error::TypeError(t) => write!(f, "Type error: could not interpret result as {t}"),
			Error::MalformedType(s) => write!(f, "Malformed type: {s}"),
		}
	}
}

impl From<Error> for JsValue {
	fn from(value: Error) -> Self {
		// TODO: Pass back an object instead?
		JsValue::from_str(&value.to_string())
	}
}
