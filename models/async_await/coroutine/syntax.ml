module N = Utils_base.Number

type aatom = Var of Utils_names.Identifier.t | Imm of N.t [@@deriving yojson]
type abinop = Add | Sub | Mul [@@deriving yojson]
type aexp = Atom of aatom | Bin of aexp * abinop * aexp [@@deriving yojson]
type relop = Eq | Ne | Lt [@@deriving yojson]

type bexp =
  | Not of bexp
  | Rel of aexp * relop * aexp
  | And of bexp * bexp
  | Or of bexp * bexp
  | Done of Utils_names.Identifier.t
[@@deriving yojson]

type stmt =
  | Assign of Utils_names.Identifier.t * aexp
  | IfGoto of bexp * int
  | Goto of int
  | Call of Utils_names.Identifier.t
  | Run of Utils_names.Identifier.t * Utils_names.Identifier.t
  | Yield
[@@deriving yojson]

type fn_decl = { name : Utils_names.Identifier.t; body : stmt list }
[@@deriving yojson]

type code = fn_decl list [@@deriving yojson]

let validate code =
  let names = List.map (fun f -> f.name) code in
  Utils_names.Identifier.check_unique
    ~message:(fun name ->
      "duplicate function: " ^ Utils_names.Identifier.to_string name)
    names;
  if not (List.mem (Utils_names.Identifier.make_exn "main") names) then
    failwith "missing main function"
