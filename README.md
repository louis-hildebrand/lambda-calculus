# Lambda Calculus Interpreter

An interpreter for the [untyped lambda calculus](https://en.wikipedia.org/wiki/Lambda_calculus).
Access it at https://louis-hildebrand.github.io/lambda-calculus/.

## Syntax

The syntax expected by the interpreter is given in [grammar.txt](./grammar.txt).
It is mostly standard, except that:
- Lambda is represented using a single backslash, to make it easier to type.
- For convenience, terms can be named using the `where` keyword. `where` bindings *cannot* be recursive and the term is simply substituted wherever the name appears.

Anything starting with { and ending with } is considered a comment. Comments may be nested.

For convenience, you can also provide a type for an expression and the interpreter will attempt to interpret the result as that type.
This is done by adding a comment like `{:: THE_TYPE }`.
For example, if you give the type `church` (i.e., a Church numeral) to the expression `\s.\z.s(s(z))`, the interpreter will output 2.
The syntax for types in given in [grammar_types.txt](./grammar_types.txt).

## Examples

As usual, natural numbers can be represented using [Church numerals](https://en.wikipedia.org/wiki/Church_encoding) and the `+` operator can be implemented using the successor function, `succ`.
```
+ 4 2
where +    = \a.\b.a succ b
where 4    = succ (succ 2)
where 2    = \s.\z.s(s(z))
where succ = \n.\s.\z.s(n s z)

{ RESULT: \s.\z.s(s(s(s(s(s(z))))))  (i.e., 6) }
```

## Development

## Requirements

- rustup and cargo (https://www.rust-lang.org/tools/install)
- wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
- npm (https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)

In addition, to run the fuzzer, you need the nightly Rust compiler and cargo-fuzz, as described here: https://rust-fuzz.github.io/book/cargo-fuzz/setup.html.

## Running the Tests

- Unit tests: `cargo test`
- Integration tests: `wasm-pack test --firefox --headless`

## Running the Fuzzer

```sh
cargo fuzz list               # List available targets
cargo fuzz run <TARGET NAME>  # Run the fuzzer
```

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

To deploy to GitHub Pages, run `deploy.sh`.
This will build the app and push to the `gh-pages` branch.
