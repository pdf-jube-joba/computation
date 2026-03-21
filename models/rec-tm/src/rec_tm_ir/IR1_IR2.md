[[Compiler]] [[IR]]

[IR2 の説明](../rec_tm_ir_jump/IR2.md)

関数の呼び出しは DAG になっている（これは **IR1 に対する制限** ）ので、そのまま名前が被らないようにして展開をすればよい。
単純に展開すると名前が被るので、呼び出し**元**の関数の中で名前が一意になるようにする必要がある。

具体例：
```
fn f {
  entry:
    return;
}

fn main {
  entry:
    call f;
  next:
    call f;
}
```

ここで単純に `fn f` だけの情報から得られるブロック列を展開すると、
このようになって同じラベル名が出てきて不適。
そもそも、どこに帰るのかもわからない。
```
main-entry:
  goto f:
f:
  goto f-end:
f-end:
  goto ??:
main-next:
  goto f:
f:
  goto f-end:
f-end:
  goto ??:
main-end:
```

これを踏まえて、ある関数の flatten は次のように行う。
1. 関数内のブロックの処理
    - 関数の実行後に必ず飛ぶ、 return 先の名前を受け取っておく。
    - return 用のブロックを別の用意し、その中には受け取った名前への jump のみを入れる。
    - return/continue/break を具体的なラベル名に置き換えてすべてを jump にする。
    - return については、引数で受け取ったところに jump とする。
    - 呼び出し元が、被らないような名前空間を用意しておく。
2. 他の関数の呼び出しをそれぞれ flatten して、ブロックの列として得る。
    - 他の関数との名前の衝突を避けるためのα変換をすること。
    - 複数の呼び出しがあるケースに対応するため、呼び出し元ラベルと呼び出しの行数を一意性として使う。
3. ブロックを `flat_map` のようにして展開する。

これを `main` に対してやればよい。

こんな感じに変形するべき（呼び出し元ラベルと行数のラベルを入れる）
```
main-entry:
    goto main-entry-0-f
  main-entry-0-f:
    goto main-next
  main-entry-0-f-end:
    goto main-next
main-next:
    goto main-next-0-f
  main-next-0-f:
    goto main-next-0-f-end
  main-next-0-f-end:
    goto main-end
main-end:
```

こうして得られたブロック列は、 `call/return/break/continue` free な IR1 である。
これを変換するのは簡単で、各ラベルを行数に直しておけばよい。

<div data-model="rec_tm_ir-rec_tm_ir_jump"></div>
