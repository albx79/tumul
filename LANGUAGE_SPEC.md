# Tumul Language Specification Draft

**Status:** design baseline / parser-ready draft

This document records the current language-design decisions and proposed syntax discussed so far. Items explicitly marked **TBD** remain open.

## 1. Design principles

Tumul is designed around these principles:

- Immutable values.
- Structural typing.
- Open records and tuples by default.
- Functions have exactly one input and exactly one output.
- Records are the primary mechanism for named arguments and structured results.
- No reserved word keywords.
- Effects are tracked in function types.
- Modules are static, non-parameterized, and non-generative.
- Private fields provide unforgeable structural brands without making types fully nominal.
- Struct literals provide both data construction and local bindings.
- Field order affects scoping and evaluation dependencies, but never type identity.

## 2. Lexical conventions

### 2.1 Comments

Comments begin with `#` and continue to the end of the line.

```text
# This is a comment
```

The token `//` is reserved for effect annotations and effect handling.

### 2.2 Identifier categories

Naming conventions are part of the grammar:

- **Type identifiers:** PascalCase, for example `Int`, `Text`, `UserId`.
- **Value identifiers:** lowercase snake_case, for example `parse_text`, `user_id`.
- **Private fields:** names beginning with `_`, for example `_brand`.
- **Symbols:** a leading apostrophe introduces a symbol literal, for example `'true`.

A symbol is a value whose identity is determined by its name. Symbols are not pattern variables.

```text
'true  # symbol constant
value  # variable pattern
_      # wildcard pattern
```

Symbols are proposed to be globally identified by spelling:

```text
'ready == 'ready
```

## 3. Types

### 3.1 Type expressions

```text
A | B        # union
A & B        # intersection
!A           # negation/complement
A -> B       # pure function
A -> B // E   # effectful function
```

Types denote sets of values.

### 3.2 Bottom type

The empty enumeration is the bottom type:

```text
[]
```

It has no inhabitants, so:

```text
[] <: A
```

for every type `A`.

The standard library may provide:

```text
Never = []
```

### 3.3 Top type

A universal/top type is not yet specified. A possible library or language-defined name is `Any` or `Top`.

## 4. Union, intersection, and negation

### 4.1 Union

```text
A | B
```

is the set-theoretic union of `A` and `B`.

### 4.2 Intersection

```text
A & B
```

is the set-theoretic intersection of `A` and `B`.

### 4.3 Negation

```text
!A
```

is the complement of `A`, relative to the language's value universe.

Difference is expressible as:

```text
A & !B
```

### 4.4 Subtyping

Subtyping is semantic:

```text
A <: B
```

means that every value inhabiting `A` also inhabits `B`.

Conceptually:

```text
A <: B  iff  A & !B = []
```

The initial implementation may use a restricted decidable representation.

## 5. Enumerations

### 5.1 Enumeration syntax

Square brackets define finite enumerations:

```text
[1, 2, 3]
['red, 'green, 'blue]
[1, "two", 3.0]
[]
```

An enumeration is a finite set of explicitly listed values. The empty enumeration is the bottom type.

### 5.2 Enumeration membership

An enumeration is conceptually a union of singleton types:

```text
[1, 2, 3]
```

is equivalent at the type level to:

```text
1 | 2 | 3
```

The bracket form additionally carries finite-enumerability information.

### 5.3 Duplicate members

The proposed rule is that duplicate members are removed from the type's set of inhabitants:

```text
[1, 2, 2, 3]
```

has the same inhabitants as `[1, 2, 3]`.

Enumeration order and the precise behavior of duplicate values remain **TBD**.

### 5.4 Enumeration subtyping

If every member of an enumeration inhabits `T`, then the enumeration is a subtype of `T`:

```text
[1, 2, 3] <: Int
[1, "two"] <: Int | Text
```

### 5.5 Enumeration operations

Finite enumerations should be programmatically enumerable. Possible standard operations include:

```text
values
size
ordinal
from_ordinal
successor
predecessor
```

The exact type-level reflection syntax is **TBD**.

## 6. Range types and numeric types

### 6.1 Ranges

Ranges define finite numeric types:

```text
1..10
-32768..32767
```

A range is conceptually an enumeration of consecutive values.

### 6.2 Library-defined numeric types

Integer types are not primitive language types. The standard library defines them:

```text
Int = -32768..32767
```

The exact bounds are library-defined.

Users can define their own ranges:

```text
Digit = 0..9
Port = 1..65535
Percentage = 0..100
```

### 6.3 Boolean values

Boolean values can be defined as symbols:

