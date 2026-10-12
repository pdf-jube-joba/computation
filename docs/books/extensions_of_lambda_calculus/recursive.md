定義に自分自身を含むものを再帰的定義という。


## 相互再帰
これは再帰が一つで終わらないやつ
```
let-rec is-even n := if n == 0 then true else !is-odd(n-1);
rec-with is-odd n := if n == 0 then false else !is-even(n-1);
```
とか
```
struct A {
  n: usize,
  b: B,
}
enum B {
  None,
  A(A),
}
```
