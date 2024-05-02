use crate::parse::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum DBExpr {
	Fun(Box<DBExpr>),
	App(Box<DBExpr>, Box<DBExpr>),
	Var(usize),
}

/// Converts a term in the "named" form to a term using de Bruijn indices.
pub fn to_debruijn(expr: &Expr) -> Box<DBExpr> {
	let mut arg_stack: Vec<&str> = Vec::new();
	let mut e_stack = vec![(false, &*expr)];
	let mut result_stack: Vec<Box<DBExpr>> = Vec::new();
	while let Some((visited, e)) = e_stack.pop() {
		if !visited {
			e_stack.push((true, e));
		}
		match (visited, e) {
			(false, Expr::Fun(x, body)) => {
				e_stack.push((false, body));
				arg_stack.push(x);
			}
			(false, Expr::App(e1, e2)) => {
				e_stack.push((false, e2));
				e_stack.push((false, e1));
			}
			(false, Expr::Var(_)) => {}
			(true, Expr::Fun(x, _)) => {
				match result_stack.pop() {
					Some(e) => result_stack.push(Box::new(DBExpr::Fun(e))),
					None => panic!("Missing result for function abstraction"),
				};
				match arg_stack.pop() {
					Some(y) if x == y => (),
					_ => panic!("Unexpected argument popped from the stack"),
				}
			}
			(true, Expr::App(_, _)) => match (result_stack.pop(), result_stack.pop()) {
				(Some(e2), Some(e1)) => result_stack.push(Box::new(DBExpr::App(e1, e2))),
				_ => panic!("Missing result for function application"),
			},
			(true, Expr::Var(x)) => {
				let i = match arg_stack.iter().rev().position(|y| y == x) {
					Some(i) => i,
					None => panic!("Free variable {x} in expression"),
				};
				result_stack.push(Box::new(DBExpr::Var(i)));
			}
		}
	}
	result_stack.pop().unwrap()
}

/// Converts a term using de Bruijn indices to a term in the "named" form.
pub fn to_named(expr: &DBExpr) -> Box<Expr> {
	let mut arg_stack: Vec<(usize, String)> = Vec::new();
	let mut e_stack = vec![(false, &*expr)];
	let mut result_stack: Vec<Box<Expr>> = Vec::new();
	while let Some((visited, e)) = e_stack.pop() {
		if !visited {
			e_stack.push((true, e));
		}
		match (visited, e) {
			(false, DBExpr::Fun(body)) => {
				let arg_num = match arg_stack.last() {
					Some((i, _)) => i + 1,
					None => 0,
				};
				let arg_str = choose_ident(arg_num);
				arg_stack.push((arg_num, arg_str));
				e_stack.push((false, body));
			},
			(false, DBExpr::App(e1, e2)) => {
				e_stack.push((false, e2));
				e_stack.push((false, e1));
			},
			(false, DBExpr::Var(_)) => {},
			(true, DBExpr::Fun(_)) => {
				let arg = match arg_stack.pop() {
					Some((_, name)) => name,
					None => panic!("Missing argument"),
				};
				match result_stack.pop() {
					Some(e) => result_stack.push(Box::new(Expr::Fun(arg, e))),
					None => panic!("Missing result for function abstraction")
				}
			},
			(true, DBExpr::App(_, _)) => {
				match (result_stack.pop(), result_stack.pop()) {
					(Some(e2), Some(e1)) => result_stack.push(Box::new(Expr::App(e1, e2))),
					_ => panic!("Missing result for function application"),
				}
			},
			(true, DBExpr::Var(i)) => {
				let name = match arg_stack.get(arg_stack.len() - 1 - i) {
					Some((_, x)) => (*x).to_owned(),
					None => panic!("Invalid de Bruijn index")
				};
				result_stack.push(Box::new(Expr::Var(name)))
			},
		}
	}
	result_stack.pop().unwrap()
}

const ALPHABET: [char; 26] = [
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
	'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
];

fn choose_ident(i: usize) -> String {
	let c = ALPHABET[i % 26];
	let n = 1 + i / 26;
	(0..n).map(|_| c).collect()
}

#[cfg(test)]
mod debruijn_tests {
	use crate::debruijn::*;
	use crate::parse::Expr;

	// TODO: Handle free variables

	#[test]
	fn test_choose_ident() -> () {
		assert_eq!("a".to_owned(), choose_ident(0));
		assert_eq!("b".to_owned(), choose_ident(1));
		assert_eq!("c".to_owned(), choose_ident(2));
		assert_eq!("aa".to_owned(), choose_ident(26));
		assert_eq!("bb".to_owned(), choose_ident(27));
		assert_eq!("cc".to_owned(), choose_ident(28));
	}

