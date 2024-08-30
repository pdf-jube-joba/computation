# 継続について
## ラムダ計算の復習
ラムダ計算の項は次のような文法で定義される。
（ただし、 variable と呼ばれる変数を扱うための型がすでに存在しているとする。）

```fsharp
type Exp =
    | Var (x: String)
    | Lam (x: String, e: Exp)
    | App (e1: Exp, e2: Exp)
```

ラムダ計算を前に考えたときはβ簡約とβ正規形（ベータ簡約ができない項）を考えたが、ここからは call-by-value で left-to-right な評価戦略を考えたい。
そう考えたいので、ラムダ項は真ん中の形をしているとき値（ value ）という。

次の簡約規則により、 call-by-value で left-to-right な評価戦略を定める。
ここで、 `subst(e, x, t)` は（変数の束縛とかをちゃんと考えた）代入である。
- base-case: ` App (Lam x e) v -> subst(e, x, v)` ただし `v` は value 
- eval-r-case: `App v1 e2 -> App v1 e2'` ただし `v1` は value で `e2 -> e2'`
- eval-l-case: `App e1 e2 -> App e1' e2` ただし `e1 -> e'`

base-case に現れるような形をしたラムダ項を redex という。
以降では、 `r` を redex 、 `v` を value を表すメタ変数とする。
簡約規則を見ると全ての `e -> e'` なる関係は base-case から eval-l か eval-r を繰り返し用いて得られ、`e -> e'` を与える理由付けは一意である。
つまり、 `e -> e'` を与える証明を証明木に直せばそれは一意になっている。
特に、 `e -> e'` は（部分）関数を定めている。
通常のβ簡約関係の定義とはいろいろ異なっていることがわかる。
例えば、ラムダ項が redex を部分項に持っていても reduction ができるとは限らない。
（ `Lam x (r: redex)` は reduction されないなど。）

（疑似言語ではあるが、）ここから単純に再帰関数を使えば簡約が定まる。

```fsharp
step: Exp -> Option Exp := function
    Var _ = None,
    Lam _ _ = None,
    App (Lam x e) e2 =
        if is_value(e2) then
            subst(e, x, e2)
        else
            App (Lam x e) step(e2)
    App e1 e2 = App step(e1) e2
```

## 継続概念について
項を redex になっている部分と残りの部分に分けることを考える。
この"残りの部分"はどのように表現されるか。
具体例として `t = App v1 (App r e)` の簡約を考えると、 `r` が `r'` へと簡約されるのであれば、 `t` は `App v1 (App r' e)` へと簡約される。
これを `let f = fun m -> App v1 (App m e)` なる \(r\) の"外側"を用いて `t = f(r) -> f(r')` と簡約されたととらえることができる。
これを踏まえると、 項を redex と外側に分けるつぎの関数が得られる。

```fsharp
decomp: Exp -> Option (Exp, Exp -> Exp) := function
    Var _ = None,
    Lam _ _ = None,
    App e1 e2 =
        if is_value(e1) && is_value(e2) then
            (App e1 e2, fun m -> m) 
        else if is-value(e1) then
            let (r, f) = decomp(e2)
            (r, fun m -> App e1 f(m))
        else
            let (r, f) = decomp(e1)
            (r, fun m -> App f(m) e2)
```
分解しただけなので、 `decomp(m) = (r, f)` なら `m == f(r)` である。
`r` は redex なので `r -> r'` が base-case に従って得られる。

この decomp 関数の値域にあらわれるデータをもう少しちゃんと考えたい。
`Option (Exp, Exp -> Exp)` の値であるが、 `Exp` 部分は redex しか現れず、 `Exp -> Exp` もこの型をもつどんな関数でも表れるわけではない。
より適切なデータ型を与えようとすると、それが評価文脈になる。

```fsharp
type Cxt =
    | Hole
    | EvalL (e: Exp, c: Cxt)
    | EvalR (v: {e: Exp | is_value(e)}, c: Cxt)
```
直感的には、 `fun x -> App e1 f(x)` なる関数を覚えておくために必要なデータ型が `EvalR e1 f` であり、 `fun x -> App f(x) e2` なる関数を覚えておくために必要なデータ型が `EvalL e2 f` である。
以降で必要な関数をいくつか導入するが、順序が人によっては入れ替わっているように感じるかもしれない。

