use std::collections::HashMap;

use crate::error::Error;
use crate::lex::{Token, TokenStream};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
	Fun(String, Box<Expr>),
	App(Box<Expr>, Box<Expr>),
	Var(String),
}

pub fn find_type_annotation(tokens: &mut TokenStream) -> Option<String> {
	loop {
		match tokens.next() {
			Token::End => return None,
			Token::Comment(s) if s.starts_with("::") => {
				return Some(s.strip_prefix("::").unwrap().trim().to_owned())
			}
			_ => (),
		}
	}
}

pub fn parse(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	let e = parse_e(tokens)?;
	let decls = parse_decls(tokens)?;
	let ee = inline_decls(&e, &decls);
	match tokens.next() {
		Token::End => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"unexpected trailing token \"{t}\""
			)))
		}
	};
	Ok(ee)
}

fn parse_e(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	let mut e = parse_eprime(tokens)?;
	loop {
		match tokens.peek() {
			Token::Lambda | Token::Ident(_) | Token::Lpar => {
				e = Box::new(Expr::App(e, parse_eprime(tokens)?))
			}
			_ => break,
		}
	}
	Ok(e)
}

fn parse_eprime(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	match tokens.peek() {
		Token::Lambda => parse_fun(tokens),
		Token::Ident(_) => parse_var(tokens),
		Token::Lpar => parse_parenthesized(tokens),
		t => return Err(Error::SyntaxError(format!("unexpected token \"{t}\""))),
	}
}

fn parse_fun(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	match tokens.next() {
		Token::Lambda => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Lambda
			)))
		}
	};
	let x = match tokens.next() {
		Token::Ident(name) => name,
		t => {
			return Err(Error::SyntaxError(format!(
				"expected an identifier but got \"{t}\""
			)))
		}
	};
	match tokens.next() {
		Token::Dot => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Dot
			)))
		}
	};
	let e = parse_e(tokens)?;
	Ok(Box::new(Expr::Fun(x, e)))
}

fn parse_var(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	match tokens.next() {
		Token::Ident(name) => Ok(Box::new(Expr::Var(name))),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected an identifier but got \"{t}\""
			)))
		}
	}
}

fn parse_parenthesized(tokens: &mut TokenStream) -> Result<Box<Expr>, Error> {
	match tokens.next() {
		Token::Lpar => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Lpar
			)))
		}
	}
	let e = parse_e(tokens);
	match tokens.next() {
		Token::Rpar => (),
		Token::End => return Err(Error::SyntaxError("unclosed parenthesis".to_owned())),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Rpar
			)))
		}
	}
	e
}

fn parse_decls(tokens: &mut TokenStream) -> Result<Vec<(String, Box<Expr>)>, Error> {
	let mut decls = Vec::new();
	loop {
		match tokens.peek() {
			Token::Where => decls.push(parse_decl(tokens)?),
			_ => break,
		}
	}
	Ok(decls)
}

fn parse_decl(tokens: &mut TokenStream) -> Result<(String, Box<Expr>), Error> {
	match tokens.next() {
		Token::Where => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Where
			)))
		}
	};
	let x = match tokens.next() {
		Token::Ident(name) => name,
		t => {
			return Err(Error::SyntaxError(format!(
				"expected an identifier but got \"{t}\""
			)))
		}
	};
	match tokens.next() {
		Token::Def => (),
		t => {
			return Err(Error::SyntaxError(format!(
				"expected \"{}\" but got \"{t}\"",
				Token::Def
			)))
		}
	};
	let e = parse_e(tokens)?;
	Ok((x, e))
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
			(false, Expr::Var(_)) => {}
			(false, Expr::Fun(x, body)) => {
				arg_stack.push(x);
				e_stack.push((false, body));
			}
			(false, Expr::App(e1, e2)) => {
				e_stack.push((false, e2));
				e_stack.push((false, e1));
			}
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
					None => panic!("missing argument"),
				};
				let body = match result_stack.pop() {
					Some(e) => e,
					None => panic!("missing result for function abstraction"),
				};
				result_stack.push(Box::new(Expr::Fun(x.clone(), body)));
			}
			(true, Expr::App(_, _)) => {
				let (e1, e2) = match (result_stack.pop(), result_stack.pop()) {
					(Some(e2), Some(e1)) => (e1, e2),
					_ => panic!("missing result for function application"),
				};
				result_stack.push(Box::new(Expr::App(e1, e2)));
			}
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
		assert_eq!(Ok(f), parse(&mut stm));
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
		assert_eq!(Ok(f), parse(&mut stm));
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
		assert_eq!(Ok(f), parse(&mut stm));
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
		assert_eq!(Ok(f), parse(&mut stm));
	}

	#[test]
	fn parse_abc() -> () {
		// \a.\b.\c.a b c
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
		assert_eq!(Ok(f), parse(&mut stm));
	}

	#[test]
	fn parse_too_many_rparens() -> () {
		// (x))
		let tokens = VecDeque::from(vec![
			Token::Lpar,
			Token::Ident("x".to_owned()),
			Token::Rpar,
			Token::Rpar,
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(
			Err(Error::SyntaxError(
				"unexpected trailing token \")\"".to_owned()
			)),
			parse(&mut stm)
		);
	}

	#[test]
	fn parse_unclosed_lparen() -> () {
		// ((x)
		let tokens = VecDeque::from(vec![
			Token::Lpar,
			Token::Lpar,
			Token::Ident("x".to_owned()),
			Token::Rpar,
		]);
		let mut stm = TokenStream { tokens };
		assert_eq!(
			Err(Error::SyntaxError("unclosed parenthesis".to_owned())),
			parse(&mut stm)
		);
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
		assert_eq!(Ok(e), parse(&mut stm));
	}
}
