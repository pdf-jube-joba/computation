src に

## OCaml implementation status

The original Rust library was empty. `src/obj_lang.ml` is a small OCaml library
recording that no working interpreter, parser, machine or compiler existed.
It exposes `implemented = false`, `unsupported_reason`, and the names of the
unchanged design documents in `src/`. There is no `Utils.Machine.S` or compiler
API to register.
