use std::{collections::VecDeque, iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum TypeToken {
	Expr,
	Bool,
	Church,
	Tuple,
	List,
	LeftSquareBracket,
	RightSquareBracket,
	Comma,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	Lambda,
	Ident(String),
	Dot,
	Lpar,
	Rpar,
	Where,
	Def,
	End,
	Comment(String),
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

	pub fn remove_comments(&mut self) {
		self.tokens = self
			.tokens
			.iter()
			.filter(|t| match t {
				Token::Comment(_) => false,
				_ => true,
			})
			.cloned()
			.collect();
	}
}

pub fn lex_type(dt: &str) -> Vec<TypeToken> {
	let mut chars = dt.chars().peekable();
	let mut tokens: Vec<TypeToken> = Vec::new();
	while let Some(c) = chars.next() {
		match c {
			'[' => tokens.push(TypeToken::LeftSquareBracket),
			']' => tokens.push(TypeToken::RightSquareBracket),
			',' => tokens.push(TypeToken::Comma),
			c if c.is_whitespace() => {}
			c if c.is_ascii_alphabetic() => {
				let mut ident_chars = vec![c];
				loop {
					match chars.peek() {
						Some(&c) if c.is_ascii_alphabetic() => {
							ident_chars.push(c);
							chars.next();
						}
						_ => break,
					}
				}
				let s = ident_chars.iter().collect::<String>();
				let tok = match s.as_str() {
					"expr" => TypeToken::Expr,
					"bool" => TypeToken::Bool,
					"church" => TypeToken::Church,
					"tuple" => TypeToken::Tuple,
					"list" => TypeToken::List,
					s => panic!("Invalid type identifier: {s}"),
				};
				tokens.push(tok);
			}
			_ => panic!("Invalid character in type: {c}"),
		}
	}
	tokens
}

const IDENT_SPECIAL_CHARS: [char; 25] = [
	'~', '`', '!', '@', '#', '$', '%', '^', '&', '*', '-', '_', '+', '[', ']', '|', ':', ';', '\'',
	'"', ',', '<', '>', '/', '?',
];

pub fn lex(code: &str) -> TokenStream {
	let mut tokens = VecDeque::new();
	let mut chars = code.chars().peekable();
	while let Some(c) = chars.next() {
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
			'=' => tokens.push_back(Token::Def),
			'{' => {
				let mut n: usize = 0;
				let mut s: Vec<char> = Vec::new();
				loop {
					let c = chars.next();
					match c {
						Some('}') if n == 0 => break,
						Some('}') => n -= 1,
						Some('{') => n += 1,
						None => panic!("Unclosed comment"),
						_ => {}
					}
					s.push(c.unwrap());
				}
				tokens.push_back(Token::Comment(s.iter().collect::<String>()));
			}
			c if is_ident_char(&c) => {
				let name = lex_ident(c, &mut chars);
				let tok = match name.as_str() {
					"where" => Token::Where,
					_ => Token::Ident(name),
				};
				tokens.push_back(tok);
			}
			c if c.is_whitespace() => {}
			c => {
				// TODO: Handle errors better
				panic!("Invalid char: '{}'", c);
			}
		}
	}
	TokenStream { tokens: tokens }
}

fn lex_ident(first: char, chars: &mut Peekable<Chars>) -> String {
	let mut s = vec![first];
	loop {
		match chars.peek() {
			Some(&c) if is_ident_char(&c) => {
				s.push(c);
				chars.next();
			}
			_ => break,
		}
	}
	s.iter().collect()
}

fn is_ident_char(c: &char) -> bool {
	c.is_alphanumeric() || IDENT_SPECIAL_CHARS.contains(&c)
}

#[cfg(test)]
mod lex_type_tests {
	use crate::lex::*;

	#[test]
	fn test_lex_expr() {
		assert_eq!(lex_type("expr"), vec![TypeToken::Expr]);
	}

	#[test]
	fn test_lex_bool() {
		assert_eq!(lex_type("bool"), vec![TypeToken::Bool]);
	}

	#[test]
	fn test_lex_church() {
		assert_eq!(lex_type("church"), vec![TypeToken::Church]);
	}

	#[test]
	fn test_lex_1_tuple() {
		assert_eq!(
			lex_type("tuple[expr]"),
			vec![
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::Expr,
				TypeToken::RightSquareBracket
			]
		);
	}

	#[test]
	fn test_lex_2_tuple() {
		assert_eq!(
			lex_type("tuple [ bool , church ]"),
			vec![
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::Bool,
				TypeToken::Comma,
				TypeToken::Church,
				TypeToken::RightSquareBracket
			]
		);
	}

	#[test]
	fn test_lex_empty_tuple() {
		assert_eq!(
			lex_type("tuple[]"),
			vec![
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::RightSquareBracket
			]
		);
	}

	#[test]
	fn test_lex_nested_tuple() {
		assert_eq!(
			lex_type("tuple[expr, tuple[bool, tuple[church]]]"),
			vec![
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::Expr,
				TypeToken::Comma,
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::Bool,
				TypeToken::Comma,
				TypeToken::Tuple,
				TypeToken::LeftSquareBracket,
				TypeToken::Church,
				TypeToken::RightSquareBracket,
				TypeToken::RightSquareBracket,
				TypeToken::RightSquareBracket
			]
		);
	}

