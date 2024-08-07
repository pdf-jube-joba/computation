# チューリング完全性と Turing tarpit
## チューリング完全性とチューリング同値
チューリングマシンをエミュレートすることができるような計算モデルをチューリング完全という。
再帰関数を含め、これまでに出したものはすべてチューリング完全である。
また、チューリング完全な計算モデルをエミュレートできるような計算モデルもチューリング完全である。
チューリング完全であり、さらにチューリングマシンによってエミュレートできる計算モデルを、チューリング同値という。
このチューリング同値とは、すなわち、チューリングマシンと計算能力が同じということである。
適当に作った計算モデルは基本的にチューリング完全になる。

チューリング完全でない計算モデルの例としてはプッシュダウンオートマトンがある。
これはチューリングマシンの持っていたテープを、スタックと呼ばれるものに取り替えたような計算モデルであり、
そのスタックの性質から、覚えておく能力がチューリングマシンより劣るためにチューリング完全でないといえる。
テープやスタックは計算を行う中で途中経過を記録するために使われる、メモリのようなものである。
計算モデルが計算を行う中で、そのモデルの用いるメモリの性質は計算能力に大きな影響を与える。

チューリング完全であるがチューリング同値でない計算モデルの例としては、
- チューリングマシンに、外部にいきなり判定をゆだねる神託機能をつけたマシン
- テープに実数を格納できるようにして遷移関数も有限関数でなくてよいとしたチューリングマシン

がある。
どんなことをするとチューリング同値になるのか、
次節でふんわりと議論する。

## チューリング同値性に必要なこと？
どのような計算モデルがちょうどチューリング同値になるのか。

計算モデルが必要とするメモリについて、次のような直感がある。
- 再帰関数やレジスターマシンや While 言語ではメモリの数（変数の数）自体は有限個しかないが、このメモリには自然数を入れることができる。
- チューリングマシン各メモリには有限種類の記号しか入れなかったが、テープが左右無限にあった。
- ラムダ計算ではプログラム自身が変形する中で、そのプログラムの大きさに制限がかかってなかった。

このことから、メモリに対応するものは、有限な情報ならいくらでも入れれるようなメモリがあればよさそう。

また、モデル側で重要なこととして、各計算に対応する具体的なモデルは、それぞれ有限な方法で記述できる必要がある。
例えば、 遷移関数が無限に大きいバージョンのチューリングマシンや、While 言語で無限に長いプログラム列を許容したりすると、チューリングマシンでエミュレートしようとする段階で、「次のステップの計算」自体が探せずに止まらなくなったりしてしまう。

## Turing tarpit
以上のことを踏まえて、ここでは、おもしろい計算モデルや、Turing Tarpit と呼ばれるようなもののうち紹介が簡単なものを紹介する。
主にチューリング同値なものしか紹介しない。

## brainfuck

## Wang tile

## セルオートマトン

## SKI 計算

## tag system

## FRACTRAN

## Grass
