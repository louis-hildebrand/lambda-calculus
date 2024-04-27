use crate::lex::{Token, TokenStream};

#[derive(Debug, PartialEq)]
pub enum Expr {
	Fun(String, Box<Expr>),
	App(Box<Expr>, Box<Expr>),
	Var(String),
}

pub fn parse(tokens: &mut TokenStream) -> Box<Expr> {
	let e = parse_e(tokens);
	match tokens.next() {
		Token::End => (),
		t => panic!("Unexpected trailing token {t:?}"),
	};
	e
}

fn parse_e(tokens: &mut TokenStream) -> Box<Expr> {
	let mut e = parse_eprime(tokens);
	loop {
		match tokens.peek() {
			Token::Lambda | Token::Ident(_) | Token::Lpar =>
				e = Box::new(Expr::App(e, parse_eprime(tokens))),
			_ => break,
		}
	}
	e
}

fn parse_eprime(tokens: &mut TokenStream) -> Box<Expr> {
	match tokens.peek() {
		Token::Lambda => parse_fun(tokens),
		Token::Ident(_) => parse_var(tokens),
		Token::Lpar => parse_parenthesized(tokens),
		t => panic!("Unexpected token {t:?}"),
	}
}

fn parse_fun(tokens: &mut TokenStream) -> Box<Expr> {
	match tokens.next() {
		Token::Lambda => (),
		t => panic!("Expected {:?} but got {:?}", Token::Lambda, t),
	};
	let x = match tokens.next() {
		Token::Ident(name) => name,
		t => panic!("Expected an identifier but got {t:?}"),
	};
	match tokens.next() {
		Token::Dot => (),
		t => panic!("Expected {:?} but got {:?}", Token::Dot, t),
	};
	let e = parse_e(tokens);
	Box::new(Expr::Fun(x, e))
}

fn parse_var(tokens: &mut TokenStream) -> Box<Expr> {
	match tokens.next() {
		Token::Ident(name) => Box::new(Expr::Var(name)),
		t => panic!("Expected an identifier but got {t:?}"),
	}
}

fn parse_parenthesized(tokens: &mut TokenStream) -> Box<Expr> {
	match tokens.next() {
		Token::Lpar => (),
		t => panic!("Expected ( but got {t:?}"),
	}
	let e =parse_e(tokens);
	match tokens.next() {
		Token::Rpar => (),
		t => panic!("Expected ) but got {t:?}"),
	}
	e
}

#[cfg(test)]
mod parse_tests {
	use std::collections::VecDeque;

	use crate::parse::*;

	#[test]
	fn parse_identity() -> () {
		let f = Box::new(Expr::Fun(
			"x".to_owned(),
			Box::new(Expr::Var("x".to_owned())),
		));
		let tokens = VecDeque::from(vec![
			Token::Lambda,
			Token::Ident("x".to_owned()),
			Token::Dot,
			Token::Ident("x".to_owned()),
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(f, parse(&mut stm));
	}

	#[test]
	fn parse_0() -> () {
		// \s.\z.z
		let f = Box::new(Expr::Fun(
			"s".to_owned(),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::Var("z".to_owned())),
			)),
		));
		let tokens = VecDeque::from(vec![
			Token::Lambda,
			Token::Ident("s".to_owned()),
			Token::Dot,
			Token::Lambda,
			Token::Ident("z".to_owned()),
			Token::Dot,
			Token::Ident("z".to_owned()),
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(f, parse(&mut stm));
	}

	#[test]
	fn parse_1() -> () {
		// \s.\z.s(z)
		let f = Box::new(Expr::Fun(
			"s".to_owned(),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("s".to_owned())),
					Box::new(Expr::Var("z".to_owned())),
				)),
			)),
		));
		let tokens = VecDeque::from(vec![
			Token::Lambda,
			Token::Ident("s".to_owned()),
			Token::Dot,
			Token::Lambda,
			Token::Ident("z".to_owned()),
			Token::Dot,
			Token::Ident("s".to_owned()),
			Token::Lpar,
			Token::Ident("z".to_owned()),
			Token::Rpar,
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(f, parse(&mut stm));
	}

	#[test]
	fn parse_2() -> () {
		// \s.\z.s(s(z))
		let f = Box::new(Expr::Fun(
			"s".to_owned(),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("s".to_owned())),
					Box::new(Expr::App(
						Box::new(Expr::Var("s".to_owned())),
						Box::new(Expr::Var("z".to_owned())),
					)),
				)),
			)),
		));
		let tokens = VecDeque::from(vec![
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
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(f, parse(&mut stm));
	}

	#[test]
	fn parse_abc() -> () {
		// \a.\b.\c.abc
		// (To make sure associativity is working properly)
		let f = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::Fun(
					"c".to_owned(),
					Box::new(Expr::App(
						Box::new(Expr::App(
							Box::new(Expr::Var("a".to_owned())),
							Box::new(Expr::Var("b".to_owned())),
						)),
						Box::new(Expr::Var("c".to_owned())),
					)),
				)),
			)),
		));
		let tokens = VecDeque::from(vec![
			Token::Lambda,
			Token::Ident("a".to_owned()),
			Token::Dot,
			Token::Lambda,
			Token::Ident("b".to_owned()),
			Token::Dot,
			Token::Lambda,
			Token::Ident("c".to_owned()),
			Token::Dot,
			Token::Ident("a".to_owned()),
			Token::Ident("b".to_owned()),
			Token::Ident("c".to_owned()),
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(f, parse(&mut stm));
	}

	#[test]
	#[should_panic(expected = "Unexpected trailing token Rpar")]
	fn parse_too_many_rparens() -> () {
		// (x))
		let tokens = VecDeque::from(vec![
			Token::Lpar,
			Token::Ident("x".to_owned()),
			Token::Rpar,
			Token::Rpar,
		]);
		let mut stm = TokenStream { tokens };
		parse(&mut stm);
	}
}
