# Tumul Language Specification Draft

**Status:** design baseline / parser-ready draft (revision 2)

This document records the current language-design decisions and proposed syntax discussed so far. Items explicitly marked **TBD** remain open.

## 1. Design principles

Tumul is designed around these principles:

- Immutable values.
- Structural typing.
- Open records and tuples by default, with literals as a deliberate exception (§7.4).
- Functions have exactly one input and exactly one output.
- Records are the primary mechanism for named arguments and structured results.
- No reserved word keywords.
- Effects are tracked in function types.
- Modules are static, non-parameterized, and non-generative.
- Private fields provide unforgeable structural brands without making types fully nominal.
- Struct literals provide both data construction and local bindings.
- Field order affects scoping and evaluation dependencies, but never type identity.
- Records and tuples are distinct type formers with parallel, analogous rules, not a single unified kind (§9.1).
- Recursive type declarations are permitted under a guardedness condition (§3a).

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
A ^ B        # symmetric difference ("XOR type"; sugar, see §4.5)
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

The language has no universal/top type. `!A` is meaningful only relative to a bounding domain (§4.4); there is deliberately no type inhabited by every value in the language, and no single type that accepts any argument (see §24 for the discussion of why this was not pursued).

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

is the complement of `A`. Because the language has no top type (§3.3), `!A` has no meaning on its own relative to a global universe. `!A` is syntactically valid anywhere a type may appear, but it only *resolves* to a concrete type where the surrounding expression supplies a bounding domain — most commonly inside an intersection, where `A & !B` is well-defined as "values in `A`'s domain that are not in `B`," without reference to anything outside that domain.

Standalone use of `!A` — for example in a top-level function signature such as `f : !A -> B` — is syntactically legal but is a type error: there is no domain for the checker to complement `A` against. This is deferred to type-checking, not rejected at parse time.

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

This law is stated relative to whatever domain bounds the types involved (§4.3); it does not presuppose a global universe. The initial implementation may use a restricted decidable representation (see §3a.4 for the intended general approach once recursive types are involved).

### 4.5 Symmetric difference (XOR types)

```text
A ^ B
```

denotes values guaranteed to belong to exactly one of `A` or `B`, never both and never neither. It is defined as sugar:

```text
A ^ B = (A | B) & !(A & B)
```

Every negation introduced by this expansion is immediately intersected with `A | B` or a subset of it, so `^` never requires a domain wider than its own two operands — it does not depend on a global top type, and is well-defined even though standalone `!` outside an intersection is not.

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

A range is conceptually an enumeration of consecutive values, grounded in the digit-list representation of `Number` (§3a.5): a range's bound may be arbitrarily large, since `Number` itself is unbounded. Whether a given range is small enough to be represented efficiently by a target machine (e.g. packed into a machine word, or even a single bit for a two-value range) is a compiler concern, not a language restriction — the language places no upper bound on a range's bounds.

### 6.2 Library-defined numeric types

Integer types are not primitive language types. The standard library defines them in terms of `Number` (§3a.5):

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

A *declared* record type is open by default:

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

Struct *literal expressions* are closed by construction, independent of this default — see §7.4.

### 7.3 Closed record types

A trailing dot closes a record type:

```text
{
  name: Text,
  age: Int,
  .
}
```

This means the record has no additional fields *nameable at this point in the source*. In the absence of private fields this is a full guarantee that no other fields exist at all. In the presence of private fields, the guarantee is weaker — see §7.3a.

### 7.3a Three row states

Because private fields (§8) can be invisibly present on an otherwise-closed record, "closed" is not a single guarantee. There are three distinguishable row states:

- **open** — arbitrary additional public fields may exist.
- **closed-public** — no additional *public* fields exist, but private fields (from the defining module, or from a module that produced the value via spread) may still be present, invisibly, to code outside their defining module.
- **closed-total** — no additional fields exist at all, of any kind. This is a strictly stronger guarantee than closed-public, and holds only when a value is provably free of private fields — for instance, when built directly from a literal with no spread from a branded source (§7.4), or when observed from inside the module that defines all of its private fields.

The `.` marker in type position denotes closed-public in general, degenerating to closed-total exactly when no private fields can be present. The distinction matters most for equality (§19) and the close operator (§8.4a).

Tuples have no analogue of this three-way split; see §9.1a.

### 7.4 Record construction and literal closedness

Record construction uses the same general syntax:

```text
{
  name: "Ada",
  age: 36
}
```

