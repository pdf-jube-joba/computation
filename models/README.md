# 計算モデル

各パッケージの `main.mbt` にある `model()` が入口。
構文・状態・遷移はモデルの型で定義し、[utils/runtime/session.mbt](../utils/runtime/session.mbt) に渡す。

`make` は初期状態、`step` は `Continue(次の状態, 実行時出力)` または `Halt(最終出力)` を返す。
snapshot の JSON 変換と検証、描画も各モデルにある。状態の保持は共通の session が担当する。

共通の字句解析は `utils/parsing/`、環境・テープ等は `utils/data/`、
名前と束縛は `utils/names/`、描画 JSON の構築は `utils/base/`。
ビルド対象は [registry/components.json](../registry/components.json) にある。

組み込みのキーワードには `\\` を付ける。`\\if` はキーワード、`if` は識別子。
モデルごとの構文と意味論は、各ディレクトリの README / spec を参照。
