# 理論側の話
- ラベルとして入力・出力・論理ゲートを考えて、頂点にラベルが乗っているグラフが論理回路
- 各頂点に $b \in \{0, 1\}$ を割り当てたのが論理回路の状態
- 計算可能な論理回路は
    - 有限なグラフ
    - 計算可能な論理回路2つ（異なってよい）を入力と出力をつなげる
    - ある一つの計算可能な論理回路を無限に並べて、入力と出力をつなげる

# 現状の把握
```rust
enum Gate {
    Not {
        state: Bool, input: Bool,
    }
    /* ... */
}

struct FinGraph {
    lcs: Vec<(Name, LogicCircuit)>,
    edges: Vec<((Name, OtPin), (Name, InPin))>,
    input: Vec<(InPin, (Name, InPin))>,
    otput: Vec<(OtPin, (Name, OtPin))>,
}

struct Iter {
    /* ... */
}

enum LogicCircuits {
    Gate(Gate),
    Fingraph(Fingraph),
    Iter(Iter),
}

impl LogicCircuits {
    fn get_input(&self, inpin: &InPin) -> Option<&Bool>;
    fn get_otput(&self, otpin: &OtPin) -> Option<&Bool>;
    fn get_state_of_gate_from_path(&self, path: &Path) -> Option<&Bool>;
}
```
## 問題点
- 理論側とのずれ
    - Gate をグラフのラベル用じゃなくてグラフとして用いている
    - どちらかというと、イメージとしては `fn next(&mut self, input: HashMap<InPin, Bool>)` の方が近い
    - 名前 `Name` があることは本質的？
        - 実用上は仕方ない気がする
    - 理論をもっとと単純にしたい
- コードが長い。
## 考えること
- [] web で表示する側でほしいメソッドは？？

# 改善案
```rust
enum Gate {
    Not, /* ... */
}

// ここに共通するメソッドをまとめて読みやすくする
trait LogicCircuit {

}

struct OneGate {
    gate: Gate,
    state: Bool,
}

// FinGraph じゃなくてより正確な名前を
// LogicCircuit の構成の仕方のグラフから論理回路をつくる
struct CompositionCircuitFromGraph {

}

// 
struct IterationCircuit {

}

```
