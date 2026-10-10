type block = Json.t
type state = block list

let text ?title ?class_name value =
  `Assoc
    ([ ("kind", `String "text"); ("text", `String value) ]
    @ (match title with
      | None -> []
      | Some title -> [ ("title", `String title) ])
    @
    match class_name with
    | None -> []
    | Some name -> [ ("className", `String name) ])

let table ?title ?(columns = []) rows =
  `Assoc
    ([
       ("kind", `String "table");
       ("columns", `List columns);
       ( "rows",
         `List (List.map (fun cells -> `Assoc [ ("cells", `List cells) ]) rows)
       );
     ]
    @ match title with None -> [] | Some title -> [ ("title", `String title) ])

let container ?title ?(orientation = "vertical") ?(display = "block") children =
  `Assoc
    ([
       ("kind", `String "container");
       ("children", `List children);
       ("orientation", `String orientation);
       ("display", `String display);
     ]
    @ match title with None -> [] | Some title -> [ ("title", `String title) ])

let state blocks = `List blocks
