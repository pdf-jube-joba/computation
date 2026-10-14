type ctx

val create : unit -> ctx
val fresh_region : ctx -> string -> Utils_names.Identifier.t
val fresh_block : ctx -> string -> Utils_names.Identifier.t
val fresh_vreg : ctx -> string -> Utils_names.Identifier.t
val go : Utils_names.Identifier.t -> Syntax.cont
val enter : Utils_names.Identifier.t -> Syntax.cont

val block :
  Utils_names.Identifier.t -> Syntax.stmt list -> Syntax.cont -> Syntax.block

val region : ctx -> Utils_names.Identifier.t -> Syntax.block list -> unit

val single :
  ctx -> Utils_names.Identifier.t -> Syntax.stmt list -> Syntax.cont -> unit

val finish :
  ?prefix:Syntax.region list -> ctx -> Syntax.static_def list -> Syntax.code
