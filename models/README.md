ここはいろんな計算モデルとコンパイラを置いておくところ

## 抽象的なインターフェースについて
`utils/lib.rs` にある定義はこんな感じ。

```rust
pub enum StepResult<M: Machine> {
    Continue { next: M, output: M::ROutput },
    Halt { output: M::FOutput },
}

// trait for models of computation
pub trait Machine: Sized {
    // "semantics" of the models
    // a model can be considered as a partial function (Code, AInput) -> FOutput
    // static code
    type Code: TextCodec;
    // ahead of time input
    type AInput: TextCodec;
    // final output at halt
    type FOutput: TextCodec;

    // small step semantics of the models
    // runtime input (some kind of "effect" or "interaction" with the outside world)
    type RInput: TextCodec;
    // runtime output (some kind of "effect" or "interaction" with the outside world)
    type ROutput: TextCodec;

    // representation of the current state
    type SnapShot: serde::Serialize + serde::de::DeserializeOwned;

    // parsing
    fn parse_code(code: &str) -> Result<Self::Code, String> {
        Self::Code::parse(code)
    }
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        Self::AInput::parse(ainput)
    }
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        Self::RInput::parse(rinput)
    }

    // semantics
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String>;

    // save/restore snapshot of the current state
    fn snapshot(&self) -> Self::SnapShot;
    fn restore(snapshot: Self::SnapShot) -> Self;

    // rendering of Snapshot for web
    fn render(snapshot: Self::SnapShot) -> RenderState;
}

pub trait Compiler: Sized {
    type Source: Machine;
    type Target: Machine;

    // compile of code
    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
    // encoding/decoding of inputs and outputs ... how to interpret the source/target code as the same "program"?
    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String>;
    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String>;
    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String>;
    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String>;
}
```

これを受けての wasm interface はこんな感じ。
```
package computation:web;

world model {
  export make: func(code: string, ainput: string) -> result<_, string>;
  export step: func(rinput: string) -> result<string, string>;
  export snapshot: func() -> result<string, string>;
  export restore: func(snapshot: string) -> result<_, string>;
  export render: func(snapshot: string) -> result<string, string>;
}

world compiler {
  export compile-code: func(input: string) -> result<string, string>;
  export encode-ainput: func(ainput: string) -> result<string, string>;
  export encode-rinput: func(rinput: string) -> result<string, string>;
  export decode-routput: func(output: string) -> result<string, string>;
  export decode-foutput: func(output: string) -> result<string, string>;
}
```

`RenderState` は web 側で（言語によらない共通の描画機能を使って）表示するための構造体で、 json としてこんなものに翻訳される。
- title, className: optional な field としてすべてのものに入れてよい。
- text: `{ kind: "text", text: string }`
  - テキストを入れる。
- table: `{ kind: "table", columns: [block], rows: [{ className?, cells: [block] }] }`
  - 表。 `columns` は各列のタイトル。ブロックにしてよい。 `rows.className` はその行に適用される。
- container: `{ kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}`
  - ブロックの列。 orientation defaults to vertical, display defaults to block.

## TODO
プログラミング言語は基本的に、
物事をよりよく記述できるように拡張していくが、
逆に「機能を極限まで減らしつつもチューリング完全性を失っていないような小さい言語」を作るのも
（個人の趣味として）よくある。
こういうのは Turing tarpit とか esolang と呼ばれ、結構いろんな人がいろんなことをやっている。
それと、論文に出てくるような言語だと、議論したい機能に絞った言語を定義しているため、これも小さい。

そういうものを集めたい。
- 抽象機械
    - [] SECD
    - [] CEK
    - [] ZINC
    - [] CAM
    - [] spinless tagless G machine
- 表示が難しそう
    - [] Wang tiles
    - [] セルオートマトン
- その他
    - [] brainfuck
    - [] SKI 計算
    - [] tag system
    - [] FRACTRAN
    - [] Grass
    - [] pure lisp