A struct literal expression is closed by construction, not merely closed by default: since a literal can only ever denote the fields written in it (or, transitively, spread into it), nothing beyond what is present in the source could possibly be part of its value. This holds regardless of control flow — a struct literal appearing inside either arm of a `?`-expression is still closed in each arm; the type of the overall expression is then governed by §13.1's rule for combining arm types (a union of the two closed record types, not a single record with optional fields).

The one qualification: a literal that spreads an operand which is itself open inherits that operand's openness, since the spread may carry unknown additional fields (§7.5). A literal with no spreads, or spreading only closed operands, is closed — closed-total if none of the spread operands carry private fields observable at this point in the source, closed-public otherwise.

### 7.5 Record updates and spreads

Record spreads use `..`:

```text
{
  ..defaults,
  timeout: 5000,
  retries: 3
}
```

Record spread is a **merge keyed by field name**: evaluation and precedence rules:

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

This merge-by-name semantics is specific to records. Tuple spread is a different operation (concatenation-by-position) and the two are not interchangeable — a tuple cannot be spread into a record literal or vice versa (§9.1a, §9.5).

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

A branded value may still be usable as a public structural supertype, and public fields remain structurally visible. From outside the defining module, `UserId`'s effective visible type is closed-public, not closed-total (§7.3a): the `_brand` field is real but unnameable, so external code must not assume the record has exactly one field.

### 8.4 Private fields and spreads

A spread copies only fields visible in the current module.

If a record contains private fields from another module, those fields are omitted from the spread.

Inside the defining module, its own private fields are visible and are preserved by spreads.

Thus, outside the defining module:

```text
{ ..id, text: "replacement" }
```

does not preserve the private brand of `id`.

### 8.4a The close operator, `.*`

`.*` produces a closed-total value from any record or tuple, by keeping only the fields or positions that are **statically visible in the declared type at the point of use**, and discarding everything else. It performs no runtime inspection of the value: the result is determined entirely by the static type, exactly as with the closed-unless-last rule for tuple spreads (§9.5).

```text
user : UserId
user = create_user("foo")

is_foo = { text: "foo" } == user.*
```

Outside `Auth` (the module defining `UserId`), `_brand` is not statically visible, so `user.*` erases it, producing a closed-total `{text: Text, .}` comparable against the literal. Inside `Auth`, the same operation on the same value preserves `_brand`, since it is visible there — `.*` gives different results at different points in the source not because it inspects the value differently, but because it always follows what is nameable at that point, and privacy already restricts that per §8.2. No special-casing of privacy is required in `.*`'s definition itself.

`.*` is the general "close to statically-known shape" operator for both records (keyed by name) and tuples (keyed by position); it is what allows a non-final tuple spread's length to be pinned down explicitly (§9.5) and what allows `==` to accept operands that are not already closed-total (§19).

### 8.5 Equality and privacy

Because `.*` already produces the correct result under module-relative visibility, equality (§19) requires no privacy-specific carve-out of its own — see §19.

## 9. Tuples

### 9.1 Tuple model

Tuples are semantically records whose fields are sequentially numbered rather than named: a tuple is guaranteed to be equivalent, at the value level, to a record with fields `0, 1, 2, …` in order. However, tuples and records are distinct type formers with independent, if structurally parallel, rules — not one unified kind reached through two spellings. There is no syntax to construct a record with numeral field names directly; `(...)` is the only construction syntax that produces numbered fields, and the language guarantees the resulting numbering is sequential and contiguous from `0`. `{...}` literals must have field names that are valid identifiers.

Explicit numeric-field records constructed any other way are not permitted.

### 9.1a Why tuples and records are kept separate

Two considerations rule out full unification of construction and operations, even though the underlying value shape coincides:

- **Spread semantics conflict.** Record spread (§7.5) is a merge keyed by name, where a later write overrides an earlier one at the same key. Tuple spread is concatenation by position (§9.5): `(..a, ..b)` places `b`'s elements after `a`'s, with nothing "overriding" anything. Forcing these through one operation would mean, under merge semantics, that `b`'s first element *replaces* `a`'s first element rather than following it — discarding data that concatenation is supposed to preserve. The same token `..` therefore has two distinct operational meanings depending on which kind it spreads into; a tuple cannot be spread into a record literal or vice versa.
- **Pattern ergonomics.** Matching a tuple as `(a, b)` is the expected form; requiring `{0: a, 1: b}` would be needlessly verbose for something with no named structure. Tuple patterns and record patterns remain separate productions in §13.3.

