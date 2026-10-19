[[Model]]

多くの言語・計算モデルの最終コンパイル先。
[cfg_vreg](../../docs/books/low_layer/cfg_vreg.md) を基本に、複数の独立した無限メモリを持つ。
[flow_ir](../flow_ir/README.md) の stack/heap は、その上の操作として実現する。

- Harvard 型、コードは不変。`tiny_isa` への lowering は前提にしない。
- 仮想レジスタは全ブロックで共有、個数制限なし、非 SSA。各プログラムで使う名前は有限個。
- ブロックは1段階、引数なし。terminator は条件付きジャンプ列と goto/halt。

## 値と状態

\[
\begin{aligned}
k &\in \mathbb N_{>0} \\
\mathcal M &:= \{\mathrm{MEM}_0,\ldots,\mathrm{MEM}_{k-1}\} \\
S &::= \mathrm{Whole}(m) \mid \mathrm{Sub}(s,b,e) \\
A &::= \mathrm{At}(s,i) \\
V &:= \mathbb N + \mathrm{CodeLabel} + A + S \\
R &:\mathrm{VReg}\to V \\
\mu &:\mathcal M\to(\mathbb N\to V) \\
\sigma &:= (\ell,j,R,\mu)
\end{aligned}
\]

- \(m\in\mathcal M,\ s\in S,\ b,e,i\in\mathbb N\)。構成子の有効条件は後述。
- \(\ell\)：現在のブロック、\(j\)：文位置。入口は `:main`。
- \(R_0(r)=0\)。レジスタ自体のアドレスは取れない。
- \(\mu(m)\) は初めから無限長。伸縮なし、入力で指定しないセルは0。
- 自然数の演算は加算と飽和減算。コードラベルの算術・数値変換はなし。

有限入力・有限ステップでは \(\operatorname{supp}(\mu)\) は有限なので、実装では疎な表などで保持できる。

## slice / addr

### origin, base, len

| \(s\) | \(o(s)\)：origin | \(b(s)\)：base | \(L(s)\)：len |
| --- | --- | --- | --- |
| \(\mathrm{Whole}(m)\) | \(m\) | \(0\) | \(\infty\) |
| \(\mathrm{Sub}(t,u,v)\) | \(o(t)\) | \(b(t)+u\) | \(v-u\) |

origin は根の `MEM_i`。独立に作っても根が同じなら同じ origin、根が違えば異なる。
境界は各 slice に属し、origin の一致だけでは広がらない。

\[
\begin{aligned}
\operatorname{valid}(\mathrm{Whole}(m))
  &\iff m\in\mathcal M \\
\operatorname{valid}(\mathrm{Sub}(s,u,v))
  &\iff \operatorname{valid}(s)\land 0\le u\le v\le L(s) \\
\operatorname{valid}(\mathrm{At}(s,i))
  &\iff \operatorname{valid}(s)\land 0\le i\le L(s) \\
\operatorname{readable}(\mathrm{At}(s,i))
  &\iff \operatorname{valid}(\mathrm{At}(s,i))\land i<L(s)
\end{aligned}
\]

- `Sub(s,u,v)`：親に対する半開区間 \([u,v)\)。端点は有限、空 slice も可。
- slice は不変。狭める場合は子を作る。親・根を取り出す操作はなし。
- 別に保持した親・根からの切り出しは可。既存の slice/addr の境界は変わらない。
- \(\infty\notin V\)。長さ取得は有限 slice にのみ定義し、`Whole` では `Err`。
- 有限 slice の終端 addr は作れるが load/store 不可。`Whole` に終端 addr はない。

### アドレス演算

\(a=\mathrm{At}(s,i)\) に対し、

\[
\begin{aligned}
o(a)&:=o(s) \\
p(a)&:=b(s)+i \\
\operatorname{advance}(a,n)&:=\mathrm{At}(s,i+n)
  &&(n\in\mathbb N,\ i+n\le L(s)) \\
\operatorname{retreat}(a,n)&:=\mathrm{At}(s,i-n)
  &&(n\in\mathbb N,\ n\le i) \\
\operatorname{diff}(a,c)&:=\max(p(a)-p(c),0)
  &&(o(a)=o(c)) \\
\operatorname{lt}(a,c)&:=[p(a)<p(c)]
  &&(o(a)=o(c)) \\
\operatorname{sameLoc}(a,c)&:=[o(a)=o(c)\land p(a)=p(c)]
\end{aligned}
\]

- `advance` / `retreat` は元の slice を保持。範囲外への移動は `Err`。
- `diff` / 順序比較は同じ origin に限る。異なる子 slice 間でも可。
- `diff` は自然数の飽和減算。方向は boolean の順序比較で判定する。符号付きの値は不要。
- `advance` / `retreat` / `diff` は意味論上の関数名。命令構文では型に応じた `+` / `-` として扱う。
- addr + addr、自然数から境界なし addr への変換はなし。
- `sameLoc` は位置の等値。境界を含む値全体の等値とは別で、真でも相互に置換できるとは限らない。

