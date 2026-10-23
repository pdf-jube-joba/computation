# チューリングマシン

- [1テープチューリングマシン](one_tape_turing_machine/README.md): 従来の実装。公開名は `one_tape_turing_machine`。既存の `turing_machine` も同じモデルを実行する。
- [多テープチューリングマシン](multi_tape_turing_machine/README.md): 複数のテープを同時に読み書きし、それぞれのヘッドを動かす。
- [多テープから1テープへの変換](../../compilers/turing_machine/multi_tape_turing_machine_to_one_tape_turing_machine/README.md): テープとヘッド位置を符号化し、1テープ機械の遷移表を生成する。

OCaml のライブラリ名は `one_tape_turing_machine` と `multi_tape_turing_machine`。
1テープ機械の構成用ライブラリは `one_tape_turing_machine_construction` にある。