`f: Cxt` と `m: Exp` に対して `f(m)` は次のように定義される。
```fsharp
let plug: Cxt -> Exp -> Exp := function
    Hole t = t
    (EvalL e c) t = plug c (App t e)
    (EvalR v c) t = plug c (App v t)
```
この対応のもとで、評価文脈 `f` を受け取って評価文脈 `fun x -> App e1 f(x)` を返す関数は次のように書ける。
```fsharp
let extend_l: Exp -> Cxt -> Cxt := function
    e1 Hole = EvalL e1 Hole
    e1 (EvalL e c) = EvalL e (extend_l e1 c)
    e1 (EvalR v c) = EvalR e (extend_l e1 c)
```
`fun x -> App f(x) e2` を返す関数は次のように書ける。
```fsharp
let extend_r: Exp -> Cxt -> Cxt := function
    e2 Hole = EvalR e2 Hole
    e2 (EvalL e c) = EvalL e (extend_r e2 c)
    e2 (EvalR v c) = EvalR e (extend_l e2 c)
```

また redex を表すデータ型を次のように与える。
```fsharp
type Red =
    Red (x: variable, e: Exp, v: {e: Exp | is_value(e)})
```
redex に関して次のような写像がある。
- redex をラムダ項に戻す関数 `as_lam: Red -> Exp`
- ラムダ項が redex かを判定する関数 `is_rdx: Exp -> Option Red`
- redex の簡約を進める `step: Rdx -> Exp`

この状態で再度 decomp を定義すると次のように書ける。
```fsharp
let decomp: Exp -> Option (Red, Cxt) := fun e -> 
    match is_rdx(e)
    Some r -> Some (r, Hole)
    None ->
      match e
      Var _ = None
      Lam _ _ = None
      App e1 e2 =
        if is_value(e1) then
          let (r, c) = decomp(e2)
          Some (r, extend_r e1 c)
        else
          let (r, c) = decomp(e1)
          Some (r, extend_l e2 c)
```
また、`step`は次のように書ける。
```fsharp
let step: Exp -> Option Exp := fun e ->
    let (r, c) = decomp(e)
    plug(c, step(r))
```

注意点として、この状態で次のように簡約関係を定義してみると、これは先ほど定義した簡約関係とは性質が異なる。
- base-case: `App (Lam x e) v -> subst(e, x, v)`ただし `v` は value
- cxt-case: `plug(c, e) -> plug(c, e')` ただし `e -> e'`

性質としては、 `M -> M'` なる関係自体は部分関数的であるが、その理由付けとしての証明木自体には一意性がなりたっていない。
（つまり、 `E1 != E2` と `e1 != e2` を用いて `plug(E1, e1) = plug(E2, e2)` のようにかけ `e1 -> e1'` かつ `e2 -> e2'` となるような項が簡単に作り出せる。
ただし、その場合でも `plug(E1, e1') = plug(E2, e2')` が成り立つ。）
この点を考慮するのであれば、（多分）次の定義のみでよくなる。
- cxt-case: `plug(c, r) -> plug(c, r')` ただし `r = App (Lam x e) v` で `is_value(v)` であり、このとき `r' = subst(e, x, v)` とする。

## stack に直す
`extend_*` をわざわざ使ったり、 `plug` の定義の仕方だったり、素直に考えれば `Cxt` に対する"解釈"を逆にすればもっと簡単に定義できそうに思える。
例えば、 `plug` を `Hole` を探してそこに `t` を代入する関数にすれば、 `decomp` の定義で `extend_*` を使わずにそのままコンストラクタに入れれる。
この定義を採用した理由が stack との関係によるもののためである。
具体例として `m = App v (App r e)` を考えると、ここからえられる `Cxt` は `(EvalL e (EvalR v Hole))` である。
`m` での `e` と `v` の登場する順番と `Cxt` での `e` と `v` の登場する順番はひっくり返っている。
一方で、 plug を見れば確かに正しい。
```
plug (EvalL e (EvalR v Hole)) r
-> plug (EvalR v Hole) (App r e)
-> plug Hole (App v (App r e))
-> (App v (App r e))
```
このように `r` の"内側から見ることで" `fun x -> App x e` と `fun x -> App v x` を適用したと解釈するのが評価文脈である。

この観点で `Cxt` の型定義に注目すると、これは次の Frame なるデータ型を用いた List として表せる。
```fsharp
type Frame =
    | EvalL (e: Exp) // ([] e) と書く。
    | EvalR (v: {e: Exp | is-value(e)}) // (v []) と書く。
```

こうすれば、 `plug` や `decomp` や `step` は次のように解釈できる。
```fsharp
let plugf: Frame -> Exp -> Exp = function
    EvalL e t = App t e
    EvalR v t = App v t

let plug: List Frame -> Exp -> Exp = function
    [] t = t
    f::c t = plug(plugf(f, t), c)

let extend: Frame -> Cxt -> Cxt = function
    f [] = f
    f f'::c = f'::(extend(f, c))

let decomp: Exp -> Option (Red, List Frame) =
    match is_rdx(e)
    Some r -> Some (r, [])
    None ->
      match e
      Var _ = None
      Lam _ _ = None
      App e1 e2 =
        if is_value(e1) then
            let (r, c) = decomp(e2)
            Some (r, extend((EvalR e1), c))
        else
            let (r, c) = decomp(e1)
            Some (r, extend((EvalL e2), c))
```

