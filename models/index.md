ここはいろんな計算モデルとコンパイラを置いておくところ

プログラミング言語は基本的に、
物事をよりよく記述できるように拡張していくが、
逆に「機能を極限まで減らしつつもチューリング完全性を失っていないような小さい言語」を作るのも
（個人の趣味として）よくある。
こういうのは Turing tarpit とか esolang と呼ばれ、結構いろんな人がいろんなことをやっている。
それと、論文に出てくるような言語だと、議論したい機能に絞った言語を定義しているため、これも小さい。

そういうものを集めたい。

### 抽象的なインターフェースについて
`utils.lib.rs` にある定義はこんな感じ。

```rust
pub trait TextCodec: Sized {
    fn parse(text: &str) -> Result<Self, String>;
    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result;
    fn print(&self) -> String {
        let mut s = String::new();
        self.write_fmt(&mut s).unwrap();
        s
    }
}

pub enum StepResult<M: Machine> {
    Continue {
        next: M,
        output: M::ROutput,
    },
    Halt {
        snapshot: M::SnapShot,
        output: M::FOutput,
    },
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

    // for each steps
    // representation of the current state
    type SnapShot;
    // runtime input
    type RInput: TextCodec;
    // runtime output after a step
    type ROutput: TextCodec;

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        Self::Code::parse(code)
    }
    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        Self::AInput::parse(ainput)
    }
    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        Self::RInput::parse(rinput)
    }
    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String>;
    fn step(self, rinput: Self::RInput) -> Result<StepResult<Self>, String>;
    fn current(&self) -> Self::SnapShot;
}

pub trait Compiler: Sized {
    type Source: Machine;
    type Target: Machine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
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

また、 Json への変換 `SnapShot: Into<serde_json::Value>` が定義されているときには、
web 側で（言語によらない共通の描画機能を使って）表示される。
ただし、 `src/bin/foobar.rs` のようにバイナリを置いて、中に `web_model!` あるいは `web_compiler!` を書くこと。
以下のような Json の描画がある。

- title, className: optional な field としてすべてのものに入れてよい。
- text: `{ kind: "text", text: string }`
  - テキストを入れる。
- table: `{ kind: "table", columns: [block], rows: [{ className?, cells: [block] }] }`
  - 表。 `columns` は各列のタイトル。ブロックにしてよい。 `rows.className` はその行に適用される。
- container: `{ kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}`
  - ブロックの列。 orientation defaults to vertical, display defaults to block.

## TODO
- [] brainfuck
- [] Wang tile
- [] セルオートマトン
- [] SKI 計算
- [] tag system
- [] FRACTRAN
- [] Grass
