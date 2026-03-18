# cli

`cli` loads wasm components built under `.repo/generated/build-wasm/` by default.
If needed, set `COMPUTATION_COMPONENT_DIR` to point to a different directory.

`pipe-machine` is the one-shot interface.
Example: `cargo run -p cli --bin pipe-machine -- model example_counter create --code 5`
This prints the snapshot JSON to stdout.
To step a machine, pipe the previous snapshot into stdin:
`echo '{"count":5}' | cargo run -p cli --bin pipe-machine -- model example_counter step --rinput inc`

`repl-machine` is the interactive interface.
Example: `cargo run -p cli --bin repl-machine -- model example_counter --code 5`
You can also omit `--code` and `--ainput` to enter them interactively.
For compiler worlds, use subcommands such as:
`cargo run -p cli --bin repl-machine -- compiler compile-code recursive_function --code '...'`
