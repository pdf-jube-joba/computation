module I = Flow_ir.Main

let get = function Ok x -> x | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "Expected error"
let eq a b = if a <> b then failwith ("Expected " ^ b ^ ", got " ^ a)

let run ?(input = "") code ainput =
  let rec loop fuel s out =
    if fuel = 0 then failwith "Step limit";
    let s' = get (I.restore (I.snapshot s)) in
    if I.snapshot s' <> I.snapshot s then failwith "Snapshot mismatch";
    ignore (I.render s');
    match get (I.step s' input) with
    | Utils.Machine.Halt output -> (output, out)
    | Continue { next; output } -> loop (fuel - 1) next (out ^ output)
  in
  loop 1000
    (get (I.make (get (I.Code.parse code)) (get (I.Ainput.parse ainput))))
    ""

let () =
  let out, text =
    run ~input:"Hi" {|@x 0
:main{:entry{input @x; %x:=ld @x; print %x; halt;}}|}
      ""
  in
  eq text "Hi";
  eq
    (I.Ainput.print_value (List.assoc "x" out))
    (I.Ainput.print_value (I.Num (Utils.Number.of_bytes "Hi")));
  let out, _ =
    run
      {|@x 0
:main{:entry{%p:=ref @x;deref %p:=st #7;push #11;push #13;pop %b;%a:=ld sacc #0;lget %l;%sum:=%a+%b;%sum:=%sum+%l;@x:=st %sum;halt;}}|}
      ""
  in
  eq (I.Ainput.print_value (List.assoc "x" out)) "25";
  let out, _ =
    run
      {|@x 0
:main{:entry{halloc(#2)%h;hacc(%h)[#1]:=st #65;%p:=ref hacc(%h)[#1];%v:=ld deref %p;@x:=st %v;hfree %h;halt;}}|}
      ""
  in
  eq (I.Ainput.print_value (List.assoc "x" out)) "65";
  let out, _ =
    run
      {|@x 0
:main{:entry{%a:=#99;enter :other;}}
:other{:entry{@x:=st %a;halt;}}|}
      ""
  in
  eq (I.Ainput.print_value (List.assoc "x" out)) "0";
  let out, _ =
    run
      {|@x 0
:main{:entry{%jmp:=:next;goto %jmp;}:next{@x:=st #3;if #1=#1 then :done;goto :next;}:done{halt;}}|}
      ""
  in
  eq (I.Ainput.print_value (List.assoc "x" out)) "3";
  let huge = "1234567890123456789012345678901234567890" in
  let out, _ =
    run ("@x 0\n:main{:entry{%n:=#" ^ huge ^ ";%n:=%n+#1;@x:=st %n;halt;}}") ""
  in
  eq (I.Ainput.print_value (List.assoc "x" out)) (Utils.Number.succ huge);
  error (I.make (get (I.Code.parse ":other{:entry{halt;}}")) []);
  error (I.make (get (I.Code.parse "@x 0 @x 1 :main{:entry{halt;}}")) []);
  error (I.make (get (I.Code.parse ":main{:entry{halt;}:entry{halt;}}")) []);
  error
    (I.make
       (get (I.Code.parse "@x 0 :main{:entry{halt;}}"))
       [ ("y", I.Num "1") ]);
  let s = get (I.make (get (I.Code.parse ":main{:entry{pop %a;halt;}}")) []) in
  error (I.step s "");
  let allocated =
    get (I.make (get (I.Code.parse ":main{:entry{halloc(#1)%h;halt;}}")) [])
  in
  let allocated =
    match get (I.step allocated "") with
    | Continue { next; _ } -> next
    | _ -> assert false
  in
  let rewound =
    {
      allocated with
      next_handle = I.Heap_id.Supply.after (get (I.Heap_id.of_int 0));
    }
  in
  error (I.restore (I.snapshot rewound));
  let inconsistent =
    { allocated with resolution = { allocated.resolution with registers = [] } }
  in
  error (I.restore (I.snapshot inconsistent));
  let s =
    get
      (I.make (get (I.Code.parse ":main{:entry{%a:=#255;print %a;halt;}}")) [])
  in
  let s =
    match get (I.step s "") with
    | Continue { next; _ } -> next
    | _ -> failwith "early halt"
  in
  error (I.step s "");
  let s =
    get
      (I.make
         (get
            (I.Code.parse
               ":main{:entry{if #1=#1 then :other;halt;}}:other{:entry{halt;}}"))
         [])
  in
  error (I.step s "");
  print_endline "flow_ir tests passed"