```text
s = Sub(Whole(MEM_0), 10, 20)
t = Sub(s, 2, 5)
p = At(t, 1)                     // MEM_0[13], bounds [12,15)
q = At(s, 6)                     // MEM_0[16], bounds [10,20)

diff(q, p) = 3
diff(p, q) = 0
lt(p, q) = true
advance(p, 3) = Err
advance(p, 2) = At(t, 3)         // end: valid, not readable
```

### load/store

\(\operatorname{readable}(a)\) を前提として、

\[
\begin{aligned}
\operatorname{load}(\mu,a)&:=\mu(o(a))(p(a)) \\
\operatorname{store}(\mu,a,v)&:=
  \mu[o(a)\mapsto\mu(o(a))[p(a)\mapsto v]]
\end{aligned}
\]

全操作について、型・有効条件を満たさなければ `Err`。
その操作の副作用は適用せず停止する。未定義動作は設けない。

slice の重なりは許す。境界検査と、所有権・生存状態の管理は別。
メモリは常に存在し、stack の有効範囲や heap の解放状態は lowering 側で検査する。

## 構文

プログラムの先頭で `\memories k;` を宣言する。`k ≥ 1`、メモリ番号は `0 .. k-1`。

### value / place / stmt / cond

\[
\begin{aligned}
\NT{vreg} &\defeq \T{\%}\NT{string} \\
\NT{code-label} &\defeq \T{:}\NT{string} \\
\NT{memory} &\defeq \T{MEM\_}\NT{number} \\
\NT{value} &\defeq \NT{vreg}\mid\NT{imm}\mid\NT{code-label}\mid\NT{memory} \\
\NT{place} &\defeq \T{\textbackslash deref}\ \NT{value} \\
\NT{binop} &\defeq \T{+}\mid\T{-} \\
\NT{rel} &\defeq \T{==}\mid\T{<}\mid\T{>} \\
\NT{cond} &\defeq \NT{value}\ \NT{rel}\ \NT{value} \\
\NT{rhs} &\defeq \NT{value} \\
  &\mid \NT{value}\ \NT{binop}\ \NT{value} \\
  &\mid \T{\textbackslash ld}\ \NT{place} \\
  &\mid \T{\textbackslash slice}\ \NT{value}\ \NT{value}\ \NT{value} \\
  &\mid \T{\textbackslash addr}\ \NT{value}\ \NT{value} \\
  &\mid \T{\textbackslash len}\ \NT{value} \\
\NT{stmt} &\defeq \NT{vreg}\ \T{:=}\ \NT{rhs}\T{;} \\
  &\mid \NT{place}\ \T{:=}\ \T{\textbackslash st}\ \NT{value}\T{;}
\end{aligned}
\]

- value はメモリを読まない atom。rhs の演算は入れ子にせず、結果をレジスタへ置く。
- addr は値、place はその参照先。`addr` が参照の作成、`deref` が場所の指定に対応する。
- cond は boolean を返すが、boolean を保持する値型は追加しない。

### 評価

\[
\begin{aligned}
\llbracket r\rrbracket_R&:=R(r) \\
\llbracket n\rrbracket_R&:=n \\
\llbracket\ell\rrbracket_R&:=\ell \\
\llbracket\mathrm{MEM}_i\rrbracket_R&:=\mathrm{Whole}(\mathrm{MEM}_i) \\
\operatorname{place}_R(\T{\textbackslash deref}\ v)&:=\llbracket v\rrbracket_R\in A
\end{aligned}
\]

ラベルはプログラム中に存在するもの、メモリ番号は宣言した \(k\) 未満に限る。
以下の引数は value 評価後の値。表の型・条件を満たさない場合は `Err`。

| rhs | 引数 | 結果 |
| --- | --- | --- |
| `v` | \(v\in V\) | \(v\) |
| `x + y` | \(x,y\in\mathbb N\) | \(x+y\) |
| `x - y` | \(x,y\in\mathbb N\) | \(\max(x-y,0)\) |
| `a + n` | \(a\in A,\ n\in\mathbb N\) | \(\operatorname{advance}(a,n)\) |
| `a - n` | \(a\in A,\ n\in\mathbb N\) | \(\operatorname{retreat}(a,n)\) |
| `a - c` | \(a,c\in A\), same origin | \(\operatorname{diff}(a,c)\) |
| `\ld p` | \(a=\operatorname{place}_R(p)\), readable | \(\operatorname{load}(\mu,a)\) |
| `\slice s u v` | \(s\in S,\ u,v\in\mathbb N\), valid | \(\mathrm{Sub}(s,u,v)\) |
| `\addr s i` | \(s\in S,\ i\in\mathbb N\), valid | \(\mathrm{At}(s,i)\) |
| `\len s` | \(s\in S,\ L(s)<\infty\) | \(L(s)\) |

binop の上記以外の型の組み合わせは `Err`（自然数 + addr、addr + addr など）。
addr の移動は境界を検査し、addr 同士の差は origin を検査する。

