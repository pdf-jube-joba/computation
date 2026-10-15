## OCaml implementation status

The original Rust library was empty. `main.ml` is a small OCaml library
recording that no working interpreter, parser, machine or compiler existed.
It exposes `implemented = false`, `unsupported_reason`, and the names of the
design documents in `class_based/`, `message_passing/`, and `obj/`. There is no `Utils_runtime.Machine.S` or compiler
API to register.