	#[test]
	fn identity_to_debruijn() -> () {
		let e = Box::new(Expr::Fun(
			"x".to_owned(),
			Box::new(Expr::Var("x".to_owned())),
		));
		let expected = Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0))));
		assert_eq!(expected, to_debruijn(&e));
	}

	#[test]
	fn identity_to_named() -> () {
		let e = Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0))));
		let expected = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Var("a".to_owned())),
		));
		assert_eq!(expected, to_named(&e));
	}

	#[test]
	fn one_to_debruijn() -> () {
		// \s.\z.s(z)
		let e = Box::new(Expr::Fun(
			"s".to_owned(),
			Box::new(Expr::Fun(
				"z".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("s".to_owned())),
					Box::new(Expr::Var("z".to_owned())),
				)),
			)),
		));
		// \.\.1(0)
		let expected = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::Var(1)),
			Box::new(DBExpr::Var(0)),
		))))));
		assert_eq!(expected, to_debruijn(&e));
	}

	#[test]
	fn one_to_named() -> () {
		// \.\.1(0)
		let e = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::Var(1)),
			Box::new(DBExpr::Var(0)),
		))))));
		// \a.\b.a(b)
		let expected = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::Var("a".to_owned())),
					Box::new(Expr::Var("b".to_owned())),
				)),
			)),
		));
		assert_eq!(expected, to_named(&e));
	}

	#[test]
	fn deeply_nested_to_debruijn() -> () {
		// \x.\y.y (\x.x) (y x)
		let e = Box::new(Expr::Fun(
			"x".to_owned(),
			Box::new(Expr::Fun(
				"y".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::App(
						Box::new(Expr::Var("y".to_owned())),
						Box::new(Expr::Fun(
							"x".to_owned(),
							Box::new(Expr::Var("x".to_owned())),
						)),
					)),
					Box::new(Expr::App(
						Box::new(Expr::Var("y".to_owned())),
						Box::new(Expr::Var("x".to_owned())),
					)),
				)),
			)),
		));
		// \.\.0 (\.0) (0 1)
		let expected = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(0)),
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
			)),
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(0)),
				Box::new(DBExpr::Var(1)),
			)),
		))))));
		assert_eq!(expected, to_debruijn(&e));
	}

	#[test]
	fn deeply_nested_to_named() -> () {
		// \.\.0 (\.0) (0 1)
		let e = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(0)),
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
			)),
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(0)),
				Box::new(DBExpr::Var(1)),
			)),
		))))));
		// \a.\b.b (\c.c) (b a)
		let expected = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::Fun(
				"b".to_owned(),
				Box::new(Expr::App(
					Box::new(Expr::App(
						Box::new(Expr::Var("b".to_owned())),
						Box::new(Expr::Fun(
							"c".to_owned(),
							Box::new(Expr::Var("c".to_owned())),
						)),
					)),
					Box::new(Expr::App(
						Box::new(Expr::Var("b".to_owned())),
						Box::new(Expr::Var("a".to_owned())),
					)),
				)),
			)),
		));
		assert_eq!(expected, to_named(&e));
	}

	#[test]
	fn flat_to_debruijn() -> () {
		// \a.(\b.b) (\b.b) a
		let e = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::App(
				Box::new(Expr::App(
					Box::new(Expr::Fun(
						"b".to_owned(),
						Box::new(Expr::Var("b".to_owned())),
					)),
					Box::new(Expr::Fun(
						"b".to_owned(),
						Box::new(Expr::Var("b".to_owned())),
					)),
				)),
				Box::new(Expr::Var("a".to_owned())),
			)),
		));
		// \.(\.0) (\.0) 0
		let expected = Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
			)),
			Box::new(DBExpr::Var(0)),
		))));
		assert_eq!(expected, to_debruijn(&e));
	}

	#[test]
	fn flat_to_named() -> () {
		// \.(\.0) (\.0) 0
		let e = Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
			)),
			Box::new(DBExpr::Var(0)),
		))));
		// \a.(\b.b) (\b.b) a
		// (Can reuse b here!)
		let expected = Box::new(Expr::Fun(
			"a".to_owned(),
			Box::new(Expr::App(
				Box::new(Expr::App(
					Box::new(Expr::Fun(
						"b".to_owned(),
						Box::new(Expr::Var("b".to_owned())),
					)),
					Box::new(Expr::Fun(
						"b".to_owned(),
						Box::new(Expr::Var("b".to_owned())),
					)),
				)),
				Box::new(Expr::Var("a".to_owned())),
			)),
		));
		assert_eq!(expected, to_named(&e));
	}
}
