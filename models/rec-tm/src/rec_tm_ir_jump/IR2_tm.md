[[Compiler]]

最初から \(S\) が与えられているので、
「tape[head] が何であろうと次はこの状態に行く」が「すべての \(S\) を列挙して状態遷移」に変換できる。
あとは基本的には行数と変数環境を状態として encode すればよい。
登場する変数も実際にわかっているので、最初から空白が代入されていると考える。

具体例：
```
0: v := @;
1: w := v;
2: RT;
3: jump 0 if w = @;
```

- \(V' \defeq \{v, w\}\)
- \(Q \defeq \{0..=4\} \times (V' \to S)\)
- \(Q_{\text{accepted}} = \{4\} \times (V' \to S)\)

このもとで、
1. `v := @`     :\(([0, (v \mapsto s_1, w \mapsto s_2)], s) \mapsto ([1, (v \mapsto s  , w \mapsto s_2)], s), C\)
2. `w := v`     :\(([1, (v \mapsto s_1, w \mapsto s_1)], s) \mapsto ([2, (v \mapsto s_1, w \mapsto s  )], s), C\)
3. `RT`         :\(([2, (v \mapsto s_1, w \mapsto s_2)], s) \mapsto ([3, (v \mapsto s_1, w \mapsto s_2)], s), R\)
4. `jump ...`   :
  - \(([3, (v \mapsto s_1, w \mapsto s_2)], s) \mapsto ([0, (v \mapsto s_1, w \mapsto s_2)], s), C\) ... これを \(s_2 = s\) の場合に追加
  - \(([3, (v \mapsto s_1, w \mapsto s_2)], s) \mapsto ([4, (v \mapsto s_1, w \mapsto s_2)], s), C\) ... これを \(s_2 \neq s\) の場合に追加

<div data-model="rec_tm_ir_jump-turing_machine"></div>
