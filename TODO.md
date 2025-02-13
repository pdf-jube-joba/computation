# book
## 全体
- [ ] after.py を mdbook のバックエンドにする。
- [ ] github.io を　
## models of computation
- [ ] 参考にした文献を列挙する。
- [ ] 万能チューリングマシンを、できたら構成する。
- [x] 最後にチューリングマシンを試す部分を作る。
- [x] ラムダ計算で例を出す。
- [x] ラムダ計算で Church encoding と computability について触れる。
- [x] 再帰関数のコンパイル部分の playground 作る。
    - [x] ラムダ計算
    - [x] Turing machine

# 全体
- [ ] view について書き方を統一する。
- [ ] yew で Propoerties を derive するときに警告が出るのを、なんとかする（ cargo check ではエラーが出ない？）
- [x] 自然数や変数については統一したクレートを作る。
    - [x] 自然数のモジュール作る
    - [x] 再帰関数をそっちにする -> やめた方がいい気がする
- [x] 自然数から usize への変換は tryfrom を使う。

# recursive function
- [x] 再帰関数を定めるための文法とパーサーを作る。
    - 新たに構造体を作って json とクレートを使ってパーサーを書く。
- [ ] 再帰関数の描画部分では、重い計算をバックグラウンドで動かすように変更する。

# turing machine
- [x] チューリングマシン内で、ビルダーの作り方を、パースを行う部分と解釈を行う部分を分ける。
- [ ] 解釈について、 Fn trait と Clone を使えないか試す。

# lambda calculus
- ラムダ計算をもっと idiomatic に？
    - [x] ベータ簡約基かどうかを判定する関数を作り、 is_normal をそれを用いる形にする。
    - [x] ベータ簡約についても同様。
- [x] yew の Properties を導入する。
- [ ] テスト書く。
- [ ] パース部分をもっと楽に書けるようにする
    - [ ] 名前付き
    - [ ] 不動点による定義

# while-minus-lang
- [ ] eval を実行している行を指定した実行状況を作る。

# counter machine
- [ ] todo 全部消す

# hardware description language
- 名前の解決をする部分と意味論を各部分は分けたほうがいい気がする。それを使えば型に名前を付けるのが楽になる。
- [ ] 定義を変更して module の名前から引っ張ってくる部分を変える。
- [ ] 型の別名を付けられるようにする。

# continutations
- [ ] proc macro を使って簡単にする
