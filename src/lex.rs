use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	Lambda,
	Ident(String),
	Dot,
	Lpar,
	Rpar,
	End,
}

pub struct TokenStream {
	pub tokens: VecDeque<Token>,
}

impl TokenStream {
	pub fn next(&mut self) -> Token {
		match self.tokens.pop_front() {
			None => Token::End,
			Some(tok) => tok,
		}
	}

	pub fn peek(&self) -> &Token {
		match self.tokens.get(0) {
			None => &Token::End,
			Some(tok) => tok,
		}
	}

	pub fn all(&self) -> Vec<Token> {
		self.tokens.iter().cloned().collect()
	}
}

pub fn lex(code: &str) -> TokenStream {
	let mut tokens = VecDeque::new();
	for c in code.chars() {
		match c {
			'\\' => {
				tokens.push_back(Token::Lambda);
			}
			'.' => {
				tokens.push_back(Token::Dot);
			}
			'(' => {
				tokens.push_back(Token::Lpar);
			}
			')' => {
				tokens.push_back(Token::Rpar);
			}
			c if c.is_alphabetic() => {
				tokens.push_back(Token::Ident(c.to_string()));
			}
			c => {
				// TODO: Handle errors better
				panic!("Invalid char: '{}'", c);
			}
		}
	}
	TokenStream { tokens: tokens }
}

#[cfg(test)]
mod lex_tests {
	use crate::lex::*;

	#[test]
	fn lex_lambda() -> () {
		assert_eq!(vec![Token::Lambda], lex(r#"\"#).all());
	}

	#[test]
	fn lex_ident() -> () {
		assert_eq!(vec![Token::Ident("a".to_owned())], lex("a").all());
	}

	#[test]
	fn lex_dot() -> () {
		assert_eq!(vec![Token::Dot], lex(".").all());
	}

	#[test]
	fn lex_lpar() -> () {
		assert_eq!(vec![Token::Lpar], lex("(").all());
	}

	#[test]
	fn lex_rpar() -> () {
		assert_eq!(vec![Token::Rpar], lex(")").all());
	}

	#[test]
	fn lex_identity() -> () {
		assert_eq!(
			vec![
				Token::Lambda,
				Token::Ident("x".to_owned()),
				Token::Dot,
				Token::Ident("x".to_owned()),
			],
			lex(r#"\x.x"#).all()
		)
	}

	#[test]
	fn lex_0() -> () {
		assert_eq!(
			vec![
				Token::Lambda,
				Token::Ident("s".to_owned()),
				Token::Dot,
				Token::Lambda,
				Token::Ident("z".to_owned()),
				Token::Dot,
				Token::Ident("z".to_owned()),
			],
			lex(r#"\s.\z.z"#).all()
		)
	}

	#[test]
	fn lex_1() -> () {
		assert_eq!(
			vec![
				Token::Lambda,
				Token::Ident("s".to_owned()),
				Token::Dot,
				Token::Lambda,
				Token::Ident("z".to_owned()),
				Token::Dot,
				Token::Ident("s".to_owned()),
				Token::Lpar,
				Token::Ident("z".to_owned()),
				Token::Rpar
			],
			lex(r#"\s.\z.s(z)"#).all()
		)
	}

	#[test]
	fn lex_2() -> () {
		assert_eq!(
			vec![
				Token::Lambda,
				Token::Ident("s".to_owned()),
				Token::Dot,
				Token::Lambda,
				Token::Ident("z".to_owned()),
				Token::Dot,
				Token::Ident("s".to_owned()),
				Token::Lpar,
				Token::Ident("s".to_owned()),
				Token::Lpar,
				Token::Ident("z".to_owned()),
				Token::Rpar,
				Token::Rpar,
			],
			lex(r#"\s.\z.s(s(z))"#).all()
		)
	}
}
