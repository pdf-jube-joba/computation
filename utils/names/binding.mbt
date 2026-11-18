///|
pub(all) enum View[T] {
  Variable(String)
  Node(Array[String], Array[(Array[Int32], T)], (Array[String], Array[T]) -> T)
}

///|
pub(all) struct Syntax[T] {
  view : (T) -> View[T]
  variable : (String) -> T
}

///|
priv struct Symbol {
  id : Int32
  name : String
}

///|
priv struct Reader {
  next : Ref[Int32]
  free : Map[String, Symbol]
}

///|
priv enum Resolved[T] {
  Var(Symbol)
  Tree(
    Array[Symbol],
    Array[(Array[Int32], Resolved[T])],
    (Array[String], Array[T]) -> T,
  )
}

///|
fn reader() -> Reader {
  { next: { val: 0, }, free: Map([]), }
}

///|
fn fresh(reader : Reader, name : String) -> Symbol {
  let id = reader.next.val
  reader.next.val += 1
  { id, name, }
}

///|
fn variable(
  reader : Reader,
  env : Array[(String, Symbol)],
  name : String,
) -> Symbol {
  for pair in env {
    if pair.0 == name {
      return pair.1
    }
  }
  match reader.free.get(name) {
    Some(v) => v
    None => {
      let v = fresh(reader, name)
      reader.free[name] = v
      v
    }
  }
}

///|
fn[T] resolve(
  syntax : Syntax[T],
  reader : Reader,
  env : Array[(String, Symbol)],
  term : T,
) -> Resolved[T] {
  match (syntax.view)(term) {
    Variable(name) => Var(variable(reader, env, name))
    Node(names, children, rebuild) => {
      let binders = names.map(name => fresh(reader, name))
      let children = children.map(child => {
        let mut env = env
        for i in child.0 {
          let v = binders[i]
          let next = [(v.name, v)]
          next.append(env)
          env = next
        }
        (child.0, resolve(syntax, reader, env, child.1))
      })
      Tree(binders, children, rebuild)
    }
  }
}

///|
fn[T] free_names(
  term : Resolved[T],
  bound : Array[Int32],
  assign : (Symbol) -> String,
) -> Unit {
  match term {
    Var(v) => if !bound.contains(v.id) { ignore(assign(v)) }
    Tree(binders, children, _) =>
      for child in children {
        let bound = bound.copy()
        for i in child.0 {
          bound.push(binders[i].id)
        }
        free_names(child.1, bound, assign)
      }
  }
}

///|
fn[T] emit_visit(
  syntax : Syntax[T],
  term : Resolved[T],
  assign : (Symbol) -> String,
) -> T {
  match term {
    Var(v) => (syntax.variable)(assign(v))
    Tree(binders, children, rebuild) => {
      let names = binders.map(assign)
      let children = children.map(c => emit_visit(syntax, c.1, assign))
      rebuild(names, children)
    }
  }
}

///|
fn[T] emit(syntax : Syntax[T], term : Resolved[T]) -> T {
  let names : Map[Int32, String] = Map([])
  let supply = supply()
  fn assign(v : Symbol) -> String {
    match names.get(v.id) {
      Some(n) => n
      None => {
        let n = supply.fresh(v.name)
        names[v.id] = n
        n
      }
    }
  }
  free_names(term, [], assign)
  emit_visit(syntax, term, assign)
}

///|
pub fn[T] normalize(syntax : Syntax[T], term : T) -> T {
  emit(syntax, resolve(syntax, reader(), [], term))
}

///|
fn[T] replace(
  term : Resolved[T],
  target : Int32,
  replacement : Resolved[T],
  changed : Ref[Bool],
) -> Resolved[T] {
  match term {
    Var(v) if v.id == target => {
      changed.val = true
      replacement
    }
    Var(_) => term
    Tree(binders, children, rebuild) =>
      Tree(
        binders,
        children.map(child => {
          (child.0, replace(child.1, target, replacement, changed))
        }),
        rebuild,
      )
  }
}

///|
pub fn[T] substitute(
  syntax : Syntax[T],
  source : T,
  target : String,
  replacement : T,
) -> T {
  let reader = reader()
  let term = resolve(syntax, reader, [], source)
  let target = variable(reader, [], target)
  let replacement = resolve(syntax, reader, [], replacement)
  let changed : Ref[Bool] = { val: false, }
  let result = replace(term, target.id, replacement, changed)
  if changed.val {
    emit(syntax, result)
  } else {
    source
  }
}