```text
Bool = ['false, 'true]
```

The quoted syntax makes constants visibly different from binding patterns.

## 7. Records / structs

### 7.1 Record values

Records use field labels and expressions:

```text
{
  name: "Ada",
  age: 36
}
```

Records are immutable.

### 7.2 Open record types

Records are open by default:

```text
{
  name: Text,
  age: Int
}
```

This means a record with at least those fields and possibly additional fields.

Therefore:

```text
{x: Int, y: Text} <: {x: Int}
```

### 7.3 Closed record types

A trailing dot closes a record type:

```text
{
  name: Text,
  age: Int,
  .
}
```

This means the record has exactly those fields. The dot is part of type syntax, not a runtime field.

### 7.4 Record construction

Record construction uses the same general syntax:

```text
{
  name: "Ada",
  age: 36
}
```

### 7.5 Record updates and spreads

Record spreads use `..`:

```text
{
  ..defaults,
  timeout: 5000,
  retries: 3
}
```

Evaluation and precedence rules:

1. Spread records are evaluated and merged from left to right.
2. Explicit fields are applied afterward.
3. Later definitions replace earlier definitions of the same visible field.

Example:

```text
{
  ..defaults,
  ..environment_config,
  timeout: 5000
}
```

The final `timeout` wins.

## 8. Private fields

### 8.1 Syntax and identity

Fields beginning with `_` are private:

```text
{
  _brand: (),
  text: Text
}
```

A private field is identified conceptually by:

```text
(resolved_module_identity, local_field_name)
```

Two modules may both define `_brand`; they are distinct labels.

### 8.2 Visibility

A private field is visible only in its defining module. Outside that module, code cannot:

- name the field;
- project the field;
- construct the field;
- pattern-match on the field;
- preserve the field through a spread.

Public fields remain accessible:

```text
user_id.text
```

### 8.3 Structural branding

Private fields provide unforgeable structural brands without making the enclosing type fully nominal.

Example:

```text
UserId = {
  _brand: (),
  text: Text
}
```

Code outside the defining module cannot construct a value satisfying the private `_brand` field.

A branded value may still be usable as a public structural supertype, and public fields remain structurally visible.

### 8.4 Private fields and spreads

A spread copies only fields visible in the current module.

If a record contains private fields from another module, those fields are omitted from the spread.

Inside the defining module, its own private fields are visible and are preserved by spreads.

Thus, outside the defining module:

```text
{ ..id, text: "replacement" }
```

does not preserve the private brand of `id`.

## 9. Tuples

### 9.1 Tuple model

Tuples are positional records. Conceptually:

```text
(Int, Text)
```

is equivalent to a record with numeric fields:

```text
{
  0: Int,
  1: Text
}
```

Tuple syntax remains conventional. Explicit numeric-field records may or may not be permitted; this is **TBD**.

### 9.2 Open tuple types

Tuples are open by default:

```text
(Int, Text)
```

means a tuple with at least positions `0: Int` and `1: Text`.

Therefore:

```text
(Int, Text) <: (Int)
(Int, Text, Bool) <: (Int, Text)
```

### 9.3 Closed tuple types

A trailing dot closes a tuple type:

```text
(Int, Text, .)
```

This means exactly two positions.

### 9.4 Empty tuple and empty struct

The empty tuple and empty struct represent the same unit value:

```text
()
{}
```

Both may be accepted as aliases for the unique unit value. Whether one spelling should be banned is **TBD**.

The open empty record/tuple type and closed empty record/tuple type must remain distinct. Exact notation for these is **TBD**.

## 10. Functions

### 10.1 Unary functions

Every function has exactly one input and exactly one output:

```text
A -> B
```

Multiple conceptual parameters are represented by one record:

```text
{
  left: Int,
  right: Int
} -> Int
```

### 10.2 Function values

Anonymous lambdas use `\`:

```text
identity = \x -> x

increment : Int -> Int =
  \x -> x + 1
```

A named function may use direct pattern syntax:

```text
sum { left, right } =
  left + right
```

This is syntactic sugar for a unary lambda accepting one record.

### 10.3 Named arguments

Named arguments are ordinary record construction:

```text
sum {
  left: 1,
  right: 2
}
```

No special multi-argument calling convention is required.

### 10.4 Function subtyping

Function inputs are contravariant and outputs are covariant:

```text
A1 -> B1 <: A2 -> B2
```

when:

```text
A2 <: A1
B1 <: B2
```

For example:

```text
{x: Int} -> Result
```

is a subtype of:

```text
{x: Int, y: Int} -> Result
```

because a function requiring only `x` can accept a record containing `x` and `y`.

## 11. Declarations

The language has no `fn`, `function`, `type`, `let`, `import`, `handle`, `return`, or other reserved word keywords.

### 11.1 Value declarations

```text
name : Type = expression
```

or, when no annotation is given:

```text
name = expression
```

Example:

```text
increment : Int -> Int =
  \x -> x + 1
