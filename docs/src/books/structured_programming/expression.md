## expression の導入
一度に一回の計算しかできないのはめんどくさいのでこれを改善する。
これは普通に再帰的な定義にすればいい。
手続きに渡す場合も、式を書ける（ついでに、値も書ける）ようにする。

\(\begin{aligned}
\NT{aexp} &\defeq \\
  &| \sp \NT{var} \sp | \sp \NT{number} \\
  &| \sp \NT{aexp} \sp \NT{binop} \sp \NT{aexp} \\
  &| \sp \T{call} \sp \NT{string} \sp \T{\LP} \sp \syntaxmacro{comma-separated}{aexp} \sp \T{\RP} \\
\end{aligned}\)

副作用があるような手続きを呼び出しうるので、
引数をどの順に計算していくのかが、何を行うかに影響する。
例：
```
static x = 0;
fn modify_x() {
  if x == 0 {
    x += 1;
    return 2
  } else {
    return 3
  }
}

fn main() {
  modify_x() + x
}
```

### 実装側

いちいち変数を新たに出してシンプルな計算手順にすればいい。
例えば、 `(x * y) + (z * x)` なら、 `left = x * y; right = z * x; result = left + right;` みたいに。

### 真偽値について
複雑な判定式を書きたいし、真偽値のよくある演算子も入れる。

## if 文と if 式
`if` は分岐と結びついているので、だいたい文と結びつくように思える。
それを式にするといい。
また、式にするためには、そうじゃない場合に何を用いるかがないといけない。
こんな感じ：`if n == 0 then 1 else 2` もしこれが `x = (if n == 0 then 1)` と書かれていたら、
`n == 0` の条件が満たされなかったときに `x` がどうなるかがわからない。
