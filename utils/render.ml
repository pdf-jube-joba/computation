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

let table ?title ?(columns = []) ?highlight rows =
  `Assoc
    ([
       ("kind", `String "table");
       ("columns", `List columns);
       ( "rows",
         `List
           (List.mapi
              (fun i cells ->
                `Assoc
                  ([ ("cells", `List cells) ]
                  @
                  if highlight = Some i then
                    [ ("className", `String "highlight") ]
                  else []))
              rows) );
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

let text_table ?title ?(columns = []) ?highlight rows =
  table ?title ~columns:(List.map text columns) ?highlight
    (List.map (List.map text) rows)

let assoc_table ?title ?(columns = [ "var"; "value" ]) value_text entries =
  text_table ?title ~columns
    (List.map (fun (name, value) -> [ name; value_text value ]) entries)

let indexed_table ?title ?(columns = [ "index"; "value" ])
    ?(index_text = string_of_int) ?highlight value_text entries =
  text_table ?title ~columns ?highlight
    (List.mapi (fun i value -> [ index_text i; value_text value ]) entries)

let statement_env stmt value_text env =
  state [ text ~title:"stmt" stmt; assoc_table ~title:"env" value_text env ]
