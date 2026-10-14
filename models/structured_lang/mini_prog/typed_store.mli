val type_match : Runtime_types.value -> Syntax.ty -> bool
val default_value : Syntax.ty -> Runtime_types.value

val lookup_location :
  Runtime_types.state ->
  Runtime_types.location ->
  Syntax.ty * Runtime_types.value

val store_assign :
  Runtime_types.state ->
  Runtime_types.location ->
  Runtime_types.value ->
  Runtime_types.state
