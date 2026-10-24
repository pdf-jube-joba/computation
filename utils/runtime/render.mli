(** Typed rendering blocks; hosts serialize a complete rendering with
    [to_yojson]. *)
type orientation = Vertical | Horizontal

type display = Inline | Block
type graph_node = { id : string; label : string; ports : string list }

type graph_edge = {
  source : string * int;
  target : string * int;
  highlighted : bool;
}

type block
type t = block list

val text : ?title:string -> ?class_name:string -> string -> block

val graph :
  ?title:string ->
  nodes:graph_node list ->
  edges:graph_edge list ->
  unit ->
  block
(** Undirected port graph. An endpoint is a node ID and a zero-based port index.
    Node IDs must be unique and all endpoints must exist. *)

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

val highlight : ?title:string -> index:int -> ('a -> string) -> 'a list -> block
(** Display a horizontal sequence, highlighting the item at [index]. *)

val to_yojson : t -> Yojson.Safe.t

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
  t
