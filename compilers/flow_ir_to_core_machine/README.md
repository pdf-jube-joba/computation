[[Compiler]]

`flow_ir → core_machine`。registry 名は `flow_ir_to_core_machine`。
全 stmt、条件分岐、goto/enter/halt、逐次入出力を変換する。

## 表現

\[
\begin{aligned}
\operatorname{Num}(n)&\mapsto(0,n,0) \\
\operatorname{Code}(l)&\mapsto(1,\operatorname{name}(l),0) \\
\operatorname{Static}(l)&\mapsto(2,\operatorname{name}(l),0) \\
\operatorname{Stack}(i)&\mapsto(3,i,0) \\
\operatorname{Heap}(h,i)&\mapsto(4,h,i)
\end{aligned}
\]

`name(l) = Number.of_bytes(Identifier.to_string(l))`。
コードラベルは `flow_ir` の名前解決後の一意な表示名を使う。
レジスタは3本、メモリセルは3セルへ展開する。

| memory | 内容 |
| --- | --- |
| `MEM_0` | 入出力レコード |
| `MEM_1` | stack：`3 * index` から3セル |
| `MEM_2` | heap：単調増加する bump pointer で領域を割り当て |
| `MEM_3` | static：宣言順の添字から3セル |
| `MEM_4` | handle 表：`3 * handle` に `(active, base, size)` |

heap handle は1から発行し、再利用しない。free は active を0にする。
未使用メモリは0なので、新規領域は `(Num 0)` で初期化済み。

## 制御と検査

- region/block を平坦化。enter の入口でソースの全レジスタを0へ戻す。
- 間接ジャンプはコード値を分岐表で解決。goto は現在の region の block、enter は region のみ。
- 数値演算・順序比較は tag 0 に限る。等値比較は三つのフィールドの一致。
- ref の作成では数値・キー種別を検査し、領域の存在や境界は load/store 時に検査する。
- stack は `index < sp`、heap は `active = 1 ∧ index < size` を要求する。
- flow_ir の `Number.to_int` / handle の制約も検査する。ホスト資源不足は意味保存の対象外。
- 検査失敗は共通エラーブロックへ。`\len MEM_0` による `Err` を使い、エラー文面は保存しない。

不正なラベル・宣言はコンパイル時に検出する場合がある。
生成コードの private register / memory を外部から変更しないことを前提とする。

## codecs

`encode_ainput` / `decode_foutput` はコードを引数に取らないので、名前付きレコードを使う。

```text
MEM_0[0]       = count
MEM_0[1+4*i]   = name
MEM_0[2+4*i]   = tag
MEM_0[3+4*i]   = payload1
MEM_0[4+4*i]   = payload2
```

初期化コードは static の宣言値を配置した後、入力レコードを順番に適用する。
未宣言 static / 不正な参照名は検査し、コード値の別名もソースと同じ規則で解決する。
halt 前に全 static を出力レコードへ書き戻す。decode は count 件を読み、名前順に返す。
レコード外の作業後のセルは出力として解釈しない。

`encode_rinput` / `decode_routput` は文字列の恒等変換。
`input` は対応するターゲットの `input` ステップで入力を読み、`print` は同じバイト列を出力する。
1ソースステップは複数ターゲットステップになるため、ステップ数や空出力の個数は保存しない。
入力列を与える実行側は、入力命令の実行に合わせて入力を進める。

## 検証

[差分テスト](../../tests/flow_ir_to_core_machine.ml) でソースとターゲットの最終出力・非空出力列・エラーの有無を比較する。
stack/heap、dangling ref、解放後アクセス、region 初期化、ラベル解決、入出力を含む。
