type 'a t = (Utils_names.Identifier.t * 'a) list [@@deriving yojson]

let get ~default env name = Option.value ~default (List.assoc_opt name env)

type natural = Utils_base.Number.t t [@@deriving yojson]

module Natural = struct
  type t = natural [@@deriving yojson]

  let get env name = get ~default:Utils_base.Number.zero env name
end
