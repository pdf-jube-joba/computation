(** Typed rendering blocks; serialization happens at [state]. *)
type orientation = Vertical | Horizontal

type display = Inline | Block
type block
type state = block list

val text : ?title:string -> ?class_name:string -> string -> block

val table :
  ?title:string ->
  ?columns:block list ->
  ?highlight:int ->
  block list list ->
  block

val container :
  ?title:string ->
  ?orientation:orientation ->
  ?display:display ->
  block list ->
  block

val to_yojson : block -> Yojson.Safe.t
val state : state -> Yojson.Safe.t

val text_table :
  ?title:string ->
  ?columns:string list ->
  ?highlight:int ->
  string list list ->
  block

val keyed_table :
  ?title:string ->
  ?columns:string list ->
  key_text:('key -> string) ->
  ('value -> string) ->
  ('key * 'value) list ->
  block

val assoc_table :
  ?title:string ->
  ?columns:string list ->
  ('value -> string) ->
  (string * 'value) list ->
  block

val indexed_table :
  ?title:string ->
  ?columns:string list ->
  ?index_text:(int -> string) ->
  ?highlight:int ->
  ('value -> string) ->
  'value list ->
  block

val statement_env :
  name_text:('name -> string) ->
  string ->
  ('value -> string) ->
  ('name * 'value) list ->
  Yojson.Safe.t
