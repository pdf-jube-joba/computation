# Turing machine（ざっくり）
Turing machineの構成要素はだいたい以下の通り

- **記号**が書かれたテープ（ちょうど前にあるテープしか読み書きができない。）
- **状態**を持つ制御部
- プログラム（とか遷移表と言われる）：「状態・記号」から次の「状態・記号・テープの動かし方」への対応一覧

このプログラムに従ってテープと制御部を動かしていくことで、テープを書き換えながら動作する。
最終的に終了状態と呼ばれる状態に行きついたら動作は終了であり、
どんなテープを入力すると停止するのか、その時のテープの状態はどうか、
といった部分を計算とみなすことができる。

[ちゃんとした定義](/models/turing_machine/index.html)

## 具体例

テープにある `a` を全部 `b` に書き換える例。
テープの最後には `c` をつけること。
<div data-model="turing_machine">
<script type="text/plain" class="default-code">
start
goal
a,start,b,start,R
b,start,b,start,R
c,start,c,goal,C
</script>
<script type="text/plain" class="default-ainput">
-|a|b,a,b,b,c
</script>
</div>

テープにある `a` と `b` の"仕分け"を行う例。

<div data-model="turing_machine">
<script type="text/plain" class="default-code">
s
g
a,s,a,s,R
-,s,-,g,C
b,s,b,k,R
b,k,b,k,R
-,k,-,g,C
a,k,b,b,L
b,b,a,r,L
b,r,b,r,L
a,r,a,r,L
x,r,x,s,R
</script>
<script type="text/plain" class="default-ainput">
x|b|a,a,b
</script>
</div>

## 具体例
二進数で書かれた自然数に1を足すマシンを作ってみる。
考え方としては、下の桁から順に見て行けばよい。
\(q_{\text{init}}\) から左に行き、 \(0\) が出てきたら \(1\) に書き換えて終わる。
\(1\) が出てきたら \(0\) に書き換えて、さらに繰り上がりモードを継続する。

遷移関数は次のようにすればいい（もっと簡単にできるはず）。

| key_sign | key_state | value_sign | value_state | value_direction |
| --- | --- | --- | --- | --- |
| ! | start | ! | add_1| L |
| 0| add_0| 0| add_0| L |
| 1| add_0| 1| add_0| L |
| 0| add_1| 1| add_0| L |
| 1| add_1| 0| add_1| L |
| !| add_0| !| endst| C |
| !| add_1| 1| write| L |
| -| write| !| endst| L |

ここでは記号や状態、初期状態と終了状態を省略した。
