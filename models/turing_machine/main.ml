include Types
module Sign = Sign
module State = State
module Direction = Direction
module Tape = Tape
module Code = Code
include Execution

let unwrap = Utils_base.Error.get
let snapshot = Snapshot.encode
let restore = Snapshot.decode
let render = View.render

module Ainput = Tape
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Ainput
