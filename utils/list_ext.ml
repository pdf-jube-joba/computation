let nth entries index error =
  match List.nth_opt entries index with
  | Some value -> value
  | None -> failwith error

let replace entries index value error =
  ignore (nth entries index error);
  List.mapi (fun i v -> if i = index then value else v) entries
