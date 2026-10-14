val validate : Syntax.code -> unit
(** Flow-specific binding rules, implemented with the common ID/name utilities.
*)

val build_resolution : Syntax.code -> Types.resolution

val name_of_id :
  (Utils_names.Identifier.t * 'id) list -> 'id -> Utils_names.Identifier.t

val id_of_name :
  (Utils_names.Identifier.t * 'id) list -> Utils_names.Identifier.t -> 'id

val local_target :
  Types.resolution -> int -> Utils_names.Identifier.t -> Types.code_target

val region_target :
  Types.resolution -> Utils_names.Identifier.t -> Types.code_target

val resolve_target :
  Types.resolution -> int -> Utils_names.Identifier.t -> Types.code_target

val target_name :
  Types.resolution -> Types.code_target -> Utils_names.Identifier.t

val resolve_input :
  Types.resolution -> int -> Types.flow_value -> Types.runtime_value

val named_value : Types.resolution -> Types.runtime_value -> Types.flow_value
