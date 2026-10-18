type orientation = Vertical | Horizontal
type display = Inline | Block

type block =
  | Text of {
      title : string option;
      class_name : string option;
      value : string;
    }
  | Table of {
      title : string option;
      columns : block list;
      rows : block list list;
      highlight : int option;
    }
  | Container of {
      title : string option;
      orientation : orientation;
      display : display;
      children : block list;
    }

let text ?title ?class_name value = Text { title; class_name; value }

let table ?title ?(columns = []) ?highlight rows =
  Table { title; columns; rows; highlight }

let container ?title ?(orientation = Vertical) ?(display = Block) children =
  Container { title; orientation; display; children }

let highlight ?title ~index value_text entries =
  container ?title ~orientation:Horizontal
    (List.mapi
       (fun i value ->
         text
           ?class_name:(if i = index then Some "highlight" else None)
           (value_text value))
       entries)

let rec to_yojson block =
  let optional name = function
    | None -> []
    | Some value -> [ (name, `String value) ]
  in
  match block with
  | Text { title; class_name; value } ->
      `Assoc
        ([ ("kind", `String "text"); ("text", `String value) ]
        @ optional "title" title
        @ optional "className" class_name)
  | Table { title; columns; rows; highlight } ->
      `Assoc
        ([
           ("kind", `String "table");
           ("columns", `List (List.map to_yojson columns));
           ( "rows",
             `List
               (List.mapi
                  (fun i cells ->
                    `Assoc
                      ([ ("cells", `List (List.map to_yojson cells)) ]
                      @
                      if highlight = Some i then
                        [ ("className", `String "highlight") ]
                      else []))
                  rows) );
         ]
        @ optional "title" title)
  | Container { title; orientation; display; children } ->
      `Assoc
        ([
           ("kind", `String "container");
           ("children", `List (List.map to_yojson children));
           ( "orientation",
             `String
               (match orientation with
               | Vertical -> "vertical"
               | Horizontal -> "horizontal") );
           ( "display",
             `String
               (match display with Inline -> "inline" | Block -> "block") );
         ]
        @ optional "title" title)

let state blocks = `List (List.map to_yojson blocks)

let text_table ?title ?(columns = []) ?highlight rows =
  table ?title ~columns:(List.map text columns) ?highlight
    (List.map (List.map text) rows)

let keyed_table ?title ?(columns = [ "var"; "value" ]) ~key_text value_text
    entries =
  text_table ?title ~columns
    (List.map
       (fun (name, value) -> [ key_text name; value_text value ])
       entries)

let assoc_table ?title ?columns value_text entries =
  keyed_table ?title ?columns ~key_text:Fun.id value_text entries

let indexed_table ?title ?(columns = [ "index"; "value" ])
    ?(index_text = string_of_int) ?highlight value_text entries =
  text_table ?title ~columns ?highlight
    (List.mapi (fun i value -> [ index_text i; value_text value ]) entries)

let statement_env ~name_text stmt value_text env =
  state
    [
      text ~title:"stmt" stmt;
      keyed_table ~key_text:name_text ~title:"env" value_text env;
    ]
