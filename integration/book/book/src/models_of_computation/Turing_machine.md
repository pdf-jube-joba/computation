# チューリングマシン
## 注意
チューリングマシンの定義は複数あり、本によって異なる。
標準的なものはないようなので、都合のいい感じで定義している。
- [ ] 参考にした文献を列挙する。

## ざっとした説明ととりあえず動く見本

チューリングマシンの構成要素はだいたい以下の通り
- 記号が書かれた左右無限のテープ
- 状態を持つ制御部
- プログラムとか遷移表とか言われるような、「状態と記号」から次の「状態と記号及びテープの動かし方」への対応一覧
このプログラムに従ってテープと制御部を動かしていくことで、テープを書き換えながら動作する。
最終的に終了状態と呼ばれる状態に行きついたら動作は終了であり、どんなテープを入力すると停止するのか、その時のテープの状態はどうか、
といった部分を計算とみなすことができる。

例えば、次の例はテープに書かれた二つの二進数の足し算を行う。

<component id="example_bin_adder">

## 定義
空白記号（ blank symbol ）と呼ばれるものを固定し、 \(\mathbb{B}\) と書く。
この元は後述する集合に含まれたりするが、チューリングマシンの定義を通して固定をしておく。
> **Note**
> 普通はこの空白記号もチューリングマシンの定義に含めるが、にもかかわらず、
> 異なるマシンで空白記号が共有されているかのような議論が多かったため、ここでは固定した。

> **definition** 次の組をチューリングマシンと言う。
> - \(\Sigma\) : \(\mathbb{B}\) を含む有限集合...この集合の元を記号という
> - \(Q\): 有限集合...この集合の元を状態という
> - \(\delta\): \(Q \times \Sigma\) から \(Q \times \Sigma \{L,R,C\}\) への部分関数
> - \(q_{\text{init}}\): \(Q\) の元...これを開始状態という
> - \(Q_{\text{fin}}\): \(Q\) の部分集合...この集合の元を終了状態という

> **Note**
> チューリングマシンの他の定義には次のものがある。
> - \(\Sigma\) の部分集合としてさらに \(\Gamma\) なる"入力記号の集合"を指定する
> - \(\delta\) の定義を全域関数とする。これについては後にフォローを行う。
> - \(\delta\) の値域を \(Q \times \Sigma \times \{L,R\}\) とする。
> - \(\delta\) の値域を \(Q \times \Sigma \cup Q \times \{L,R,C\}\) とする。
> - 終了状態の集合 \(Q_{\text{fin}}\)ではなく終了状態 \(q_{\text{fin}} \in Q\) を指定する。

今、集合 \(\Sigma\) に対して（ \(\Sigma\) 上の）テープと呼ばれるものを考える。
このテープとは、各セルに \(\Sigma\) の元が格納されたものが左右無限に並んだもののことを言う。
ただし、テープの中の有限個のセルを除いて、セルは基本的に \(\mathbb{B}\) が入っているものとする。
さらに、このテープには制御部と呼ばれるものがついていて、セルのうちの一つを指し示している。

このテープに対しては次のような操作を行うことができる。
- テープ \(T\) に対して、制御部を一つ右（ resp. 左）に動かしたテープを表す... \(\text{Right} T\) （ resp. \(\text{Left} T\) ）
- テープ \(T\) に対して、制御部の指し示すセルの中身の記号を取り出す... \(\text{Head} T\)
- テープ \(T\) と \(s \in \Sigma\) に対して、制御部の指し示すセルの中身を \(s\) で書き換える... \(\text{write} (T, s)\)

> **Note**
> テープの数学的な定義を形式的に書いているものはほとんどなかった。
> 一応次の二つの定義が考えられる。
> - 整数の集合 \(\mathbb{Z}\) から \(\Sigma\) への写像であって \(\Sigma - \{\mathbb{B}\}\) の逆像が有限集合になっているものと \(q \in Q\) の組のこと。
>  制御部は \(0 \in \mathbb{Z}\) を指し、状態 \(q\) を保持している。
> - 自然数 \(i\) と、 \(\{1 \ldots i\}\) から \(\Sigma\) への写像であって先ほどと同様の条件を満たすものと、 \(l \in \{1 \ldots i\}\) の組のこと。制御部は \(l\) を指し、状態 \(q\) を保持している。
> なんにせよ、ここで重要なのは \(\text{Right} T\) 、 \(\text{Left} T\) や \(\text{Head} T\) といった操作を行えることである。
> また、テープの定義には \(\Sigma\) が表れるのだが、テープと書いている本が多い。
> ここでは、 \(\Sigma\) 上の、と書くことでテープの定義に言及できるようにした。
> ただしい用語かはわからない。

さらに、有限制御部と呼ばれる、 \(Q\) 上の元を一つ保持するものがある。

