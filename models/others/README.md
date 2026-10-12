# OCaml library

`Others.add : int64 -> int64 -> int64` ports the crate's only implemented
function. The arguments hold unsigned 64-bit bit patterns; addition wraps as
in release Rust. `add_checked` reports overflow, matching the debug Rust
overflow check. `uint64_of_string` and `uint64_to_string` preserve the entire
unsigned range without reducing it to OCaml's smaller native integer range.

The state-jump design remains in `state_jump/spec.md`; no interpreter for it
existed in the Rust crate. There is no `Machine.S` or compiler API to register.
