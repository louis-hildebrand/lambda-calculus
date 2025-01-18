pub mod debruijn;
pub mod emit;
pub mod error;
pub mod eval;
pub mod interpret_as;
pub mod lex;
pub mod parse;

use crate::error::Error;
use interpret_as::interpret_as;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
	// When the `console_error_panic_hook` feature is enabled, we can call the
	// `set_panic_hook` function at least once during initialization, and then
	// we will get better error messages if our code ever panics.
	//
	// For more details see
	// https://github.com/rustwasm/console_error_panic_hook#readme
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn eval_lambda(src: &str) -> Result<String, Error> {
	set_panic_hook();
	let mut stream = lex::lex(src)?;
	stream.remove_comments();
	let e = parse::parse(&mut stream)?;
	let evaluated = e.to_debruijn().eval().to_named();
	stream = lex::lex(src)?;
	let datatype_str = parse::find_type_annotation(&mut stream).unwrap_or("expr".to_owned());
	let datatype = datatype_str.as_str().try_into()?;
	let out = interpret_as(&evaluated, &datatype);
	match out {
		Ok(s) => Ok(s),
		Err(()) => Err(Error::TypeError(datatype_str)),
	}
}
