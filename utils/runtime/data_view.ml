let tape value_text t =
  let signs, pos = Utils_data.Tape.into_vec t in
  Render.container ~title:"tape" ~orientation:Render.Horizontal
    (List.mapi
       (fun i s ->
         if i = pos then Render.text ~class_name:"highlight" (value_text s)
         else Render.text (value_text s))
       signs)
