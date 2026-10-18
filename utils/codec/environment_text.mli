val fold_lines :
  ?invalid_line:(string -> string) ->
  ('env -> string -> string -> 'env) ->
  'env ->
  string ->
  ('env, string) result
(** The line format is independent of a model's environment representation. *)

val parse :
  ?invalid_line:(string -> string) ->
  (string -> ('value, string) result) ->
  string ->
  ('value Utils_data.Environment.t, string) result

val print_lines :
  ?trailing_newline:bool ->
  ('value -> string) ->
  (string * 'value) list ->
  string

val print :
  ?trailing_newline:bool ->
  ('value -> string) ->
  'value Utils_data.Environment.t ->
  string

module Natural : sig
  type t = Utils_data.Environment.Natural.t

  val parse : string -> (t, string) result
  val print : t -> string
  val print_inline : t -> string
end
