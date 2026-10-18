type t
(** Combinational expressions with names resolved to local, typed IDs. *)

val make :
  modules:Syntax.comb_module list ->
  inputs:Utils_names.Identifier.t list ->
  Syntax.comb_exp_val ->
  (t, string) result
(** Collect all module declarations before resolving their bodies and the root
    expression. Module bodies see only [IN]; the root sees [inputs]. Rejects
    duplicate declarations, unknown names and combinational call cycles.

    Sequence bindings become visible after their right-hand sides. An earlier
    binding of the same name keeps precedence, as in the original evaluator. *)

val eval :
  t ->
  (Utils_names.Identifier.t * Syntax.value) list ->
  (Syntax.value, string) result
(** Supply values for the declared inputs. Definitions and references have
    already been resolved; all module calls and variable reads use IDs. *)

val to_yojson : t -> Yojson.Safe.t

val of_yojson : Yojson.Safe.t -> (t, string) result
(** Save named declarations once and resolve them again when restoring, checking
    all references instead of trusting IDs from external JSON. *)
