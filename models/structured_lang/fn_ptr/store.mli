val bindings :
  Runtime_types.Frame_id.t ->
  Utils_names.Identifier.t list ->
  Runtime_types.value list ->
  (Utils_names.Identifier.t * Runtime_types.location) list
  * (Runtime_types.location * Runtime_types.value) list

val store_lookup :
  Runtime_types.state -> Runtime_types.location -> Runtime_types.value

val store_assign :
  Runtime_types.state ->
  Runtime_types.location ->
  Runtime_types.value ->
  Runtime_types.state