もう少し decomp を見通しよく書くこともできる。
```fsharp
let decomp0: Exp -> List Frame -> Option (Red, List Frame) = fun e l ->
    match is_rdx(e)
    Some r -> Some (r, l)
    None ->
        match e
        Var _ = None
        Lam _ _ = None
        App e1 e2 =
            if is_value(e1) then
                decompf(e2, (EvalR e1)::l)
            else
                decompf(e1, (EvalL e2)::l)
let decomp: Exp -> Option (Red, List Frame) = fun e -> decomp0(e, [])
```
これがコールスタックに対応している。
実際、ラムダ項の実行は次のように思える。
```fsharp
type State = State (e: Exp, c: List Frame) // <e | c> と書く。
```

```
<v' | ([] e)::c> ->_("plug") <App v' e | c>
<v' | (v []::c)> ->_("plug") <App v v' | c>
<r | c> ->_("step") <r' | c>
<App v e | c> ->_("decomp") <e | (v [])::c>
<App e e' | c> ->_("decomp") <e | ([] e')::c>
```
このとき、 `->_*` は部分関数になっている。
`->_("plug")` をできる限り行うのが plug 関数で、 `->_("step")` を行うのが step 関数で `->_("decomp")` を行うのが decomp 関数である。
## CPS変換について
- [ ] ラムダ項のCPS変換とスタックの関係について調べる。

# abort/control によるラムダ計算の拡張
## motivative な例
計算の残りの部分をプログラムが扱えるようにすることで、例えば必要のない関数のリターンを読み飛ばすことができる。
次の関数を考える。
```fsharp
let prod: List Number -> Number = function
    | [] -> 1
    | x::xs -> x * prod(xs)
```
これは自然数のリストを受け取ってその積をとる関数である。
もしリストに `0` が含まれるならその時点で `0` を返してよい。
```fsharp
let prod: List Number -> Number = function
    | [] -> 1
    | x::xs -> if x == 0 then 0 else x * prod(xs)
```

ただこれでも次のように必要のない積は含まれる。
例えば、最後の3行は計算する必要がない。
```
prod [1, 2, 0, 3]
-> if 1 == 0 then 0 else 1 * prod [2, 0, 3] 
-> 1 * prod[2, 0, 3]
-> 1 * if 2 == 0 then 0 else 2 * prod[0, 3]
-> 1 * (2 * prod[0, 3])
-> 1 * (2 * if 0 == 0 then 0 else prod [3])
-> 1 * (2 * 0)
-> 1 * 0
-> 0
```
これを一気に `0` にステップを進めるようにしたい。
"次に何をやるか"の情報を前の節と同じようにスタックに管理しておけば、
次のような操作を行いたいということがわかる。
```
< prod [1, 2, 0, 3] | []> // c は
-> < if 1 == 0 then 0 else 1 * prod [2, 0, 3] | []>
-> <1 == 0 | (if [] then 0 else 1 * prod [2, 0, 3])::[]>
-> <false | (if...)::[]>
-> <if false then 0 ... | []>
-> <1 * prod [2, 0, 3] | []>
-> < prod[2, 0, 3] | (1 * [])::[]>
->_* <prod[0, 3] | (2 * [])::(1 * [])::[]>
->_* <if true then 0 else prod [3] | (2 * [])::(1 * [])::[]>
-> <0 | []> // blowup stack
```
これをしたいだけなら、ラムダ項を abort と呼ばれる項で拡張すればいい。
項の簡約としては `< Abort e | c > -> < e | []>` という風に理解できる。
また、もし計算の残りの部分を陽に扱いたい場合には control と呼ばれる項で拡張すればいい。
この control は `< Control e | c > -> < App e k | []>` ただし `k = Lam z (Abort c(z))`
のように簡約される。
`k` に `Abort` が入るのはその方がうまくいくから。
（ここもっとちゃんとした説明が欲しい。）

## 定義
```fsharp
type Exp =
    | Var (x: String)
    | Lam (x: String, e: Exp)
    | App (e1: Exp, e2: Exp)
    | Abort (e: Exp)
    | Control (e: Exp)
```
みたいにして、
- cxt-case: `plug(e, r) -> plug(e, r')`
- abort-case: `plug(e, Abort(M)) -> M`
- control-case: `plug(e, Control(M)) -> App M (Lam y plug(E, y))`
みたいな感じに定義する。
- [ ] ちゃんと定義する。

# grab/delimit によるラムダ計算の拡張
abort/control の具体例だと prod を内部で使っているだけの関数のスタックも壊されてしまう。
スタックに区切りを入れるのが grab/delimit である。
項の定義はそこまで変わらない。
```fsharp
type Exp =
    | Var (x: String)
    | Lam (x: String, e: Exp)
    | App (e1: Exp, e2: Exp)
    | Delimit (e: Exp)
    | Grab (k: String, e: Exp)
```
value の定義は変わらないが、 redex を定義するために次に定義する PureCxt が必要になる。