```

### 11.2 Type declarations

Type names use PascalCase:

```text
Int = -32768..32767
Bool = ['false, 'true]
Port = 1..65535
```

The capitalization convention distinguishes type declarations from value declarations.

## 12. Struct literals as scoped computation

### 12.1 Sequential field scope

Every struct literal introduces sequential field bindings:

```text
{
  a: 41,
  b: a + 1,
  result: string(b)
}
```

Each field expression may reference fields declared earlier in the same struct.

Forward references are invalid:

```text
{
  a: b + 1,
  b: 41
}
```

Self-references are invalid:

```text
{
  a: a + 1
}
```

### 12.2 Structs as local bindings

Structs replace a separate local-variable or block syntax:

```text
compute_total { items } =
  {
    _subtotal: sum items,
    tax: _subtotal * tax_rate,
    total: _subtotal + tax
  }
```

The result is the whole struct.

Private fields can be temporary local values:

```text
{
  _subtotal: sum items,
  _tax: _subtotal * tax_rate,
  total: _subtotal + _tax
}
```

The compiler may remove private fields that cannot be observed outside the construction.

### 12.3 Returning one value with `<<`

A final `<<` expression returns a value instead of the constructed record:

```text
compute_total { items } =
  {
    _subtotal: sum items,
    tax: _subtotal * tax_rate,
    << _subtotal + tax
  }
```

Rules:

- `<<` may appear at most once;
- `<<` must be the final element;
- `<<` is not a field;
- preceding fields are in scope for it;
- without `<<`, the value is the complete struct.

### 12.4 Evaluation dependencies

Field evaluation is constrained by data dependencies rather than necessarily by declaration order.

A field must be evaluated after every earlier field it references. Independent fields may be evaluated in any order.

Example:

```text
{
  a: compute_a(),
  b: compute_b(),
  c: a + 1,
  d: b + 1
}
```

Only these orderings are required:

```text
a before c
b before d
```

A declaration-order implementation is valid but not required.

### 12.5 Effects and independent fields

If independent fields perform effects, their relative ordering is unspecified unless a dependency is introduced.

The effect system tracks may-effects, not a sequence of effects.

## 13. Pattern matching

### 13.1 Match operator

The proposed match operator is:

```text
expression ? {
  pattern -> expression,
  pattern -> expression
}
```

Example:

```text
direction ? {
  'north -> 'west,
  'west  -> 'south,
  'south -> 'east,
  'east  -> 'north
}
```

### 13.2 Boolean conditional sugar

A two-branch Boolean match may use:

```text
condition ? when_true : when_false
```

This is sugar for matching over the Boolean enumeration.

### 13.3 Patterns

Patterns include:

- symbols;
- enumeration members;
- tuple patterns;
- record patterns;
- wildcard `_`;
- binding identifiers.

An unquoted identifier in pattern position is a binding pattern:

```text
value
```

A quoted symbol is a constant pattern:

```text
'true
```

### 13.4 Catch-all patterns

The wildcard pattern matches anything:

```text
_ -> fallback
```

A final identifier pattern may bind the matched value:

```text
value -> value
```

The exact rules for catch-all bindings and exhaustiveness checking are **TBD**.

## 14. Effects

### 14.1 Effectful function types

Effects are written after the result type using `//`:

```text
A -> B // E
```

Examples:

```text
read_file : Path -> Text // IO
parse_text : Text -> Ast // Raise(ParseError)
```

A function with no effects may omit the annotation.

### 14.2 Effect unions

Effect sets use `|`:

```text
A -> B // IO | Raise(Error)
```

This means the computation may perform either effect.

### 14.3 Effects are not values

Effects describe observable interaction or control behavior during evaluation. Examples include:

```text
IO
Raise(Error)
Crash
Random
Clock
Async
```

### 14.4 Effect subtyping

Effect sets are ordered by inclusion:

```text
E1 <: E2
```

when:

```text
E1 ⊆ E2
```

Function subtyping includes effect inclusion:

```text
A1 -> B1 // E1 <: A2 -> B2 // E2
```

when:

```text
A2 <: A1
B1 <: B2
E1 ⊆ E2
```

A pure function is usable where an effectful function is expected:

```text
A -> B <: A -> B // IO
```

