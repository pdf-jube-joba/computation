module Source = Multi_tape_turing_machine.Main
module Target = One_tape_turing_machine.Main

include
  Utils_runtime.Compiler.S
    with module Source := Source
     and module Target := Target

val api : Utils_runtime.Compiler_text_api.api
