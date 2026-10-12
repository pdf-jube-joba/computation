let () =
  Computation_cli.Host.main (function
    | "repl" :: args -> Computation_cli.Host.repl args
    | args -> Computation_cli.Host.pipe args)