### 14.5 Effect composition

If:

```text
f : A -> B // E1
g : B -> C // E2
```

then:

```text
g ∘ f : A -> C // E1 | E2
```

A scoped struct has the union of the effects of its reachable field expressions.

## 15. Effect operations

An effect operation may look like an ordinary function call:

```text
Raise.raise(error)
Random.next_int(range)
IO.read_file(path)
```

The compiler knows from the operation declaration that the call performs an effect. The exact syntax for declaring user-defined effects and operations is **TBD**.

A non-resumable raise operation may have a type conceptually like:

```text
Raise.raise : Error -> Never // Raise(Error)
```

## 16. Effect handlers

### 16.1 Handler syntax

Effect handling uses the same `//` symbol:

```text
expression // {
  operation_pattern -> handler_expression,
  completion_pattern -> handler_expression
}
```

Example:

```text
parse text // {
  Raise.raise(error) -> fallback,
  value -> value,
}
```

### 16.2 Handler clauses

Operation clauses appear first. An optional normal-completion clause appears last.

Rules:

- at most one completion clause;
- the completion clause must be last;
- if absent, normal completion passes through unchanged;
- unlisted effects propagate outward.

Thus:

```text
parse text // {
  Raise.raise(error) -> fallback,
}
```

implicitly passes through a successful result.

### 16.3 Handling `Raise`

`Raise` is abortive and non-resumable. If it is handled, execution does not continue after the original raise point.

### 16.4 Converting errors to ordinary data

A handler can remove an effect and produce an ordinary union value:

```text
parse text // {
  Raise.raise(error) -> error,
  value -> value,
}
```

If:

```text
parse : Text -> Int // Raise(ParseError)
```

then the handled expression has a type equivalent to:

```text
Int | ParseError
```

### 16.5 Resumable effects

Future effects may be resumable. A handler may receive a continuation:

```text
Ask.get_name(prompt, resume) -> resume("Ada")
```

The exact syntax and semantics for resumable handlers are **TBD**.

## 17. Iteration

### 17.1 Library-provided iteration

Iteration over an immutable finite enumeration need not be an effect. It can be implemented by ordinary library functions such as:

```text
for_each
map
fold
```

Conceptually:

```text
for_each : {
  values: Values,
  body: Element -> Unit // E
} -> Unit // E
```

The loop itself contributes no effect; effects from the body propagate.

### 17.2 Optional `for` syntax

A `for` construct, if provided, can be syntax sugar for a library function rather than a keyword:

```text
for color in colors {
  print color
}
```

could elaborate to:

```text
for_each {
  values: colors,
  body: \color -> print color
}
```

Whether such syntax exists is **TBD**.

### 17.3 Iteration effect

A dedicated `Iteration` effect is appropriate only for stateful, asynchronous, generator-based, or nondeterministic producers. Basic enumeration over immutable finite values is pure except for effects in the loop body.

## 18. Modules

### 18.1 Module identity

Modules are determined by file paths. No module declaration is required.

Example:

```text
app/config.lang
```

corresponds to a module path such as:

```text
app/config
```

### 18.2 Reserved module roots

Two module roots are reserved:

```text
std
ext
```

- `std` identifies the standard library.
- `ext` identifies external dependencies.
- Other top-level names belong to the application.

### 18.3 Imports

Imports use `@`:

```text
@std/text
@std/io
@ext/toml/parse
```

Selected names and aliases may be specified:

```text
@std/text { trim, split }
@ext/toml/parse = Toml
```

The exact import grammar remains **TBD**.

### 18.4 Dependency management

Fetching external modules, dependency resolution, registries, lockfiles, versions, and placement are outside the language specification and compiler language semantics.

Source imports do not need to include dependency versions:

```text
@ext/toml/parse
```

The build system resolves this to a concrete dependency instance. The compiler must receive a unique internal identity for each resolved package instance, possibly based on a lockfile identity, content hash, package-store path, or build-graph identifier.

### 18.5 Private modules

A module whose file name begins with `_` is private:

```text
app/_internal.lang
```

The exact visibility boundary is **TBD**. A likely rule is that private modules are importable only within their parent module subtree.

### 18.6 Module properties

Modules are:

- static;
- non-parameterized;
- non-generative;
- resolved before compilation;
- assigned one stable identity per application build.

## 19. Equality

Equality compares complete runtime values, including private fields. It does not depend on the module in which the comparison occurs, the static type view, or which fields are visible at the comparison site.

Values with additional fields are not equal to otherwise matching values with fewer fields:

```text
{ x: 1 } != { x: 1, y: 2 }
```

