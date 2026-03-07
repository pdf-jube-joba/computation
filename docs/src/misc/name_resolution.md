## 名前解決をどうする？

```
<function>  ::= "fn" <name> "{" <stmt>* "}"
<stmt>      ::= ...
    | call <name>

<program>   ::= <function>*
```

ぐらいの構文の言語を考える。
内部の表現について。

## 1. 素直に文字列にする。

```
enum Function {
  name: String,
  body: Vec<Stmt>,
}

enum Stmt {
  ...
  Call(String),
}
```

### 問題点

実行に関数のレジストリが必要になる。
```
struct Registry {
  table: HashMap<String, Function>
}

fn eval(function: Function, registry: Registry) -> Output {
  // ...
}
```
1. 同じ関数名のものが 2 つあったらどうなるか？存在しない関数名の場合はどうなるのか？
2. 間違えて更新前のレジストリを使ってしまわないか？ ... レジストリが singleton になっていることを保証したい。
3. レジストリへの登録作業の仕方は？レジストリ側のファイルを書き換える必要がある？

以下は 3. で上げた話のつらさ。
内部からこの言語を書くときに、関数を結び付ける手段があまりない。
```
fn make_fun() -> Function {
  Function {
    function: "make_fun",
    body: vec![],
  }
}

fn make_some_function() -> Function {
  Function {
    function: "main",
    body: vec![
      Stmt::Call("make_fun") // この Call が上と同一であることをどう保証する？こんなに近くにあるのに。
    ],
  }
}
```

レジストリが必要問題は、 GlobalID を使うにしても問題になる。
どの辞書を引いてくるのか問題はこの方向性なら避けられない。

## 2. 構文解決後に名前解決をして、その後の言語を扱う

```
enum Function {
  name: String,
  body: Vec<Stmt>,
}

enum Stmt {
  ...
  Call(Function),
}
```

悪くない、というか、多くの toy lang ではこう書いている気がする。

### 問題点
内部で生成する場合でも、コードから AST に直す場合でも、毎回新しい関数が作られる。
```
fn make_fun() -> Function {
  Function {
    function: "make_fun",
    body: vec![],
  }
}

fn make_some_function() -> Function {
  Function {
    function: "main",
    body: vec![
      Stmt::Call(make_fun())
    ],
  }
}
```

実質的に、同じ関数が毎回コピーされることになるのがいやかも。
単純な解決策の参照：ライフタイム周りで嫌な思いをしそう。
`rust` の場合。 `rust` じゃない場合は、GC ありならわからない。
`new MakeFun()` なら毎回生成している？

## 3. Rc と static を使う？
AI による提案：
- `Stmt::Call(Rc<Function>)` にする
  - 実際、関数定義自体を可変にする意味はないので、よく見る `RefCell` にする必要がない。
- レジストリは static にする。
  - ID の管理体が singleton になるし、関数の signature にレジストリを渡さないので、あやまって"前"のレジストリを渡さなくて済む。
- `fn name() -> Rc<Function>` のように関数を生成する側は、レジストリを見て存在する場合は `.clone()` する。
- 内部で書くときは、 `Stmt::Call(make_make_fun())` みたいに書けばいいことになる。

## 一次元情報からの復元について？
今いくつかの問題に直面している。
1. AI agents は REPL を使わないので、 CLI oriented で書けるようにしたい
  - REPL ならこう：
    ```
    $machine model example_counter
    code> ..
    ainput> ..
    [snapshot] ..
    rinput> ..
    [snapshot] ..
    ```
  - CLI の使い方はこんな感じ：
    ```
    $machine model example_counter create --code ".." --ainput ".."
    [snapshot] ...
    $machine step --rinput ".."
    [snapshot] ...
    ```
    全然伝わらなそう。
    ともかく、毎回コマンドを打つと exit で返ってきて、かつ背後に状態がある。
  - `trait Machine` 自体に Serialize/Deserialize を仮定しないなら、状態を持つのはプログラムを起動させ続けるしかない： daemon とか API とか。
    - `Operation not permitted` が出るので使わないらしい。
  - Serialize/Deserialize を仮定するなら dump とそこからの復元ができるということになるのでこんなことができるようになる
    - ファイルで状態を (implicit/explicit に) 保存できる ... CLI コマンド自体は stateless にできる（前回の状態を保存するのは外部だから）
    - stdin/stdout で pipe に流せるので、つなげるようにもできる： `machine create | machine step`
2. AST は名前解決前なので `Call(String)` になっているが、扱いたい本質な対象の形は `Call(Function)` 
  - `Call(Function)` と書くと tree のように見えるが、本当は graph になっているべき。
  - `Call(Function)` の方から Snapshot を書くと、また AST っぽいものに戻ってしまう。
    - Snapshot の方にID附番とか registry を付けるとかしてもいいけれど、それがなにかしらの validity を失っている。
      - 普通に `Call(Function)` をそのまま展開する...もとは同じ名前で呼ばれていた関数が複数あることになる？
        `fn f(){} fn g(){call f; call f}` とかは `fn g() { call {..}; call {..}` で二重に展開される
      - 名前に戻すことは工夫すればできる。
      - それ以外の問題：`Call(Function)` 側は順序がいらないが、 `Call(String)` 側は順序がついていた状態が自然（一次元のASTに対応しているので）。
    - graph としてそのまま乗っけることもできる。でも、外からやってきた graph の validity は？

これの 2. についてもう少し詳しく書く。
`\x. \x. x` というラムダ式を考える。
「`x` が2番目のラムダ抽象にくっついている」ことが表されているデータが（ `Call(Function)` と同じような）"解決後"のデータになっている。
解決後を `\x[0]. \x[1]. x[1]` みたいに附番されていると考えてもいい。
解決前のデータは `\x. \x. x` という文字列かもしれないし、それこそ `Call(String)` みたいに表現されたデータかもしれない。
解決後のデータと解決前のデータはそれぞれ変換ができると思っていい。（何かしら変なデータが入らないように `Err` を出すことはあるかも。）

じゃあ解決後のデータの保存はというと、 `\x[0]. \x[1]. x[1]` みたいに文字列をそのまま書いてもいいが、
ちょっとこれ以上書くのはめんどくさくなってきたな。

結論としては、
- `Machine`: 内部の状態 == 解決後の状態
  - 直接文字列とか構造化データとして外部に保存することは考えずらい。
  - `Call(Function)` っぽい
- `Snapshot`: 内部の状態の"表現" == 解決前の状態に近い
  - print/parse で構造化データとして復元ができる
  - 構文木に落としただけとか、附番の validity を確かめる前
  - `Call(String)` っぽい
- `Machine -> Snapshot, Snapshot -> Machine` は多少意味のつながりが失われてもよくて、復元されればいい

> [!Note]
> trees that grow や AST expression problem とも関係があるかも
> これらは毎回書くのがだるいよねみたいな話だが、お互いの復元についても書いていたかも。