\[
\begin{aligned}
(r\T{:=}e, R,\mu)&\longmapsto(R[r\mapsto\operatorname{eval}_{R,\mu}(e)],\mu) \\
(p\T{:=}\T{\textbackslash st}\ v,R,\mu)&\longmapsto
  (R,\operatorname{store}(\mu,\operatorname{place}_R(p),\llbracket v\rrbracket_R))
\end{aligned}
\]

成功時は文位置 \(j\) を1進める。store の readable 条件も同じ。右辺はすべて更新前の状態で評価する。

| cond の引数 | `==` | `<` / `>` |
| --- | --- | --- |
| 自然数同士 | 数値の等値 | 数値の順序 |
| コードラベル同士 | ラベルの一致 | `Err` |
| addr 同士 | `sameLoc`（異なる origin なら偽） | same origin のとき根からの位置を比較、それ以外 `Err` |
| slice 同士 | 構成子の構造的等値 | `Err` |
| 異なる種類の値 | `Err` | `Err` |

slice の構造的等値は作成時刻によらない。addr の `==` は境界を比較しない。

```text
\memories 1;
:main {
  %s := \slice MEM_0 10 20;
  %p := \addr %s 0;
  \deref %p := \st 42;
  %x := \ld \deref %p;
  %q := %p + 1;
  %d := %q - %p;
  \if %p < %q \then :next;
  \halt;
}
:next {
  \halt;
}
```

### ブロックと制御

\[
\begin{aligned}
\NT{target} &\defeq \NT{code-label}\mid\NT{vreg} \\
\NT{jump-if} &\defeq
  \T{\textbackslash if}\ \NT{cond}\ \T{\textbackslash then}\ \NT{target}\T{;} \\
\NT{cont} &\defeq \NT{jump-if}^*
  (\T{\textbackslash goto}\ \NT{target}\T{;}\mid\T{\textbackslash halt}\T{;}) \\
\NT{block} &\defeq \NT{code-label}\T{\{}\ \NT{stmt}^*\ \NT{cont}\ \T{\}}
\end{aligned}
\]

- 条件は上から評価し、最初に成立した target へ。不成立なら最後の goto/halt。
- target の評価結果はコードラベル。goto は \((\ell,j,R,\mu)\mapsto(\ell',0,R,\mu)\)。
- region、ブロック引数、call/return、相対ジャンプ、ブロック途中へのジャンプはなし。
- call/return は戻り先と必要な値の保存・復元、および間接ジャンプで構成する。

自己書き換えなし。自己書き換えのあるモデルは、コードをデータとして保持する実行器へコンパイルする。

## 入出力

| interface | 意味 |
| --- | --- |
| `AInput` | `index := value;` の列。メモリ0の初期指定、指定外・他メモリは0 |
| `FOutput` | 同形式。メモリ0の非0セルのみ、添字の昇順 |
| `RInput` / `ROutput` | 現実装は unit（空文字列）。`flow_ir` の input/print 対応は今後追加 |

入出力の値は自然数、`:label`、`MEM_i`、`\slice s b e`、`\addr s i`。slice/addr は再帰的に記述できる。
重複添字・未宣言ラベル/メモリ・不正な境界は `Err`。空入力は全セル0。

```text
0 := 42;
10 := \addr \slice MEM_0 20 30 3;
```

実行は1文または1条件・terminatorずつ。snapshot はコード・文位置・レジスタ・疎なメモリを保持し、復元時にも有効性を検査する。

<div data-model="core_machine"></div>

static 初期値は初期化コードで配置する案。入力との配置規約はコンパイラ側で定める。

## flow_ir からのコンパイル

| flow_ir | core_machine |
| --- | --- |
| static | `MEM_0` 内の固定位置 |
| stack | stack 用メモリ、要素数 `sp` |
| stack 参照 | 添字と、アクセス時の \(i<sp\) 検査 |
| heap 領域 | heap 用メモリ内の区間 |
| heap handle / 参照 | 先頭位置・長さ・生存状態の管理表、アクセス時の検査 |
| region 内の block | 名前を区別して平坦化した block |
| enter | 対象 region のレジスタを初期化する入口へのジャンプ |
| code label | 対応する block のラベル値 |

stack 用メモリを \(m_s\)、\(a_i:=\mathrm{At}(\mathrm{Whole}(m_s),i)\) とすれば、

\[
\begin{aligned}
\operatorname{push}(v)&:\quad\mu\leftarrow\operatorname{store}(\mu,a_{sp},v);\quad sp\leftarrow sp+1 \\
\operatorname{pop}()&:\quad sp>0;\quad v\leftarrow\operatorname{load}(\mu,a_{sp-1});\quad sp\leftarrow sp-1
\end{aligned}
\]

これは使用範囲の管理であり、メモリ・slice の伸縮ではない。
heap も handle 発行・確保サイズ・解放状態を管理する。保存済み参照にも検査が必要。

> [!Note]
> lowering の正しさで `Err` も保存するかは別途明示する。
> 保存する場合、生成コードから明示的に失敗させる操作も決める。
