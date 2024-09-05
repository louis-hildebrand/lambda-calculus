# Lambda Calculus Interpreter

An interpreter for the [untyped lambda calculus](https://en.wikipedia.org/wiki/Lambda_calculus).

## Syntax

The syntax expected by the interpreter is given in [grammar.txt](./grammar.txt).
It is mostly standard, except that:
- Lambda is represented using a single backslash, to make it easier to type.
- For convenience, terms can be named using the `where` keyword. `where` bindings *cannot* be recursive and the term is simply substituted wherever the name appears.

Anything starting with { and ending with } is considered a comment. Comments may be nested.

## Examples

As usual, natural numbers can be represented using [Church numerals](https://en.wikipedia.org/wiki/Church_encoding) and the `+` operator can be implemented using the successor function, `succ`.
```
+ 4 2
where +    = \a.\b.a succ b
where 4    = succ (succ 2)
where 2    = \s.\z.s(s(z))
where succ = \n.\s.\z.s(n s z)

{ RESULT: \s.\z.s(s(s(s(s(s(z))))))  [i.e., 6] }
```

More examples can be found in the [examples/](./examples/) directory.

## Running on localhost

Starting in the root of the repository, run the following commands (without the dollar signs).
You should then be able to access the website at http://localhost:8080.

```bash
$ wasm-pack build
$ cd www
$ npm install  # (only the first time)
$ npm run start
```

## Deploying to GitHub Pages

1. Start in the root of the repository.
2. Clean up old artifacts using `git clean -xdi`.
3. Run `wasm-pack build`.
4. Move to the `www` directory.
5. Run `npm install`.
6. Run `npm run build`. This should produce some build artifacts in the `www/dist/` directory.
7. Copy the contents of `www/dist/` as well as any CSS files in `www/` into the root of the repository in the `gh-pages` branch.
8. Commit the changes and push to GitHub.

## Running via Command Line

In the root of the repository, run `cargo build`.
You should then be able to run the command-line tool using `./target/debug/lambda`, as in the example below.

```bash
$ cargo build
$ ./target/debug/lambda -f ./examples/plus.lam
\a.\b.a (a (a (a (a (a b)))))
```
