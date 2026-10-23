# 小さな計算モデル

| モデル | registry 名 | 内容 |
| --- | --- | --- |
| [Brainfuck](brainfuck/README.md) | `brainfuck` | 8 bit セルと 8 命令、バイト列の入出力 |
| [SKI 計算](ski_calculus/README.md) | `ski_calculus` | S・K・I の左端最外簡約 |
| [P′′](p_double_prime/README.md) | `p_double_prime` | 左無限テープ上の R・λ・反復 |

いずれも一段階ずつ実行でき、途中状態の描画と snapshot の保存・復元に対応する。
RInput は空文字列。Brainfuck と P′′ は一命令、SKI は一つの簡約を一段階とし、
計算を終えた状態の次の step で最終結果を返す。

`loop_code.ml` はループの括弧対応を、`cells.ml` はセルの値とテープ・バイト列の codec を共有する。
テープ操作と描画は `Utils_data.Tape` と `Utils_runtime.Render` を使う。

[state_jump](state_jump/spec.md) は未実装の設計メモ。
