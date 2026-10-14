open Syntax

include
  Utils_names.Binding.Make
    (struct
      type t = expr

      let variable name = Var name

      let view e =
        let open Utils_names.Binding in
        match e with
        | Var name -> Variable name
        | Fun (x, body) ->
            node ~binders:[ x ]
              [ ([ 0 ], body) ]
              (fun names children -> Fun (List.hd names, List.hd children))
        | Rec (f, x, body) ->
            node ~binders:[ f; x ]
              [ ([ 0; 1 ], body) ]
              (fun names children ->
                Rec (List.nth names 0, List.nth names 1, List.hd children))
        | BinOp (a, op, b) ->
            node
              [ ([], a); ([], b) ]
              (fun _ children ->
                BinOp (List.nth children 0, op, List.nth children 1))
        | UnOp (op, body) ->
            node [ ([], body) ] (fun _ children -> UnOp (op, List.hd children))
        | App (a, b) ->
            node
              [ ([], a); ([], b) ]
              (fun _ children -> App (List.nth children 0, List.nth children 1))
        | If (c, a, b) ->
            node
              [ ([], c); ([], a); ([], b) ]
              (fun _ children ->
                If
                  (List.nth children 0, List.nth children 1, List.nth children 2))
        | Block _ ->
            invalid_arg "substitution applies to the direct expression language"
        | _ -> node [] (fun _ _ -> e)
    end)
    ()