There is also no privacy analogue for tuples (§7.3a): nothing can inject an unnameable positional element into a tuple the way a private field can ride along on a record. A closed tuple is therefore always closed-total; the open / closed-public / closed-total distinction is a records-only concern.

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

As with records (§7.4), a tuple *literal* is closed by construction rather than open by default — the same qualification applies: a literal spreading an open operand inherits that operand's openness.

### 9.3 Closed tuple types

A trailing dot closes a tuple type:

```text
(Int, Text, .)
```

This means exactly two positions, and — since tuples have no private-field analogue — this is always a closed-total guarantee, unlike the corresponding record marker (§7.3a).

### 9.4 Empty tuple and empty record

Because records and tuples are open by default (§7.2, §9.2), the empty forms follow directly from ordinary width subtyping rather than needing a separate stipulation:

- `{}` and `()` denote the *open* empty row — a type with no required fields/positions, i.e. the top of the record subtyping order and the top of the tuple subtyping order respectively. Any record inhabits `{}`; any tuple inhabits `()`. These are not the unit type.
- `{.}` and `(.)` denote the *closed* empty row — a record, respectively tuple, with no fields/positions at all. This is the unique unit value, in each notation.

`{}` and `()` are therefore not interchangeable with each other (they are the tops of two distinct subtyping orders — the record order and the tuple order — not a single shared top), and neither is the unit type; `{.}` and `(.)` are.

### 9.5 Tuple spreads

Tuple spread, written with the same `..` token as record spread, is **concatenation by position**, not merge by name:

```text
(..a, ..b)
```

places every element of `a` before every element of `b`, renumbered contiguously; no element of either operand is discarded or overridden.

Because tuples are open by default, a spread operand's length may not be statically known — its declared type may admit additional trailing elements beyond what is visible. Renumbering subsequent elements correctly requires knowing exactly how many positions a non-final spread operand contributes. The rule: **a spread operand must be statically closed (closed-total, which for tuples is the only kind of closed — §9.1a) unless it is the last element in the literal.** A spread in final position needs no such guarantee, since nothing follows it to be mis-numbered.

This can be satisfied explicitly in either of two ways:

- Projecting out a known number of elements by hand, e.g. `(a.0, a.1, a.2, ..b)` — here `a.0`, `a.1`, `a.2` are three independent projections, not a spread, so the closed-unless-last rule does not apply to them at all; the type of the enclosing literal is built from three explicitly-typed elements plus whatever `b` contributes.
- Using the close operator, `a.*`, to produce a closed-total tuple from `a`'s statically-declared elements (§8.4a), when the number of elements to keep is not known as a literal at the write site (e.g. inside generic code).

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

The capitalization convention distinguishes type declarations from value declarations. Type declarations may be self-referential or mutually recursive, subject to the guardedness condition of §3a.

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

The compiler may remove a private field that cannot be observed outside the construction. Note that observability here must account for equality: because `==` requires closed-total operands, and a value's own defining module can compare it including its private fields, a private field is only safely elidable when no reachable `==` comparison (within its defining module, where the field is visible) could depend on its presence. This is a stronger condition than "unused outside this module" and may require whole-function or whole-module analysis to establish, not merely a local unused-field check.

### 12.3 Returning one value with `<<`

A final `<<` expression returns a value instead of the constructed record. It is semantically equivalent to binding the trailing expression as a private-style final field and immediately projecting it:

```text
compute_total { items } =
  {
    _subtotal: sum items,
    tax: _subtotal * tax_rate,
    << _subtotal + tax
  }
```

is equivalent to:

```text
{
  _subtotal: sum items,
  tax: _subtotal * tax_rate,
  _result: _subtotal + tax
}._result
```

without requiring a field to be named solely in order to be immediately projected.

Rules:

- `<<` may appear at most once;
- `<<` must be the final element;
- `<<` is not a field;
- preceding fields are in scope for it;
- without `<<`, the value is the complete struct.

Because `<<`'s right-hand side is an ordinary expression, it may itself contain a `?`-expression with differently-shaped arms; the resulting type is governed by §13.1's union rule, exactly as for any other expression position — no special case is needed here or at any other place an expression may appear.

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

**Type of a `?`-expression.** Each arm's right-hand-side expression may have a different type. The type of the overall `?`-expression is the union of the arms' types — `pattern_1 -> e_1, ..., pattern_n -> e_n` has type `T_1 | ... | T_n`, where `T_i` is the type of `e_i` — with no merging or collapsing across arms. In particular, when arms are struct literals with different fields, the result is a union of the (closed, per §7.4) record types of each arm, not a single record type with field-level optionality:

```text
bool_expr ? { x: 2 } : { y: 2 }
```

has type `{x: Int, .} | {y: Int, .}`, not `{x: Int|Nothing, y: Int|Nothing, .}` — the latter would admit values (such as one with both fields, or neither) that this expression can never actually produce. This rule applies uniformly wherever a `?`-expression appears — as a struct field's value, as the right-hand side of `<<` (§12.3), as a function body, or anywhere else an expression is expected — since it is a property of `?` itself, not of the surrounding context.

### 13.2 Boolean conditional sugar

A two-branch Boolean match may use:

```text
condition ? when_true : when_false
```

This is sugar for matching over the Boolean enumeration, and inherits the union-typing rule of §13.1 directly.

### 13.3 Patterns

Patterns include:

- symbols;
- enumeration members;
- tuple patterns;
- record patterns;
- wildcard `_`;
- binding identifiers.

Tuple patterns and record patterns are kept as separate pattern productions (rather than one collapsing into the other), consistent with tuples and records being distinct type formers (§9.1a).

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

A resumable continuation is naturally typed using recursive, arrow-guarded types (§3a): a continuation that may itself be resumed into another resumable point needs a self-referential type of roughly the shape

```text
Cont A = A -> (Result | Cont A)
```

i.e. "given an `A`, produce either a final result or another continuation expecting another `A`." The exact syntax and semantics for resumable handlers, including how such a continuation type is spelled in this language, are **TBD**, but the arrow-guarded recursion permitted by §3a.1 is expected to be the mechanism.

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

A module's meaning must not depend on unrelated modules elsewhere in the build graph — see §24 for why a whole-program-derived numeric universe was rejected on these grounds.

## 19. Equality

Equality compares complete runtime values. `==` requires both operands to be **closed-total** (§7.3a): a value type with no possible additional fields or positions of any kind, public or private.

When both operands' static types are already closed-total, no explicit action is needed — the comparison is well-formed as written. When an operand's static type is open, or closed-public but not provably closed-total (as with a value of a type carrying private fields, viewed from outside its defining module), `==` is a type error until each such operand is closed explicitly with `.*` (§8.4a).

This has the effect that a value's own defining module — where any private fields it carries are visible, and can therefore be included by `.*` — can compare it including that private structure, while code outside the defining module can only compare the erased, publicly-visible projection. This is what makes private-field structural branding (§8.3) unforgeable at the value level, not merely at the type level: an external caller cannot manufacture a value that is `==`-equal to a genuine branded value once compared inside the defining module, since it cannot reproduce a private field it can neither name nor construct.

Because `.*` is a static-type-directed, module-relative operation with no special-casing for privacy in its own definition (§8.4a), this whole account requires no exception to §8.2's visibility rules — equality simply requires closed-total operands like any other structural operation would, and closing follows visibility automatically.

## 20. Standard library

The language core should remain small. The standard library may define:

- `Number` as a recursive, arbitrary-precision type (§3a.5), with `Int` and other bounded numeric types defined as ranges (§6) over it;
- `Bool` as a symbol enumeration;
- `Text` as `List Char` (§3a.5);
- `List` as the general recursive sequence type (§3a.5);
- `Unit` as `{.}` / `(.)` (§9.4);
- `Path`;
- `Option`;
- `Raise`;
- `IO`;
- other standard effects;
- enumeration operations;
- record and tuple utilities;
- iteration functions.

The exact primitive runtime representations are implementation concerns unless observable through language operations. In particular, how a `Number` or `List` is represented at runtime (a packed machine word, a linked structure, a single bit for a trivially small range, etc.) is entirely a compiler decision, per §6.1.

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
| `!` | Type negation (well-formed everywhere; resolves only within a bounding domain, §4.3) |
| `^` | Type symmetric difference / XOR (sugar over `|`, `&`, `!`, §4.5) |
| `?` | Pattern matching / Boolean conditional |
| `@` | Module reference/import |
| `<<` | Final expression from scoped record construction |
| `..` | Spread/update — merge-by-name for records (§7.5), concatenation-by-position for tuples (§9.5); not interchangeable between the two kinds |
| `.` | Field/position projection (`t.0`, `r.name`); also the closed-row marker when trailing in a type |
| `.*` | Close: erase to the statically-declared shape, producing a closed-total value (§8.4a) |
| `'` | Symbol literal |
| `#` | Comment |
| `{}` | Record construction/pattern; as a bare type, the open (unconstrained) record top (§9.4) |
| `()` | Tuple construction/pattern; as a bare type, the open (unconstrained) tuple top (§9.4) |
| `{.}` | The closed empty record type — the unit value in record notation (§9.4) |
| `(.)` | The closed empty tuple type — the unit value in tuple notation (§9.4) |
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
. closed rows / projection
.* close operator
| unions
& intersections
! negation
^ symmetric difference
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
- tuple spreads;
- lambdas;
- function application;
- field/position projection;
- the close operator `.*`;
- arithmetic and ordinary operators;
- scoped record fields;
- `<<` final expressions.

