use crate::debruijn::DBExpr;

pub fn eval(e: &DBExpr) -> Box<DBExpr> {
	let mut ee = Box::new(e.clone());
	loop {
		match try_beta_reduce(&ee) {
			Some(eee) => ee = eee,
			None => break,
		}
	}
	ee
}

/// Attempts to apply a single beta reduction to the given expression.
/// Returns `None` if no reduction is possible.
/// Otherwise, returns the reduced expression.
fn try_beta_reduce(e: &DBExpr) -> Option<Box<DBExpr>> {
	match e {
		DBExpr::Var(_) => None,
		DBExpr::Fun(body) => match try_beta_reduce(body) {
			Some(nb) => Some(Box::new(DBExpr::Fun(nb))),
			None => None,
		},
		DBExpr::App(f, arg) => {
			match f.as_ref() {
				DBExpr::Fun(body) => return Some(replace(body, arg, 0)),
				_ => (),
			}
			match try_beta_reduce(f) {
				Some(nf) => return Some(Box::new(DBExpr::App(nf, arg.clone()))),
				None => (),
			}
			match try_beta_reduce(arg) {
				Some(na) => Some(Box::new(DBExpr::App(f.clone(), na))),
				None => None,
			}
		}
	}
}

fn replace(e: &DBExpr, arg: &DBExpr, idx: usize) -> Box<DBExpr> {
	match e {
		DBExpr::Var(i) if *i == idx =>
			// This variable is bound by the lambda we're getting rid of
			Box::new(arg.clone()),
		DBExpr::Var(i) if *i > idx =>
			// This variable is bound by a lambda outside the one we're getting
			// rid of
			Box::new(DBExpr::Var(*i - 1)),
		DBExpr::Var(i) =>
			// This variable is bound by a lambda inside the one we're getting
			// rid of
			Box::new(DBExpr::Var(*i)),
		DBExpr::Fun(body) => Box::new(DBExpr::Fun(replace(body, arg, idx + 1))),
		DBExpr::App(f, a) => Box::new(DBExpr::App(replace(f, arg, idx), replace(a, arg, idx))),
	}
}

#[cfg(test)]
mod eval_tests {
	use crate::debruijn::DBExpr;
	use crate::eval::eval;

	#[test]
	fn eval_identity() -> () {
		let id = Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0))));
		assert_eq!(id, eval(&id));
	}

	#[test]
	fn eval_app_inside_fun() -> () {
		// \(\1 (\0) 0)(\0)
		let e = Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::Fun(Box::new(DBExpr::App(
				Box::new(DBExpr::App(
					Box::new(DBExpr::Var(1)),
					Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
				)),
				Box::new(DBExpr::Var(0)),
			)))),
			Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
		))));
		// \0 (\0) (\0)
		let expected = Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(0)),
				Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
			)),
			Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0)))),
		))));
		assert_eq!(expected, eval(&e));
	}

	#[test]
	fn eval_succ_0() -> () {
		// \\0
		let zero = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::Var(0))))));
		// \\\1(2 1 0)
		let succ = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(
			Box::new(DBExpr::App(
				Box::new(DBExpr::Var(1)),
				Box::new(DBExpr::App(
					Box::new(DBExpr::App(
						Box::new(DBExpr::Var(2)),
						Box::new(DBExpr::Var(1)),
					)),
					Box::new(DBExpr::Var(0)),
				)),
			)),
		))))));
		let f = Box::new(DBExpr::App(succ, zero));
		// \\1(0)
		let one = Box::new(DBExpr::Fun(Box::new(DBExpr::Fun(Box::new(DBExpr::App(
			Box::new(DBExpr::Var(1)),
			Box::new(DBExpr::Var(0)),
		))))));
		assert_eq!(one, eval(&f));
	}
}
