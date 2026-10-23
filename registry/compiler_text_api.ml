(* Typed model-to-model compilers can retain Source/Target modules and expose
   this text boundary to the common browser and CLI hosts. *)
type api = {
  compile_code : string -> (string, string) result;
  encode_ainput : string -> (string, string) result;
  encode_rinput : string -> (string, string) result;
  decode_routput : string -> (string, string) result;
  decode_foutput : string -> (string, string) result;
}

let make (module C : Utils_runtime.Compiler.S) =
  let convert parse transform print text =
    Result.bind (parse text) (fun value -> Result.map print (transform value))
  in
  {
    compile_code = convert C.Source.Code.parse C.compile C.Target.Code.print;
    encode_ainput =
      convert C.Source.Ainput.parse C.encode_ainput C.Target.Ainput.print;
    encode_rinput =
      convert C.Source.Rinput.parse C.encode_rinput C.Target.Rinput.print;
    decode_routput =
      convert C.Target.Routput.parse C.decode_routput C.Source.Routput.print;
    decode_foutput =
      convert C.Target.Foutput.parse C.decode_foutput C.Source.Foutput.print;
  }
