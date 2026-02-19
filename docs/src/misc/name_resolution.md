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

```


```
