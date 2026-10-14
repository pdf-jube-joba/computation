val preprocess :
  Syntax.code ->
  ( Syntax.inst array
    * Utils_base.Number.t list
    * Types.symbols
    * Types.debug_info,
    string )
  result

val resolve_raw_value :
  Types.symbols ->
  Syntax.raw_directive ->
  int ->
  Syntax.raw_value ->
  Utils_base.Number.t
