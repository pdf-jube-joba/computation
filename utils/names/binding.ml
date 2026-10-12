(** A syntax adapter describes which binders are in scope for each child.
    Substitution resolves names to IDs, substitutes by identity, then chooses
    printable names. No model-specific string renaming is required. *)
type 'a view =
  | Variable of Identifier.t
  | Node of
      Identifier.t list
      * (int list * 'a) list
      * (Identifier.t list -> 'a list -> 'a)

let node ?(binders = []) children rebuild = Node (binders, children, rebuild)

module Make
    (Syntax : sig
      type t

      val view : t -> t view
      val variable : Identifier.t -> t
    end)
    () =
struct
  module Var = Symbol.Make ()

  type resolved =
    | Var of Var.t
    | Tree of
        Var.t list
        * (int list * resolved) list
        * (Identifier.t list -> Syntax.t list -> Syntax.t)

  let rec resolve reader env term =
    match Syntax.view term with
    | Variable name -> Var (Var.Reader.use reader env name)
    | Node (names, children, rebuild) ->
        let binders = List.map (Var.Reader.bind reader) names in
        let children =
          List.map
            (fun (scope, child) ->
              let env =
                List.fold_left
                  (fun env i ->
                    let v = List.nth binders i in
                    (v.name, v) :: env)
                  env scope
              in
              (scope, resolve reader env child))
            children
        in
        Tree (binders, children, rebuild)

  let emit result =
    let names = ref Var.Names.empty in
    let name (v : Var.t) =
      let text, next = Var.Names.assign ~hint:v.name v.id !names in
      names := next;
      text
    in
    let rec free bound = function
      | Var v -> if not (Var.Id.Set.mem v.id bound) then ignore (name v)
      | Tree (binders, children, _) ->
          List.iter
            (fun (scope, child) ->
              let bound =
                List.fold_left
                  (fun bound i -> Var.Id.Set.add (List.nth binders i).id bound)
                  bound scope
              in
              free bound child)
            children
    in
    free Var.Id.Set.empty result;
    let rec visit = function
      | Var v -> Syntax.variable (name v)
      | Tree (binders, children, rebuild) ->
          let binders = List.map name binders in
          let children = List.map (fun (_, child) -> visit child) children in
          rebuild binders children
    in
    visit result

  let normalize term = emit (resolve (Var.Reader.create ()) [] term)

  let substitute source target replacement =
    let reader = Var.Reader.create () in
    let term = resolve reader [] source in
    let target = Var.Reader.use reader [] target in
    let replacement = resolve reader [] replacement in
    let changed = ref false in
    let rec replace = function
      | Var v when Var.equal v target ->
          changed := true;
          replacement
      | Var _ as v -> v
      | Tree (binders, children, rebuild) ->
          Tree
            ( binders,
              List.map (fun (scope, child) -> (scope, replace child)) children,
              rebuild )
    in
    let result = replace term in
    if !changed then emit result else source
end
