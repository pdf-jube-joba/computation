type t = AtomValue of string | ConsValue of t * t [@@deriving yojson]