```fsharp
type PureCxt =
    | Hole 
    | EvalL (e: Exp, E: PureCxt) 
    | EvalR (v: {e: Exp | is_value(e)}, E: PureCxt) 

type Cxt =
    | Hole 
    | EvalL (e: Exp, E: Cxt) 
    | EvalR (v: {e: Exp | is_value(e)}, E: Cxt) 
    | Delim (e: Cxt)
```
`plug` 関数の実装は省略する。

簡約を定める関係式としては次のようになる。
- base-case: `App (Lam x e) v -> subst(e, x, v)`
- delim-case: `Delim v -> v`
- grab-case: `Delim plug(c, Grab k e) -> subst(e, k, Lam y plug(c, y))` ただし `c` は PureCxt
- cxt-case: `plug(c, e) -> plug(c, e')` ただし `e -> e'` かつ `c` は Cxt

この関係については、部分関数性は成り立つが、証明木の一意性は成り立たない。
また、 "grab" が現れることで単純な再帰関数を用いた step の実装が難しくなっている。
例えば次のように実装を書いてみる。

```fsharp
let step: Exp -> Option Exp := function
    | Var _ = None,
    | Lam _ _ = None,
    | App (Lam x e) e2 =
        if is_value(e2) then
            subst(e, x, e2)
        else
            App (Lam x e) step(e2)
    | App e1 e2 = App step(e1) e2
    | Delim e =
        if is_value(e) then
            e
        else
            Delim (step)
    | Grab k e = ?
```
ここでどのように Grab を書こうとしても、再帰的に step が呼ばれる中で対応する Delim の情報が（このコード上には）残っていないために、実装を行うことができない。
（当然、ホスト言語のコールスタックには残っているが、それを陽に扱わなければいけない。）

これを解決するため、以降では順に必要なデータ型を（通常のラムダ計算のやり方にのっとり）定義していく。
redex は次のようなデータ型である。
（右側はラムダ式への変換を表す。）
```fsharp
type Rdx =
    | Red (x: String, e: Exp, v: {e: Exp | is-value(e)})  // = App (Lam x e) v 
    | Delim (v: {e: Exp | is-value(e)})  // = Delim v \
    | DelGrab (c: PureCxt, k: variable, e: Exp) // = Delim (plug(c, (Grab k e)))
$
次の関数が定義できる。
- `as_lam: Rdx -> Exp`
- `is_rdx: Exp -> Option Rdx`
ここで、 `as_lam` が単射であることが重要である。
また、そのいわば逆写像である `is_lam` もプログラムとして書ける。
そのさいに必要な `extend_l` や `extend_r` は変わらない。

次の step 関数が簡約における cxt を除いた関係を担っている。
```fsharp
let step: Rdx -> Exp := function
    | Rdx x e v = subst(e, x, v)
    | Delim v = v
    | DelGrab c k e =
        let cont = Lam y (Delim plug(c, y))
        subst(e, k, cont)
```

decomp の定義は以下である。
```fsharp
let decomp: Exp -> Option (Rdx, Cxt) := fun t ->
    if is_value(t) then
        None
    else if is_redex(t) then
        let r = is_rdx(t)
        Some (r, Hole)
    else match t with
        | Var _ | Lam _ _ | Grab _ _ = None
        | App e1 e2 =
            if is_value(e1) then
                let (r, c) = decomp(e2)
                Some (r, extend_l c e1)
            else
                let (r, c) = decomp(e1)
                Some (r, extend_r c e2)
        | Delim e =
            let (r, c) = decomp(e)
            Some (r, extend_d c e)
```

これにより step が同様に定義できる。

## stack を用いる。
```fsharp
type Frame =
    | EvalL (e: Exp) // ([] e) と書く。
    | EvalR (v: {e: Exp | is-value(e)}) // (v []) と書く。
    | Delim // delim と書く。
type State =
    | State (e: Exp, c: List Frame) //  = < e | c > $
```

State の関係は次のようになる。
```
<v | delim::c> ->_("comp") <delim v | c>
<Delim e | c> ->_("decomp") <e | delim::c>
<Grab k e | c> ->_("grab") <Delim plug(F, Grab k e) | c'>
```
ただし、次の条件が満たされている。
- `c = (c_1: List Frame) :: delim :: c_2` で `c_1` には `EvalL` か `EvalR` の Frame のみ含まれる。
- `F` は `c_1` から得られる `PureCxt` とする。
それ以外は定義は同様である。

# 拡張について
rec の定義の参考: [https://www.kurims.kyoto-u.ac.jp/~kenkyubu/kokai-koza/katsumata.pdf]
