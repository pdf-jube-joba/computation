(** Shared syntax: literals, matched groups and expression parsing. Models
    supply their operator precedence and AST constructors. *)
type token = Lexer.token =
  | Ident of string
  | Keyword of string
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

val offset : t -> int
(** Byte offset of the next token, or the closing bracket at scope Eof. *)

val protect : (unit -> 'a) -> ('a, string) result

val parse :
  ?nested_comments:bool ->
  ?character_literals:bool ->
  (t -> 'a) ->
  string ->
  ('a, string) result

val peek : t -> token
(** At the end of a scope, returns Eof. Use [is_group] to recognize groups and
    [group] to consume them. *)

val bump : t -> token
val eof : t -> bool
val expect_eof : t -> unit

val bump_if : t -> string -> bool
(** Match a token sequence atomically. A failed match leaves the cursor intact.
    Use [group] for bracketed syntax. *)

val expect : t -> string -> unit
val bump_if_symbol : t -> char -> bool
val expect_symbol : t -> char -> unit
val bump_if_keyword : t -> string -> bool
val expect_keyword : t -> string -> unit

val keyword : t -> string
(** Read a backslash-prefixed word. Keyword helpers take its name without the
    backslash; [identifier] accepts only unprefixed names. *)

val identifier : t -> Utils_names.Identifier.t
val natural : t -> Utils_base.Number.t

val natural_with_plus : t -> Utils_base.Number.t
(** Like [natural], accepting an optional plus directly followed by digits. *)

val integer : t -> int
val string_literal : t -> string
val character_literal : t -> string

val is_group : t -> delimiter -> bool
(** Whether the next item is a group with the given delimiter. *)

val group : t -> delimiter -> (t -> 'a) -> 'a
(** Read an already matched group through a child parser. The child sees Eof at
    the closing bracket and must consume all its contents. The parent advances
    past the whole group only on success. *)

val bump_group_if : t -> delimiter -> (t -> 'a) -> 'a option
(** None means the opening bracket is absent; errors inside a group propagate.
*)

val take_until : t -> stop:(token -> bool) -> string
(** Consume a nonempty sequence up to [stop] or scope Eof and return its
    original text with surrounding whitespace trimmed. Groups are consumed as
    whole items; tokens inside them do not trigger [stop]. The stopping token
    remains unread. *)

val checkpoint : t -> checkpoint
(** Checkpoints can only be restored on the parser that created them. *)

val restore : t -> checkpoint -> unit

val attempt : t -> (t -> 'a) -> ('a, string) result
(** Try a parse, restoring the cursor on failure. The same exceptions as
    [protect] are converted to [Error]. Successful reads keep their cursor
    changes. *)

val chain_left :
  ?allow_trailing:bool ->
  t ->
  separator:char ->
  combine:('a -> 'a -> 'a) ->
  (t -> 'a) ->
  'a
(** Read a nonempty sequence and combine it from the left. A trailing separator
    may be accepted at scope Eof. *)

val expression :
  ?prefix:(string * int * ('a -> 'a)) list ->
  infix:(string * int * int * ('a -> 'a -> 'a)) list ->
  atom:((t -> 'a) -> t -> 'a) ->
  t ->
  'a
(** Parse an expression with binding powers starting at zero. Prefix entries
    contain the token sequence, right binding power and AST constructor. Infix
    entries contain the token sequence, left/right binding powers and
    constructor. Higher powers bind more tightly. For left associativity use
    right = left + 1; for right associativity use right = left. Order
    overlapping token sequences with the longest first. [atom] receives the
    recursive expression reader, for use with [group] or other model-specific
    syntax. Unrecognized operators remain unread, so callers can consume
    delimiters or keywords such as [goto]. *)

val binary :
  t ->
  operators:(string * 'op) list ->
  operand:(t -> 'a) ->
  make:('a -> 'op -> 'a -> 'b) ->
  'b
(** Read exactly one binary operation, such as a relation. This does not permit
    chained relations. Operators are tried in the supplied order. *)

val many : ?skip_separator:char -> t -> (t -> 'a) -> 'a list
(** Read items to scope Eof. An optional separator may be omitted or repeated.
*)

val separated_list :
  ?allow_trailing:bool ->
  ?skip_empty:bool ->
  t ->
  separator:char ->
  (t -> 'a) ->
  'a list
(** Read a strict separated list to scope Eof. Empty lists are accepted.
    [skip_empty] ignores empty fields, including leading, repeated and trailing
    separators. This matches the permissive list format used by input codecs. *)

val separated : t -> separator:char -> (t -> 'a) -> 'a list
