ここはいろんな計算モデルとコンパイラを置いておくところ

## Turing tarpit について
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
pub trait Machine: Sized {
    type Code: TextCodec; // static code
    type AInput: TextCodec; // ahead of time input
    type SnapShot; // representation of the current state
    type RInput: TextCodec; // runtime input
    type Output: TextCodec; // output after a step

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
    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::SnapShot;
}

pub trait Compiler: Sized {
    type Source: Machine; // source code
    type Target: Machine; // target code

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String>;
    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String>;
    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String>;
    fn decode_output(
        output: <<Self as Compiler>::Target as Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String>;
}
```

また、 Json への変換 `SnapShot: Into<serde_json::Value>` が定義されているときには、
web 側で（言語によらない共通の描画機能を使って）表示される。
ただし、 `src/bin/foobar.rs` のようにバイナリを置いて、中に `web_model!` あるいは `web_compiler!` を書くこと。
以下のような Json の描画がある。

- Supported block kinds: title, className are optional for all block types.
- text: { kind: "text", text: string }
  - simple text block
- table: { kind: "table", columns: [block], rows: [{ className?, cells: [block] }] }
  - a table with optional header and rows. each cell can be a block.
- container: { kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}
  - a flat displayed container for grouping blocks. orientation defaults to vertical, display defaults to block.

## TODO
- [] brainfuck
- [] Wang tile
- [] セルオートマトン
- [] SKI 計算
- [] tag system
- [] FRACTRAN
- [] Grass
