# book
## 全体
- [ ] after.py を mdbook のバックエンドにする。
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

# counter machine
- [ ] todo 全部消す
