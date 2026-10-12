variant としてほしいのがいくつかある。
- [expr](expr/spec.md) が source 言語
- [CEK](cek/spec.md) が target 言語1
- [SECD](secd/spec.md) が target 言語2

AI の生成とあまりかみ合わないので、 `usize` でいっか。
- SECD -> flow_ir を頑張る
  - list は全部連結リストにして、ポインタ経由でアクセスする
- flow_ir で VM を書いて、 SECD をバイトコードにする
- 中間言語として使える言語を考える
