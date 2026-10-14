module I = Syntax
module Fresh = Utils_names.Naming.Fresh ()

type ctx = {
  mutable regions : I.region list;
  region_names : Fresh.t;
  block_names : Fresh.t;
  vreg_names : Fresh.t;
}

let create () =
  {
    regions = [];
    region_names =
      Fresh.create ~reserved:[ Utils_names.Identifier.make_exn "main" ] ();
    block_names = Fresh.create ();
    vreg_names = Fresh.create ();
  }

let fresh_region c hint =
  Fresh.take c.region_names (Utils_names.Identifier.make_exn hint)

let fresh_block c hint =
  Fresh.take c.block_names (Utils_names.Identifier.make_exn hint)

let fresh_vreg c hint =
  Fresh.take c.vreg_names (Utils_names.Identifier.make_exn hint)

let go label = I.Go ([], I.CodeLabel label)
let enter label = I.Enter ([], I.CodeLabel label)
let block label stmts cont : I.block = { label; stmts; cont }
let region c label blocks = c.regions <- c.regions @ [ { I.label; blocks } ]

let single c label stmts cont =
  region c label [ block (fresh_block c "entry") stmts cont ]

let finish ?(prefix = []) c statics =
  { I.statics; regions = prefix @ c.regions }
