# 継続について
## ラムダ計算の復習
ラムダ計算の項は次のような文法で定義される。
（ただし、 variable と呼ばれる変数を扱うための型がすでに存在しているとする。）

```ml
type Exp = Var of Variable | Lam of Variable * Exp | App of Exp * Exp
```
