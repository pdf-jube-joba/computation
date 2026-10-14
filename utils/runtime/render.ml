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

type state = block list

let text ?title ?class_name value = Text { title; class_name; value }

let table ?title ?(columns = []) ?highlight rows =
  Table { title; columns; rows; highlight }

let container ?title ?(orientation = Vertical) ?(display = Block) children =
  Container { title; orientation; display; children }

let optional name = function
  | None -> []
  | Some value -> [ (name, `String value) ]

let rec to_yojson = function
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

let assoc_table ?title ?(columns = [ "var"; "value" ]) value_text entries =
  text_table ?title ~columns
    (List.map (fun (name, value) -> [ name; value_text value ]) entries)

let indexed_table ?title ?(columns = [ "index"; "value" ])
    ?(index_text = string_of_int) ?highlight value_text entries =
  text_table ?title ~columns ?highlight
    (List.mapi (fun i value -> [ index_text i; value_text value ]) entries)

let statement_env stmt value_text env =
  state [ text ~title:"stmt" stmt; assoc_table ~title:"env" value_text env ]
