## 手続きの導入
全てがラベル付きブロックの列として存在していると、次のような問題がある。
- 同じ内容のコードで別の場所から飛んで戻ってくるには工夫が必要。
  - 単純な解決方法：コピペする。これはめんどくさい。
- 全てレジスタが共有されているせいで、誤って書き換えてしまう。
  - 単純な解決方法：各ブロックの規約として、書き換えられたくないものは自分で用意したメモリに書いておく。

例えば、これらの解決方法としてこんなやり方がある。
（call with continuation ?）
```
.data
@return_addr 0;
@register_save 0;

.text
@block1:
  return_addr := @block1
  register_save := %v1
  jump @callee
  %v1 := register_save

@block2:
  return_addr := @block2
  register_save := %v1
  jump @callee
  %v1 := register_save

@callee:
  do_something
  %v1 = return_addr
  jump %v1
```
こうやって static な場所に置くと \(n\) が定まっていないときの \(n\) 重の呼び出しは実現できない。
（コンパイル時に最大でも \(n\) 重の呼び出ししかないとわかっているなら、この方法でできる。）
代わりに、使われていないメモリに積むようにすればいい。

それはさておき、これと同じようなことができるようになったのが、手続きである。
```
fn block1 {
  %v1 = 0;
  call callee
  // %v1 == 0 のまま
}

fn block2 {
  %v1 = 2;
  call callee
  // %v1 == 2 のまま
}

fn callee {
  do_something
  %v1 = 3;
  return;
}
```

手続きを導入してこれらが解決できる。
- 意味側：手続きという単位を考えて、その中にブロックを入れる。
  手続き間ではレジスタを共有しない：手続きの呼び出しの前後でレジスタは勝手に保存されている。
- 実装側（上で書かれたコードを下のようなコードに直したい場合）：
  手続きを呼び出すコードを、「戻る先のアドレスとそのときのレジスタ」をメモリに保存するようにする。

この実装側のやり方の1つがスタックを使うものになる。
関数フレームと呼ばれて、どこに戻るか、どんなレジスタだったかを保存しておく。
イメージとしてはこんな感じに展開する。

```
.data
@frame_addr 0;

.macro push $x
  [ @frame_addr ] := $x
  @frame_addr += 1
.endmacro

.macro pop $x
  @frame_addr -= 1
  $x := [ @frame_addr ]
.endmacro

.text
@block1:
  push %v1
  push @block1
  jump @callee
  pop %v1

@block2:
  push %v1
  push @block1
  jump @callee
  pop %v1

@callee:
  do_something
  pop %v1
  jump %v1
```

どの順でスタックに置くか、どのレジスタをどちらが保存するか、こういう規約を ABI という。

### 引数と返り値
ところで、これでレジスタが共有されなくなったのはちょっと困る。
ブロック間で呼び出すときのパラメータをレジスタで渡したり、結果を呼び出し元に返したりできなくなるから。
ただ、これも return と同じようにして、メモリの一部を使うことで共有することができる。
これを自動的にやるのも手続きに含めたほうが良い。

例えばこんな感じ：
```
fn foo(n) {
  m := 0
  u := n
  @loop:
    jump @end if u == 0
    u := u - 1
    m := m + n
  @end:
    return m
}

fn main() {
  v1 := foo(3)
  v2 := foo(v1)
}
```

これも push/pop でお互いに取り出せる。
```
.data
@frame_addr 0;

.macro push $x
  [ @frame_addr ] := $x
  @frame_addr += 1
.endmacro

.macro pop $x
  @frame_addr -= 1
  $x := [ @frame_addr ]
.endmacro

.text
@foo:
  pop %n // 引数が push されているので、 pop して取り出す。
  %m = 0;
  %u = %n;
@foo_loop:
  jump @end if %u == 0
  %u := %u - 1
  %m := %m + n
@foo_end:
  pop %return // 戻るアドレスを取り出す。
  push %m // 返り値を push する。
  jump %return

@main:
  // call f(3);
  push @main  // 戻るアドレスを入れる
  push 3      // 呼び出す引数を入れる
  jump @foo   // 呼び出す
  pop %v1     // 返り値を取り出す
  // call f(%v1)
  push @main  // 戻るアドレスを入れる
  push %v1      // 呼び出す引数を入れる
  jump @foo   // 呼び出す
  pop %v2     // 返り値を取り出す
```

### 再帰性の問題
じゃあ、自分自身を呼び出すのはどうか？
```
fn fact(n) {
  @entry:
    jump @case0 if n == 1
    jump @other
  @case0:
    return 0;
  @other
    v := fact(n - 1) // 自分自身を呼び出している？
    w := v * n
    return w
}
```

これもできたほうが嬉しい。
スタックフレームを使うならできる。

### 値渡しと参照渡し
呼び出し先が受け取ったものを書き換えるとどうなる？
```
fn f(n) {
  n := 0
  return
}

fn main() {
  k := 1
  f(1)
  // k == 0 ? k == 1 どっち？
}
```

- 値渡し：呼び出すときに、引数として渡した何かに入っている値を渡すと考える。上のプログラムでは k は 0 になる。
- 参照渡し：呼び出すときに、引数として変数自体を渡すと考える。上のプログラムでは k は 1 になる。

> [!Tip]
> 関連した用語：値呼び、参照呼び、名前呼び
> 名前呼びだけ、最初から thunk を使っていることが前提っぽい。
> value への参照を渡していると解釈するのであれば、参照渡し。
> 参照値として渡していることもある。
> 明確に言語側に参照値を表す構文があるなら、その言語は値呼びだ。
> そういうのがなくて全て参照として表されているなら、参照呼びだ。
