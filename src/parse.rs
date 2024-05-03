use std::collections::HashMap;

use crate::lex::{Token, TokenStream};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
	Fun(String, Box<Expr>),
	App(Box<Expr>, Box<Expr>),
	Var(String),
}

pub fn parse(tokens: &mut TokenStream) -> Box<Expr> {
	let e = parse_e(tokens);
	let decls = parse_decls(tokens);
	let ee = inline_decls(&e, &decls);
	match tokens.next() {
		Token::End => (),
		t => panic!("Unexpected trailing token {t:?}"),
	};
	ee
}

fn parse_e(tokens: &mut TokenStream) -> Box<Expr> {
	let mut e = parse_eprime(tokens);
	loop {
		match tokens.peek() {
			Token::Lambda | Token::Ident(_) | Token::Lpar => {
				e = Box::new(Expr::App(e, parse_eprime(tokens)))
			}
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
	let e = parse_e(tokens);
	match tokens.next() {
		Token::Rpar => (),
		t => panic!("Expected ) but got {t:?}"),
	}
	e
}

fn parse_decls(tokens: &mut TokenStream) -> Vec<(String, Box<Expr>)> {
	let mut decls = Vec::new();
	loop {
		match tokens.peek() {
			Token::Where => decls.push(parse_decl(tokens)),
			_ => break,
		}
	}
	decls
}

fn parse_decl(tokens: &mut TokenStream) -> (String, Box<Expr>) {
	match tokens.next() {
		Token::Where => (),
		t => panic!("Expected 'where' but got {t:?}"),
	};
	let x = match tokens.next() {
		Token::Ident(name) => name,
		t => panic!("Expected an identifier but got {t:?}"),
	};
	match tokens.next() {
		Token::Def => (),
		t => panic!("Expected = but got {t:?}"),
	};
	let e = parse_e(tokens);
	(x, e)
}

fn inline_decls(e: &Expr, decls: &[(String, Box<Expr>)]) -> Box<Expr> {
	let mut e_by_var = HashMap::new();
	for (x, e) in decls.iter().rev() {
		e_by_var.insert(x, inline(e, &e_by_var));
	}
	inline(e, &e_by_var)
}

fn inline(e: &Expr, decls: &HashMap<&String, Box<Expr>>) -> Box<Expr> {
	let mut arg_stack = Vec::new();
	let mut e_stack = vec![(false, e)];
	let mut result_stack = Vec::new();
	while let Some((visited, e)) = e_stack.pop() {
		if !visited {
			e_stack.push((true, e));
		}
		match (visited, e) {
			(false, Expr::Var(_)) => {},
			(false, Expr::Fun(x, body)) => {
				arg_stack.push(x);
				e_stack.push((false, body));
			}
			(false, Expr::App(e1, e2)) => {
				e_stack.push((false, e2));
				e_stack.push((false, e1));
			},
			(true, Expr::Var(name)) => {
				let e = if arg_stack.contains(&name) {
					Expr::Var(name.clone())
				} else {
					match decls.get(&name) {
						Some(e) => *e.clone(),
						None => Expr::Var(name.clone()),
					}
				};
				result_stack.push(Box::new(e));
			}
			(true, Expr::Fun(x, _)) => {
				match arg_stack.pop() {
					Some(_) => (),
					None => panic!("Missing argument"),
				};
				let body = match result_stack.pop() {
					Some(e) => e,
					None => panic!("Missing result for function abstraction"),
				};
				result_stack.push(Box::new(Expr::Fun(x.clone(), body)));
			}
			(true, Expr::App(_, _)) => {
				let (e1, e2) = match (result_stack.pop(), result_stack.pop()) {
					(Some(e2), Some(e1)) => (e1, e2),
					_ => panic!("Missing result for function application")
				};
				result_stack.push(Box::new(Expr::App(e1, e2)));
			},
		}
	}
	result_stack.pop().unwrap()
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

	#[test]
	fn parse_with_decls() -> () {
		// x y where x = y y (\y.y) where y = \z.z z
		let e = Box::new(Expr::App(
			Box::new(Expr::App(
				Box::new(Expr::App(
					Box::new(Expr::Fun(
						"z".to_owned(),
						Box::new(Expr::App(
							Box::new(Expr::Var("z".to_owned())),
							Box::new(Expr::Var("z".to_owned())),
						)),
					)),
					Box::new(Expr::Fun(
						"z".to_owned(),
						Box::new(Expr::App(
							Box::new(Expr::Var("z".to_owned())),
							Box::new(Expr::Var("z".to_owned())),
						)),
					)),
				)),
				Box::new(Expr::Fun(
					"y".to_owned(),
					Box::new(Expr::Var("y".to_owned())),
				)),
			)),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("z".to_owned())),
					Box::new(Expr::Var("z".to_owned())),
				)),
			)),
		));
		let tokens = VecDeque::from(vec![
			Token::Ident("x".to_owned()),
			Token::Ident("y".to_owned()),
			Token::Where,
			Token::Ident("x".to_owned()),
			Token::Def,
			Token::Ident("y".to_owned()),
			Token::Ident("y".to_owned()),
			Token::Lpar,
			Token::Lambda,
			Token::Ident("y".to_owned()),
			Token::Dot,
			Token::Ident("y".to_owned()),
			Token::Rpar,
			Token::Where,
			Token::Ident("y".to_owned()),
			Token::Def,
			Token::Lambda,
			Token::Ident("z".to_owned()),
			Token::Dot,
			Token::Ident("z".to_owned()),
			Token::Ident("z".to_owned()),
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(e, parse(&mut stm));
	}
}
