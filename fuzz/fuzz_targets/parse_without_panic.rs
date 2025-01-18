#![no_main]

use lambda;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
	if let Ok(s) = std::str::from_utf8(data) {
		let mut stream = match lambda::lex::lex(s) {
			Ok(s) => s,
			Err(_) => return,
		};
		stream.remove_comments();
		let _ = lambda::parse::parse(&mut stream);
	}
});