### Stage 3: types

Implement:

- type identifiers;
- records;
- tuples;
- open, closed-public, and closed-total rows;
- unions;
- intersections;
- negation (well-formed everywhere, resolved only within a bounding domain);
- symmetric difference (`^`);
- recursive type declarations, with the guardedness check (§3a);
- function types;
- effect annotations.

### Stage 4: declarations

Implement:

```text
name = expression
name : Type = expression
TypeName = TypeExpression
```

Use identifier casing to distinguish value and type declarations. Type declarations may be mutually recursive subject to guardedness (§3a.1–3a.2).

### Stage 5: patterns and matching

Implement:

- wildcard;
- bindings;
- symbol constants;
- tuple patterns;
- record patterns (kept distinct from tuple patterns, §9.1a);
- catch-all branches;
- `?`, including the arm-union typing rule of §13.1.

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
- private-field identity;
- module-relative visibility as consumed by `.*` (§8.4a) and `==` (§19).

## 3a. Recursive types

### 3a.1 Self-referential type declarations

A type declaration may refer to itself, directly or through other type declarations:

```text
List A = () | (A, List A, .)
```

This reads as: a `List` of `A` is either the empty tuple, or a closed pair of an `A` and another `List A`.

Unrestricted self-reference is not permitted. A recursive occurrence of a type name must be **guarded**: every recursive occurrence must appear strictly inside a closed tuple or an arrow (function) type, never as a bare alias and never inside a union or intersection on its own.

```text
List A = () | (A, List A, .)   # legal: recursive occurrence is inside a closed tuple
Cont A = A -> (Result | Cont A) # legal: recursive occurrence is inside an arrow's output
X = X                           # illegal: unguarded
X = X | Int                     # illegal: unguarded
```

The tuple guard must specifically be a **closed** tuple (§9.3), not an open one: open tuples admit unknown trailing elements of unconstrained type, and combining that openness with self-reference would make the shape of recursive structures much harder to reason about for negligible practical gain. Recursive type definitions are expected to use closed tuples even though tuples are open by default (§9.2).

Arrow types are permitted as a guarding constructor in both the input (contravariant) and output (covariant) position, following Amadio & Cardelli's original treatment of recursive subtyping (§3a.4); the variance of the position must be tracked through the subtyping check, not merely the presence of the constructor.

### 3a.2 Mutual recursion

Recursion may span a group of declarations rather than a single one:

```text
Forest A = []
Tree A = (A, Forest A, .)
```

Here `Forest A` and `Tree A` refer to each other. The guardedness condition (§3a.1) applies to the group as a whole: every path from a type name back to itself, however many other declarations it passes through, must cross at least one closed tuple or arrow type.

### 3a.3 Finite values, infinite types

A recursive type declaration such as `List A` describes an infinite family of possible values — lists of every length. Any individual value is still finite: a list terminates in a finite number of steps at `()`. Equality (§19), which compares complete runtime values, therefore always terminates on any actual value, even though the type itself has infinitely many inhabitants. Type-level questions (subtyping, exhaustiveness) and value-level questions (equality, pattern matching) are separate concerns; only the former needs special treatment for recursive types.

### 3a.4 Subtyping and decidability

Recursive types, restricted to guarded self-reference through closed tuples and arrow types as above, correspond to **regular tree types**: types describable by a finite grammar even though they admit infinitely many values. Subtyping between such types is decided by translating each type into a finite tree automaton and checking language containment (equivalently, checking that `A & !B` denotes the empty type, per the general law in §4.4) via automaton emptiness, with variance tracked through arrow-type positions.

This is decidable, though nontrivial to implement. The approach follows the semantic-subtyping line of work on recursive, set-theoretic type systems:

