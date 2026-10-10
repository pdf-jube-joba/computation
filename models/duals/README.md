以下を実装できたらいいな
- lambda mu
- lambda mu mu-tilde
- symmetric lambda
- dual calculus
- CBPV

## OCaml implementation status

The original Rust entry point only printed `Hello, world!`; it did not implement
a calculus or a parser. `src/duals.ml` records `implemented = false`,
`unsupported_reason`, and the location of these unchanged design notes. There
is no `Utils.Machine.S` or compiler API to register.