	#[test]
	fn test_lex_list() {
		assert_eq!(
			lex_type("list[church]"),
			vec![
				TypeToken::List,
				TypeToken::LeftSquareBracket,
				TypeToken::Church,
				TypeToken::RightSquareBracket
			]
		);
	}

	#[test]
	#[should_panic(expected = "Invalid type identifier: boo")]
	fn test_lex_invalid_type_identifier() {
		lex_type("boo");
	}

	#[test]
	#[should_panic(expected = "Invalid character in type: (")]
	fn test_lex_invalid_character() {
		lex_type("bool()");
	}
}

#[cfg(test)]
mod lex_tests {
	use crate::lex::*;

	#[test]
	fn lex_comment() -> () {
		assert_eq!(
			vec![Token::Comment(" Hello there! ".to_owned()), Token::Lambda],
			lex(r#"{ Hello there! } \"#).all()
		)
	}

	#[test]
	fn lex_nested_comments() -> () {
		assert_eq!(
			vec![
				Token::Comment(" Outside { Inside } Outside ".to_owned()),
				Token::Lambda
			],
			lex(r#"{ Outside { Inside } Outside } \"#).all()
		)
	}

	#[test]
	#[should_panic(expected = "Unclosed comment")]
	fn lex_unclosed_comment() -> () {
		lex(r#"{ Hello there!"#);
	}

	#[test]
	#[should_panic(expected = "Unclosed comment")]
	fn lex_unclosed_nested_comment() -> () {
		lex(r#"{ { Hello there! }"#);
	}

	#[test]
	fn lex_lambda() -> () {
		assert_eq!(vec![Token::Lambda], lex(r#"\"#).all());
	}

	#[test]
	fn lex_simple_ident() -> () {
		assert_eq!(vec![Token::Ident("a".to_owned())], lex("a").all());
	}

	#[test]
	fn lex_long_ident() -> () {
		assert_eq!(
			vec![Token::Ident("aa".to_owned()), Token::Ident("bb".to_owned())],
			lex("aa bb").all()
		);
	}

	#[test]
	fn test_all_ident_chars() -> () {
		assert_eq!(
			vec![
				Token::Ident("ab".to_owned()),
				Token::Ident("CD".to_owned()),
				Token::Ident("12".to_owned()),
				Token::Ident("~~".to_owned()),
				Token::Ident("``".to_owned()),
				Token::Ident("!!".to_owned()),
				Token::Ident("@@".to_owned()),
				Token::Ident("##".to_owned()),
				Token::Ident("$$".to_owned()),
				Token::Ident("%%".to_owned()),
				Token::Ident("^^".to_owned()),
				Token::Ident("&&".to_owned()),
				Token::Ident("**".to_owned()),
				Token::Ident("--".to_owned()),
				Token::Ident("__".to_owned()),
				Token::Ident("++".to_owned()),
				Token::Ident("[[".to_owned()),
				Token::Ident("]]".to_owned()),
				Token::Ident("||".to_owned()),
				Token::Ident("::".to_owned()),
				Token::Ident(";;".to_owned()),
				Token::Ident("''".to_owned()),
				Token::Ident("\"\"".to_owned()),
				Token::Ident(",,".to_owned()),
				Token::Ident("<<".to_owned()),
				Token::Ident(">>".to_owned()),
				Token::Ident("//".to_owned()),
				Token::Ident("??".to_owned()),
				Token::Dot,
			],
			lex("ab CD 12 ~~ `` !! @@ ## $$ %% ^^ && ** -- __ ++ [[ ]] || :: ;; '' \"\" ,, << >> // ?? .").all(),
		);
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
	fn lex_where() -> () {
		assert_eq!(vec![Token::Where], lex("where").all());
	}

	#[test]
	fn lex_def() -> () {
		assert_eq!(vec![Token::Def], lex("=").all());
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

	#[test]
	fn lex_with_whitespace() -> () {
		assert_eq!(
			vec![
				Token::Lambda,
				Token::Ident("a".to_owned()),
				Token::Dot,
				Token::Lambda,
				Token::Ident("b".to_owned()),
				Token::Dot,
				Token::Ident("a".to_owned()),
				Token::Lpar,
				Token::Ident("a".to_owned()),
				Token::Ident("b".to_owned()),
				Token::Rpar,
			],
			lex("\\a.\\b.\n\ta (a b)").all()
		);
	}
	#[test]
	fn lex_with_decls() -> () {
		assert_eq!(
			vec![
				Token::Ident("f".to_owned()),
				Token::Ident("x".to_owned()),
				Token::Where,
				Token::Ident("f".to_owned()),
				Token::Def,
				Token::Lambda,
				Token::Ident("z".to_owned()),
				Token::Dot,
				Token::Ident("z".to_owned()),
				Token::Where,
				Token::Ident("x".to_owned()),
				Token::Def,
				Token::Lambda,
				Token::Ident("a".to_owned()),
				Token::Dot,
				Token::Ident("a".to_owned())
			],
			lex(r#"f x where f = \z.z where x = \a.a"#).all()
		);
	}
}
