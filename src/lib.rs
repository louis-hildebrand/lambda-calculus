pub mod lex;
pub mod parse;
pub mod debruijn;
pub mod eval;
pub mod emit;

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
pub fn eval_lambda(e: &str) -> String {
	set_panic_hook();
	// TODO: handle errors gracefully
	let mut stream = lex::lex(e);
	let e = parse::parse(&mut stream);
	let output = e.to_debruijn().eval().to_named().to_string();
	output
}