- Amadio, R., Cardelli, L. *Subtyping Recursive Types.* POPL 1993 — the original decidability result for subtyping recursive types via automata on infinite trees, including arrow types in contravariant/covariant guarded position.
- Frisch, A., Castagna, G., Benzaken, V. *Semantic Subtyping: Dealing Set-Theoretically with Function, Union, Intersection, and Negation Types.* ACM TOPLAS, 2008 — the semantic-subtyping framework this specification's union/intersection/negation model (§4) is based on, extended to recursive types.
- Benzaken, V., Castagna, G., Frisch, A. — the CDuce language and its implementation, a working system using this technique for XML-oriented recursive types.

An implementation is not required to support the general algorithm from the outset; §4.4 already reserves the option of "a restricted decidable representation" for an initial implementation. This section makes explicit what that representation is expected to converge toward.

### 3a.5 List, Text, and Number as recursive types

With guarded recursion available, unbounded standard-library types need no bedrock primitive support beyond the recursion mechanism itself:

```text
List A = () | (A, List A, .)
```

```text
Text = List Char
Number = Sign & (Digit, List Digit, .)     # arbitrary-precision; exact digit-list encoding TBD
```

A numeral literal is not a separate concept layered on top of `Number` — it denotes a `Number` value directly, under whatever desugaring into the digit-list representation is settled (§24). This keeps the type system's primitive vocabulary small: enumerations (§5) and ranges (§6) remain finite constructs, and every unbounded type in the standard library is an instance of the one recursive mechanism defined here.

Note that recursion through an arrow type's output, as permitted by §3a.1, is what a hypothetical function-top (`[] -> Top`) would require — but since the language has no top type (§3.3), this guard form is used for genuinely recursive function protocols (§16.5, continuations; state-machine/typestate APIs; parser combinators) rather than for constructing a universal type.

## 24. Intentionally unresolved questions

1. Exact import grammar after `@`.
2. Exact syntax for open and closed empty record/tuple types beyond the notation fixed in §9.4 (`{}`/`()` vs `{.}`/`(.)`) — e.g. whether any shorthand is needed.
3. Closed-row semantics in the presence of inaccessible private fields — resolved in outline by the three-state row model (§7.3a) and the close operator (§8.4a); the exact typing rules for `.*` still need to be spelled out formally (what type it assigns in each of the open / closed-public / closed-total cases).
4. Whether type declarations use `=` or a separate punctuation form.
5. Type-level reflection syntax for enumerations.
6. Enumeration order and duplicate-member behavior.
7. Exact digit-list encoding of `Number` (sign representation, leading zeros, digit order) and the desugaring rule from numeral literals to `Number` values.
8. Exact pattern grammar and binding behavior.
9. Whether a final bare identifier pattern is always a catch-all binding.
10. Whether general effect declarations are available in version one.
11. Syntax and semantics for resumable handlers and continuation binding — expected to use arrow-guarded recursive types (§3a.1, §3a.5), exact form TBD.
12. Exact syntax for generic `Raise` operations.
13. Private-module visibility boundaries.
14. Whether `for` syntax exists or is entirely library-based.
15. Whether top-level declarations may be mutually recursive — resolved: yes, subject to the guardedness condition of §3a.1–§3a.2.
16. Whether public values can explicitly hide or project fields.
17. Exact static semantics of finite enumeration reflection.
18. A global top type (`Any`/`Top`) was considered and deliberately not adopted: the one concrete motivating use case (a universally-accepting function such as `print : Any -> Text`) was rejected as not meaningful (what does it mean to print a function or closure?), and the other motivating case (XOR types) turned out not to need one, since `A ^ B` only ever uses negation inside an intersection with `A | B` (§4.5), which never requires a domain wider than the two operands. Constructing a global top would additionally require resolving a separate, disjoint infinite primitive for symbols (unbounded by name, not by structure, and not reachable via the recursive-tuple/arrow mechanism of §3a) on top of records, tuples, and functions. This remains open only in the sense that if a concrete need for global negation arises, the construction would need function-top, symbol-top, and record/tuple-top unified, each independently justified.
19. Rejected: deriving a numeric universe by scanning the largest range literal across all modules in a build. This does not bound `Number`, which is unbounded by definition and can't be bounded by any finite scan; and it violates §18.6's requirement that a module's meaning not depend on unrelated modules elsewhere in the build graph, since adding an unrelated module could silently enlarge the inferred universe and change type-checking results elsewhere.
20. Formal typing rules for `.*` (item 3, restated precisely): given a value of static type `T`, what is the type of `T.*`, in each of the open / closed-public / closed-total cases, for both records and tuples.