This remains true even though the latter value can be used where `{ x: Int }` is expected.

If equality under a restricted public view is needed, the program must explicitly construct or project that view.

## 20. Standard library

The language core should remain small. The standard library may define:

- `Int` as a range;
- `Bool` as a symbol enumeration;
- `Text`;
- `Unit`;
- `Path`;
- `List`;
- `Option`;
- `Raise`;
- `IO`;
- other standard effects;
- enumeration operations;
- record and tuple utilities;
- iteration functions.

The exact primitive runtime representations are implementation concerns unless observable through language operations.

## 21. Punctuation vocabulary

| Syntax | Meaning |
|---|---|
| `:` | Type annotation / record field separator |
| `=` | Value or type declaration |
| `->` | Function type / lambda body separator |
| `\\` | Anonymous lambda |
| `//` | Effect annotation / effect handler |
| `|` | Type union / effect union |
| `&` | Type intersection |
| `!` | Type negation |
| `?` | Pattern matching / Boolean conditional |
| `@` | Module reference/import |
| `<<` | Final expression from scoped record construction |
| `..` | Record spread/update |
| `.` | Closed-row marker when trailing in a type |
| `'` | Symbol literal |
| `#` | Comment |
| `{}` | Record value/type/pattern |
| `()` | Tuple or unit value/type |
| `[]` | Finite enumeration / bottom type |

## 22. Combined example

```text
# app/config.lang

@std/text { trim }
@std/io { read_file }
@ext/toml/parse = Toml

Port = 1..65535
Bool = ['false, 'true]

Config = {
  host: Text,
  port: Port,
  .
}

parse_config : Text -> Config // Raise(ParseError) =
  \text ->
    {
      document: Toml.parse(trim text),
      << {
        host: document.host,
        port: document.port
      }
    }

load_config : { path: Path } -> Config // IO | Raise(ParseError) =
  \{ path } ->
    {
      source: read_file path,
      << parse_config source
    }

try_load_config :
  { path: Path } -> Config | ParseError // IO =
  \{ path } ->
    load_config { path } // {
      Raise.raise(error) -> error,
      value -> value,
    }
```

## 23. Parser implementation priorities

### Stage 1: lexical syntax

Implement tokens for:

```text
# comments
@ modules
// effects/handlers
-> arrows/lambdas
<< scoped-struct result
.. spreads
' symbols
. closed rows
| unions
& intersections
! negation
? matches
```

### Stage 2: expressions

Implement:

- identifiers;
- literals;
- symbol literals;
- tuples;
- records;
- record spreads;
- lambdas;
- function application;
- field projection;
- arithmetic and ordinary operators;
- scoped record fields;
- `<<` final expressions.

### Stage 3: types

Implement:

- type identifiers;
- records;
- tuples;
- open and closed rows;
- unions;
- intersections;
- negation;
- function types;
- effect annotations.

### Stage 4: declarations

Implement:

```text
name = expression
name : Type = expression
TypeName = TypeExpression
```

Use identifier casing to distinguish value and type declarations.

### Stage 5: patterns and matching

Implement:

- wildcard;
- bindings;
- symbol constants;
- tuple patterns;
- record patterns;
- catch-all branches;
- `?`.

### Stage 6: effects

Implement:

- effect annotations;
- effect sets;
- operation calls;
- handler expressions;
- final normal-completion branches.

### Stage 7: modules

Implement:

- file-derived module paths;
- `@` imports;
- aliases;
- private module visibility;
- private-field identity.

## 24. Intentionally unresolved questions

1. Exact import grammar after `@`.
2. Whether `{}` and `()` are both accepted as unit values.
3. Exact syntax for open and closed empty record/tuple types.
4. Closed-row semantics in the presence of inaccessible private fields.
5. Whether type declarations use `=` or a separate punctuation form.
6. Type-level reflection syntax for enumerations.
7. Enumeration order and duplicate-member behavior.
8. Exact primitive runtime types such as `Text`.
9. Exact pattern grammar and binding behavior.
10. Whether a final bare identifier pattern is always a catch-all binding.
11. Whether general effect declarations are available in version one.
12. Syntax for resumable handlers and continuation binding.
13. Exact syntax for generic `Raise` operations.
14. Private-module visibility boundaries.
15. Whether explicit numeric-field records are allowed.
16. Whether `for` syntax exists or is entirely library-based.
17. Whether top-level declarations may be mutually recursive.
18. Whether public values can explicitly hide or project fields.
19. Exact static semantics of finite enumeration reflection.
20. Exact type universe relative to which `!A` is interpreted.
