(** Shared syntax: literals and matched parentheses, square brackets and braces.
    Language-specific expression precedence and AST construction belong to each
    model. *)
type token = Lexer.token =
  | Ident of string
  | Number of string
  | String_lit of string
  | Character_lit of string
  | Symbol of char
  | Eof

type delimiter = Parentheses | Brackets | Braces

type node =
  | Atom of { token : token; offset : int }
  | Group of {
      delimiter : delimiter;
      children : node list;
      opening_offset : int;
      closing_offset : int;
    }

type t
type checkpoint

exception Error of string

val create : ?nested_comments:bool -> ?character_literals:bool -> string -> t
(** Tokenize and build the tree once, rejecting unmatched/mismatched brackets.
    Offsets are byte offsets in the original source. *)

val tree : t -> node list
(** The immutable tree for this parser's scope, including consumed nodes. *)

val error : t -> string -> 'a
val protect : (unit -> 'a) -> ('a, string) result

val parse :
  ?nested_comments:bool ->
  ?character_literals:bool ->
  (t -> 'a) ->
  string ->
  ('a, string) result

val peek : t -> token
(** Flat token access remains available for grammars that explicitly consume
    opening and closing symbols. At the end of a scope, bump returns Eof and
    does not advance. *)

val bump : t -> token
val eof : t -> bool
val expect_eof : t -> unit

val bump_if : t -> string -> bool
(** Match a token sequence atomically. A failed match leaves the cursor intact.
    Patterns may contain partial brackets, e.g. ") ->". *)

val expect : t -> string -> unit
val bump_if_symbol : t -> char -> bool
val expect_symbol : t -> char -> unit
val bump_if_keyword : t -> string -> bool
val expect_keyword : t -> string -> unit
val identifier : t -> Identifier.t
val natural : t -> Number.t
val integer : t -> int
val string_literal : t -> string
val character_literal : t -> string

val group : t -> opening:char -> (t -> 'a) -> 'a
(** Read an already matched group through a child parser. The child sees Eof at
    the closing bracket and must consume all its contents. The parent advances
    past the whole group only on success. Opening must be a parenthesis, square
    bracket or brace. *)

val bump_group_if : t -> opening:char -> (t -> 'a) -> 'a option
(** None means the opening bracket is absent; errors inside a group propagate.
*)

val checkpoint : t -> checkpoint
(** Checkpoints can only be restored on the parser that created them. *)

val restore : t -> checkpoint -> unit

val many : ?skip_separator:char -> t -> (t -> 'a) -> 'a list
(** Read items to scope Eof. An optional separator may be omitted or repeated.
*)

val separated_list :
  ?allow_trailing:bool -> t -> separator:char -> (t -> 'a) -> 'a list
(** Read a strict separated list to scope Eof. Empty lists are accepted. *)

val many_until :
  ?skip_separator:char -> t -> closing:char -> (t -> 'a) -> 'a list
(** Flat token helpers for grammars with explicit closing symbols. *)

val separated : t -> separator:char -> (t -> 'a) -> 'a list

val separated_until :
  ?allow_trailing:bool ->
  t ->
  separator:char ->
  closing:char ->
  (t -> 'a) ->
  'a list
