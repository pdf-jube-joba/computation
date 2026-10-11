module I = Flow_ir

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
    (get (I.make (get (I.parse_code code)) (get (I.Ainput.parse ainput))))
    ""

let () =
  let code =
    {|@slot 0
:main {
 :entry {
 nop; %p := ref @slot; deref %p := st :next; %jmp := ld @slot;
 if %cond = #0 then :next; goto %jmp;
 }
 :next { halloc(#3) %h; hacc(%h)[#1] := st #7; %x := ld hacc(%h)[#1];
 print %x; input @slot; push %x; halt; }
}
:r2 { :entry { lget %len; pop %tmp; push %len; hfree #1; halt; } }
|}
  in
  let parsed = get (I.parse_code code) in
  if get (I.parse_code (I.print parsed)) <> parsed then failwith "Roundtrip";
  let dense =
    get
      (I.parse_code
         {|@slot 0 // static
/* region */ :main{:entry{%p:=ref @slot;if %cond=#0 then :next;goto %p;}
:next{halloc(#3)%h;hacc(%h)[#1]:=st #7;%x:=ld hacc(%h)[#1];halt;}}|})
  in
  if get (I.parse_code (I.print dense)) <> dense then failwith "Dense roundtrip";
  List.iter
    (fun s ->
      let v = get (I.parse_flow_value s) in
      if get (I.parse_flow_value (I.value_print v)) <> v then
        failwith "Value roundtrip")
    [ "123"; ":next"; "k:@slot"; "k:s[3]"; "k:h[2][4]" ];
  eq (I.print_env (get (I.Ainput.parse "z=3\na=1\nz=5"))) "a = 1\nz = 5\n";
  let out, text =
    run ~input:"Hi" {|@x 0
:main{:entry{input @x; %x:=ld @x; print %x; halt;}}|}
      ""
  in
  eq text "Hi";
  eq
    (I.value_print (List.assoc "x" out))
    (I.value_print (I.Num (Utils.Number.of_bytes "Hi")));
  let out, _ =
    run
      {|@x 0
:main{:entry{%p:=ref @x;deref %p:=st #7;push #11;push #13;pop %b;%a:=ld sacc #0;lget %l;%sum:=%a+%b;%sum:=%sum+%l;@x:=st %sum;halt;}}|}
      ""
  in
  eq (I.value_print (List.assoc "x" out)) "25";
  let out, _ =
    run
      {|@x 0
:main{:entry{halloc(#2)%h;hacc(%h)[#1]:=st #65;%p:=ref hacc(%h)[#1];%v:=ld deref %p;@x:=st %v;hfree %h;halt;}}|}
      ""
  in
  eq (I.value_print (List.assoc "x" out)) "65";
  let out, _ =
    run
      {|@x 0
:main{:entry{%a:=#99;enter :other;}}
:other{:entry{@x:=st %a;halt;}}|}
      ""
  in
  eq (I.value_print (List.assoc "x" out)) "0";
  let out, _ =
    run
      {|@x 0
:main{:entry{%jmp:=:next;goto %jmp;}:next{@x:=st #3;if #1=#1 then :done;goto :next;}:done{halt;}}|}
      ""
  in
  eq (I.value_print (List.assoc "x" out)) "3";
  let huge = "1234567890123456789012345678901234567890" in
  let out, _ =
    run ("@x 0\n:main{:entry{%n:=#" ^ huge ^ ";%n:=%n+#1;@x:=st %n;halt;}}") ""
  in
  eq (I.value_print (List.assoc "x" out)) (Utils.Number.succ huge);
  error (I.make (get (I.parse_code ":other{:entry{halt;}}")) []);
  error (I.make (get (I.parse_code "@x 0 @x 1 :main{:entry{halt;}}")) []);
  error (I.make (get (I.parse_code ":main{:entry{halt;}:entry{halt;}}")) []);
  error
    (I.make
       (get (I.parse_code "@x 0 :main{:entry{halt;}}"))
       [ ("y", I.Num "1") ]);
  error (I.parse_code ":main{:entry{nop;}}");
  let s = get (I.make (get (I.parse_code ":main{:entry{pop %a;halt;}}")) []) in
  error (I.step s "");
  let s =
    get
      (I.make (get (I.parse_code ":main{:entry{%a:=#255;print %a;halt;}}")) [])
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
            (I.parse_code
               ":main{:entry{if #1=#1 then :other;halt;}}:other{:entry{halt;}}"))
         [])
  in
  error (I.step s "");
  print_endline "flow_ir tests passed"
