# Tumul

Tumul is a design for a small, statically typed, functional language built on **structural, set-theoretic types**. It is a specification draft, not an implementation. The full draft is in [LANGUAGE_SPEC.md](LANGUAGE_SPEC.md).

## Ideas

- **Types are sets.** Union `|`, intersection `&`, negation `!` and XOR `^` work on types, and subtyping is set inclusion. There is no global top type, on purpose.
- **Structural, with unforgeable brands.** Records `{ x: Int }` and tuples `(Int, Text)` are open by default and closed with a trailing `.`. Private fields (`_brand`) give branding without nominal types. `.*` erases a value to its statically known shape, and `==` requires such closed operands.
- **Immutable values, one input and one output per function.** Several arguments are one record. Functions are applied by juxtaposition: `sin 1`.
- **Recursive types.** `List(A) = (.) | (A, List(A), .)`. Recursion must be guarded by a closed tuple or an arrow, which keeps subtyping decidable (regular tree types, checked by tree-automaton containment).
- **Unbounded numbers and text.** `Number` and `Text` are library types over recursive lists, so ranges like `0..8989898989898989898989` are legal. Any machine representation is the compiler's business.
- **Literals are elaborated, not typed.** `42` and `"foo"` are compile-time entities that become values at the type they are used at, through a pure companion function on that type (`Text = … :: { from_literal_str = … }`). It fails with `Raise(Text)`, so a bad literal is a compile error.
- **Effects in types.** `Text -> Config // IO | Raise(ParseError)`. There is a fixed set of built-in effects. Handlers are deep and continuations are multi-shot.
- **Pattern matching as the one conditional.** `x ? { pattern -> expr, … }` with set-theoretic patterns, narrowing by earlier arms, and exhaustiveness checked as subtyping. The Boolean form is `c ?? a :: b`.
- **Small, uniform syntax.** No reserved keywords. `=` binds and `:` gives a type, at every level. A struct body, a block and a module file are all the same thing: comma-separated declarations. Types can be declared anywhere.
- **Modules.** A file is a module and a same-named directory holds its submodules. Private declarations are visible to descendants. Imports are declarations: `@std/text { trim }`, `@ext/toml/parse = Toml`.

## A taste

```text
@std/text { trim },
@std/io { read_file },
@ext/toml/parse = Toml,

Port = 1..65535,

Config = { host: Text, port: Port, . },

parse_config : Text -> Config // Raise(ParseError) =
  \text -> {
    document = Toml.parse(trim text),
    << { host = document.host, port = document.port }
  },

load_config : { path: Path } -> Config // IO | Raise(ParseError) =
  \{ path } -> {
    source = read_file path,
    << parse_config source
  },

describe : (Int | Text | 'none) -> Text =
  \x -> x ? {
    'none  -> "nothing",
    0      -> "zero",
    n: Int -> string(n),
    t      -> t
  },
```

## Status

The core type system, records and tuples, patterns, effects, modules and literals are specified. Open questions are collected in §24 of the spec. The main ones are polymorphic function signatures, the exact `Number` digit encoding, string escapes, the built-in effect set, and how `f(x).y` parses.

## References

- Frisch, Castagna, Benzaken. *Semantic Subtyping.* TOPLAS 2008 (the model behind the union, intersection and negation types), and the CDuce language.
- Amadio, Cardelli. *Subtyping Recursive Types.* POPL 1993.
- Castagna et al. *Polymorphic Functions with Set-Theoretic Types,* Parts 1 and 2. POPL 2014 and 2015.
