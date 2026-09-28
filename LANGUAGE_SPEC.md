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
- Struct literals provide both data construction and local bindings, and a module file is a struct body without braces.
- `=` always binds and `:` always gives a type, at every level.
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

- **Type and namespace identifiers:** PascalCase. These name types (`Int`, `Text`, `UserId`), module aliases (`Toml`, §18.3), and effects (`IO`, `Raise`, §14). `Name.member` is always namespace access: an operation of an effect, or a declaration of an aliased module.
- **Value identifiers:** lowercase snake_case, for example `parse_text`, `user_id`.
- **Private names:** names beginning with `_`. These are private fields (`_brand`, §8), private top-level declarations (`_helper`, `_Token`, §18.5), and private modules (`_internal`, §18.5).
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
List(A)      # type application (§11.4)
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

The language has no universal/top type, and this is a deliberate, settled decision rather than an open gap. `!A` is meaningful only relative to a bounding domain (§4.4); there is no type inhabited by every value in the language, and no single type that accepts any argument. The motivating use cases were considered and did not hold up: a universally-accepting function such as `print : Any -> Text` was rejected as not meaningful (printing a function or closure has no defined behavior), and XOR types (§4.5) turned out not to need a top at all, since `A ^ B`'s negation is always immediately bounded by `A | B`. Constructing a global top would also require a separate, disjoint infinite primitive for symbols (unbounded by name, not by structure, and not reachable via the recursive-tuple/arrow mechanism of §3a) on top of records, tuples, and functions — should a concrete need for global negation arise later, that unification is the open work, not a small addition.

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
Digit = 0..9,
Port = 1..65535,
Percentage = 0..100,
```

### 6.3 Boolean values

Boolean values can be defined as symbols:

```text
Bool = ['false, 'true]
```

The quoted syntax makes constants visibly different from binding patterns.

## 7. Records / structs

### 7.1 Record values

Records use field labels and expressions, with `=` binding each field to its value (§11):

```text
{
  name = "Ada",
  age = 36
}
```

Records are immutable.

### 7.2 Open record types

A record type gives each field a type with `:`. A *declared* record type is open by default:

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

The `.` marker in type position denotes closed-public in general, degenerating to closed-total exactly when no private fields can be present. The distinction matters most for equality (§19) and the close operator (§8.4a); §8.4b gives closed-total a precise, recursive characterization (`Close(T) = T`) rather than leaving it at the informal description above.

Tuples have no analogue of this three-way split; see §9.1a.

### 7.4 Record construction and literal closedness

Record construction uses the same general syntax:

```text
{
  name = "Ada",
  age = 36
}
```

A struct literal expression is closed by construction, not merely closed by default: since a literal can only ever denote the fields written in it (or, transitively, spread into it), nothing beyond what is present in the source could possibly be part of its value. This holds regardless of control flow — a struct literal appearing inside either arm of a `?`-expression is still closed in each arm; the type of the overall expression is then governed by §13.1's rule for combining arm types (a union of the two closed record types, not a single record with optional fields).

The one qualification: a literal that spreads an operand which is itself open inherits that operand's openness, since the spread may carry unknown additional fields (§7.5). A literal with no spreads, or spreading only closed operands, is closed — closed-total if none of the spread operands carry private fields observable at this point in the source, closed-public otherwise.

### 7.5 Record updates and spreads

Record spreads use `..`:

```text
{
  ..defaults,
  timeout = 5000,
  retries = 3
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
  timeout = 5000
}
```

The final `timeout` wins.

This merge-by-name semantics is specific to records. Tuple spread is a different operation (concatenation-by-position) and the two are not interchangeable — a tuple cannot be spread into a record literal or vice versa (§9.1a, §9.5).

## 8. Private fields

### 8.1 Syntax and identity

Fields beginning with `_` are private:

```text
{
  _brand: (.),
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

These restrictions apply to the defining module's own submodules too. Private *declarations* (top-level `_` names and `_` modules) are visible to descendant modules (§18.5), but private *field labels* are not: a `_brand` written in module `auth/session` always denotes the label `(auth/session, _brand)`, never a `_brand` of its ancestor `auth`. The reason is that field labels are not declared anywhere; they exist wherever they are written. If a label written in a child could resolve to an ancestor's label, its meaning would depend on which labels happen to be used elsewhere in the ancestor's subtree, and adding a `_brand` to an unrelated part of `auth` could silently change what the child's code refers to. A descendant works with an ancestor's private fields through private helper functions the ancestor declares, which it can see. An explicit, module-qualified syntax for naming an ancestor's field labels may be added later (§24).

### 8.3 Structural branding

Private fields provide unforgeable structural brands without making the enclosing type fully nominal.

Example:

```text
UserId = {
  _brand: (.),
  text: Text
}
```

Code outside the defining module cannot construct a value satisfying the private `_brand` field. This includes the defining module's own descendants (§8.2); they create branded values only through functions the defining module provides, so every branded value is still created by code in the defining module.

A branded value may still be usable as a public structural supertype, and public fields remain structurally visible. From outside the defining module, `UserId`'s effective visible type is closed-public, not closed-total (§7.3a): the `_brand` field is real but unnameable, so external code must not assume the record has exactly one field.

### 8.4 Private fields and spreads

A spread copies only fields visible in the current module.

If a record contains private fields from another module, those fields are omitted from the spread.

Inside the defining module, its own private fields are visible and are preserved by spreads.

Thus, outside the defining module:

```text
{ ..id, text = "replacement" }
```

does not preserve the private brand of `id`.

### 8.4a The close operator, `.*`

`.*` produces a closed-total value from any record or tuple, by keeping only the fields or positions that are **statically visible in the declared type at the point of use**, and discarding everything else. It performs no runtime inspection of the value: the result is determined entirely by the static type, exactly as with the closed-unless-last rule for tuple spreads (§9.5).

```text
user : UserId = create_user("foo"),

is_foo = { text = "foo" } == user.*,
```

Outside `Auth` (the module defining `UserId`), `_brand` is not statically visible, so `user.*` erases it, producing a closed-total `{text: Text, .}` comparable against the literal. Inside `Auth`, the same operation on the same value preserves `_brand`, since it is visible there — `.*` gives different results at different points in the source not because it inspects the value differently, but because it always follows what is nameable at that point, and privacy already restricts that per §8.2. No special-casing of privacy is required in `.*`'s definition itself.

`.*` is the general "close to statically-known shape" operator for both records (keyed by name) and tuples (keyed by position); it is what allows a non-final tuple spread's length to be pinned down explicitly (§9.5) and what allows `==` to accept operands that are not already closed-total (§19).

### 8.4b Formal typing rule for `.*`

Given a value expression `e` with static type `T`, the type of `e.*` is computed by a type-level operation, written `Close(T)` here. `Close` is what §8.4a's "erase to the statically-declared shape" means made precise; it never inspects a runtime value, only the type the checker assigns to the result of `.*`.

`Close` works on the row (record or tuple) that `T` denotes **at the point of use** — i.e., after visibility (§8.2) has already been applied, so a private field not nameable at the call site is simply absent from the row `Close` starts from, and one that is nameable there is present:

1. **Record row.** If, at the point of use, `T`'s row is `{f_1: A_1, ..., f_n: A_n}` (whether written open or closed-public — the row-state itself plays no further part):

   ```text
   Close(T) = { f_1: Close(A_1), ..., f_n: Close(A_n), . }
   ```

   The result is always written closed, and is provably closed-total (§7.3a): every field not among `f_1, ..., f_n` — public or private, known or hidden — has been discarded, and every remaining field's own type has itself been closed, recursively.

2. **Tuple row.** Symmetrically, if `T`'s row is `(A_0, ..., A_{k-1})`:

   ```text
   Close(T) = (Close(A_0), ..., Close(A_{k-1}), .)
   ```

3. **Union.** `Close(A | B) = Close(A) | Close(B)` — a union of closed-total arms is closed-total; whichever arm a given runtime value inhabits, it is that arm's row that gets closed.

4. **Intersection.** An intersection of record or tuple types is reduced to its merged row (§4.2, §7.2) before rule 1 or 2 applies; `Close` needs no separate intersection case.

5. **Named row types.** A declared name such as `UserId` is unfolded once, to the row visible at the point of use, and rules 1–2 recurse into its fields exactly as for a written-out row literal. This is what lets `.*` do anything at all on a value declared with a named type, rather than treating the name as opaque — it is exactly what §8.4a's worked example relies on.

6. **Recursive types — termination.** Record and tuple field types may themselves be recursive (§3a), so rule 5 must not unfold the same named type twice along one recursion path, or `Close` would not terminate. The rule: once a named type has been unfolded during a given application of `Close`, a second, nested occurrence of that same name along the same path is left as-is — `Close` stops there rather than unfolding it again. This is the same state-repetition criterion that already makes subtyping over recursive types decidable (§3a.4, via automaton emptiness); `Close` reuses it rather than needing a separate termination argument.

   This gives, as a direct consequence rather than a separate design choice, the right scope for library containers built from recursion (§3a.5). For `List(A) = (.) | (A, List(A), .)`:

   ```text
   Close(List(A)) = (.) | (Close(A), List(A), .)
   ```

   The head element's type is closed, one level; the tail (`List(A)`, the same name reappearing) is left untouched, because unfolding it again is exactly the repeated occurrence rule 6 forbids. `.*` therefore closes a value's own shape, plus whatever record/tuple structure is nested directly inside it — but it does not walk an unbounded list, string, or other recursive container closing every element in turn. Closing every element of a `List({x: Int})`, for instance, is a `map (\e -> e.*) list` away, not something `.*` does by itself on the list as a whole. This boundary is not an ad hoc restriction picked for this operator specifically; it falls directly out of reusing rule 6's termination criterion, already justified independently in §3a.4.

7. **Everything else.** A type with no row to close — a primitive, an enumeration or range, a symbol type, or a function (arrow) type — is left unchanged: `Close(A) = A`. This includes a function type's own argument and result types; `.*` does not recurse into what a function accepts or produces, since a function value has no observable row of its own for `.*` to act on.

**Closed-total, restated precisely.** §7.3a's three-state model can now be stated as: `T` is closed-total exactly when `Close(T) = T` — `T` is already a fixed point of closing, so `.*` on a value of that type is a genuine no-op. `Close` is idempotent in general (`Close(Close(T)) = Close(T)`) for the same reason: whatever `Close` produces is, by construction, already stable under a further application. This is also the precise form of the requirement `==` imposes on its operands (§19): an operand's type must satisfy `Close(T) = T`, not merely "look closed at the top level" — a record with an open field is not closed-total by this definition even if its own row has a trailing dot, and `==` on it remains a type error until `.*` is applied.

**Worked examples.**

```text
UserId = { _brand: (.), text: Text }   # declared inside module Auth
```

Outside `Auth`, `_brand` is not nameable, so the row visible at that point is `{text: Text}`:

```text
Close(UserId) = { text: Close(Text), . } = { text: Text, . }     # (outside Auth)
```

Inside `Auth`, `_brand` is visible, so:

```text
Close(UserId) = { _brand: Close((.)), text: Close(Text), . }
              = { _brand: (.), text: Text, . }                    # (inside Auth)
```

For a non-final tuple spread operand of open type `(Int, Text)` (§9.5):

```text
Close((Int, Text)) = (Close(Int), Close(Text), .) = (Int, Text, .)
```

For a record with a nested open field, with no private fields anywhere:

```text
T = { a: { x: Int }, y: Text }

Close(T) = { a: Close({x: Int}), y: Close(Text), . }
         = { a: { x: Int, . }, y: Text, . }
```

`a`'s value has any extra fields beyond `x` discarded too, one level down — the recursive case of the same "no runtime inspection, follows the static type" principle §8.4a already states for the shallow, single-level case.

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

This is syntactic sugar for a unary lambda accepting one record. The parameter pattern may give types, using the pattern form of §13.3:

```text
sum { left: Int, right: Int } =
  left + right
```

### 10.3 Named arguments

Named arguments are ordinary record construction:

```text
sum {
  left = 1,
  right = 2
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

## 11. Declarations and bodies

The language has no `fn`, `function`, `type`, `let`, `import`, `handle`, `return`, or other reserved word keywords.

Two symbols do all the work, each with one meaning everywhere:

- `=` **binds**: a name to a value, a type name to a type, a field to its value, an alias to a module.
- `:` **gives a type**: to a declared name, to a field of a record type, to a pattern (§13.3), or to an expression (§11.6).

### 11.1 Bodies

A **body** is a comma-separated sequence of declarations. It is the content of a struct literal between its braces (§12), and it is the whole content of a module file (§18.8), which is a body without braces. The same rules apply to both. A trailing comma is allowed.

A body may contain:

- value declarations (§11.2);
- type declarations (§11.3);
- imports (§18.3), which are declarations that bind names from another module;
- in a struct literal only, spreads (§7.5) and a final `<<` (§12.3).

### 11.2 Value declarations

```text
name = expression
name : Type = expression
```

For example, in a module and in a struct literal alike:

```text
increment : Int -> Int = \x -> x + 1,

totals = {
  subtotal : Int = sum items,
  tax = subtotal * tax_rate,
}
```

A named function may use direct pattern syntax (§10.2):

```text
sum { left: Int, right: Int } = left + right,
```

In a struct literal, a field written alone is shorthand for binding it to the variable of the same name, mirroring the record-pattern shorthand (§13.3): `{ path }` means `{ path = path }`.

### 11.3 Type declarations

Type names use PascalCase, and the capitalization convention distinguishes type declarations from value declarations:

```text
Int = -32768..32767,
Bool = ['false, 'true],
Port = 1..65535,
```

Type declarations may appear in any body, not only at the top of a module:

```text
compute_total { items } = {
  Line = { price: Int, qty: Int },
  lines : List(Line) = parse_lines items,
  << sum_lines lines
}
```

Types are not values, so a type declared in a struct literal is **not a field** of the resulting record. It is a binding in scope within the body. It may still appear in the types of values that leave the body: in a structural type system a type name is an alias for its definition, so a value of type `{ line: Line }` leaves as `{ line: { price: Int, qty: Int } }`. In a module, type declarations are exported and can be imported (§18.3).

Type declarations may be self-referential or mutually recursive, subject to the guardedness condition of §3a.

### 11.4 Type parameters

A type declaration may take type parameters, written as a parenthesized, comma-separated list after its name. The parameters are type identifiers (§2.2), bound by the declaration and in scope in its right-hand side:

```text
List(A) = (.) | (A, List(A), .),
Pair(A, B) = (A, B, .),
```

A parameterized type is used by applying it to type arguments in the same form: `List(Int)`, `Pair(Int, Text)`. Effects take parameters the same way: `Raise(ParseError)` (§14.7).

The argument list is not a tuple type. `Pair(Int, Text)` is `Pair` applied to two arguments, not to the tuple type `(Int, Text)`. To keep the two apart, this is the only form of type application: there is no juxtaposition form such as `List Int`.

This is one deliberate asymmetry with values. Every function takes exactly one argument (§10.1), and several conceptual arguments are one record or tuple. A type constructor may take several parameters, because types are not values and a parameter list is not a tuple.

### 11.5 Scope

Declarations are read top to bottom. A declaration may refer to declarations before it in the same body, and to declarations of enclosing bodies.

**Exception: types and functions.** A type declaration, or a value declaration whose right-hand side is a lambda (including the named-function form), may be referred to from anywhere in its body: before it, after it, and from within itself. This is what makes recursive and mutually recursive functions and types possible, at any level of nesting:

```text
{
  is_even = \n -> n == 0 ? 'true : is_odd(n - 1),    # refers forward to is_odd
  is_odd  = \n -> n == 0 ? 'false : is_even(n - 1),
  << is_even(10)
}
```

All other values may only refer backwards. There is one further check, because a function declared early may call a function declared later: a value may not **depend** on a value declared after it, whether directly or through the functions it calls. Here, `total` is rejected:

```text
{
  scale = \x -> x * factor(),      # refers forward to factor: allowed, it is a function
  total = scale(10),               # error: scale calls factor, which needs rate, declared after total
  rate = 3,
  factor = \_ -> rate,
}
```

With these rules, declaration order is always a valid evaluation order for the values of a body. Implementations may still evaluate independent declarations in any order (§12.4).

The same rules hold in a module, whose body is a module file (§18.8). Across modules, which may import each other within a package (§18.6), the same check applies: no value may depend on itself through any chain of modules.

### 11.6 Type ascription

An expression can be given a type with `:`, in parentheses:

```text
(expression : Type)
```

The expression's type must be a subtype of `Type`, and the ascribed expression has type `Type`. Ascription only widens what the checker knows about the value; unlike `.*` (§8.4a), it never changes the value.

The parentheses are required. They keep ascription from colliding with the Boolean conditional `condition ? a : b` (§13.2), whose `:` is the only use of `:` that is not about types.

## 12. Struct literals as scoped computation

### 12.1 Sequential field scope

A struct literal is a body (§11.1) in braces, and its fields are declarations. Every struct literal introduces sequential field bindings:

```text
{
  a = 41,
  b = a + 1,
  result = string(b)
}
```

Each field expression may reference fields declared earlier in the same struct. Types and functions are the exception, and may be referred to before they are declared (§11.5).

Forward references to other values are invalid:

```text
{
  a = b + 1,
  b = 41
}
```

Self-references are invalid:

```text
{
  a = a + 1
}
```

### 12.2 Structs as local bindings

Structs replace a separate local-variable or block syntax:

```text
compute_total { items } =
  {
    _subtotal = sum items,
    tax = _subtotal * tax_rate,
    total = _subtotal + tax
  }
```

The result is the whole struct.

Private fields can be temporary local values:

```text
{
  _subtotal = sum items,
  _tax = _subtotal * tax_rate,
  total = _subtotal + _tax
}
```

The compiler may remove a private field that cannot be observed outside the construction. Note that observability here must account for equality: because `==` requires closed-total operands, and a value's own defining module can compare it including its private fields, a private field is only safely elidable when no reachable `==` comparison (within its defining module, where the field is visible) could depend on its presence. This is a stronger condition than "unused outside this module" and may require whole-function or whole-module analysis to establish, not merely a local unused-field check.

### 12.3 Returning one value with `<<`

A final `<<` expression returns a value instead of the constructed record. It is semantically equivalent to binding the trailing expression as a private-style final field and immediately projecting it:

```text
compute_total { items } =
  {
    _subtotal = sum items,
    tax = _subtotal * tax_rate,
    << _subtotal + tax
  }
```

is equivalent to:

```text
{
  _subtotal = sum items,
  tax = _subtotal * tax_rate,
  _result = _subtotal + tax
}._result
```

without requiring a field to be named solely in order to be immediately projected.

Rules:

- `<<` may appear at most once;
- `<<` must be the final element;
- `<<` is not a field;
- preceding fields are in scope for it;
- without `<<`, the value is the complete struct;
- `<<` may not appear in a module body (§18.8), since a module is a set of declarations, not a single value.

Because `<<`'s right-hand side is an ordinary expression, it may itself contain a `?`-expression with differently-shaped arms; the resulting type is governed by §13.1's union rule, exactly as for any other expression position — no special case is needed here or at any other place an expression may appear.

### 12.4 Evaluation dependencies

Field evaluation is constrained by data dependencies rather than necessarily by declaration order.

A field must be evaluated after every earlier field it references. Independent fields may be evaluated in any order.

Example:

```text
{
  a = compute_a(),
  b = compute_b(),
  c = a + 1,
  d = b + 1
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

**Arm order.** Arms are tried in order, and the first arm whose pattern matches is taken. Patterns may overlap; order decides which arm wins.

**Type of a `?`-expression.** Each arm's right-hand-side expression may have a different type. The type of the overall `?`-expression is the union of the arms' types — `pattern_1 -> e_1, ..., pattern_n -> e_n` has type `T_1 | ... | T_n`, where `T_i` is the type of `e_i` — with no merging or collapsing across arms. In particular, when arms are struct literals with different fields, the result is a union of the (closed, per §7.4) record types of each arm, not a single record type with field-level optionality:

```text
bool_expr ? { x = 2 } : { y = 2 }
```

has type `{x: Int, .} | {y: Int, .}`, not `{x: Int|Nothing, y: Int|Nothing, .}` — the latter would admit values (such as one with both fields, or neither) that this expression can never actually produce. This rule applies uniformly wherever a `?`-expression appears — as a struct field's value, as the right-hand side of `<<` (§12.3), as a function body, or anywhere else an expression is expected — since it is a property of `?` itself, not of the surrounding context.

### 13.2 Boolean conditional sugar

A two-branch Boolean match may use:

```text
condition ? when_true : when_false
```

This is sugar for matching over the Boolean enumeration, and inherits the union-typing rule of §13.1 directly.

The `:` here is the only `:` in the language that does not give a type. It cannot collide with type ascription, because ascription inside an expression is always parenthesized (§11.6): `c ? (a : T) : b`.

### 13.3 Patterns

A pattern denotes a set of values, exactly as a type does, together with a set of names it binds. As everywhere else (§11), `=` binds and `:` gives a type: in a pattern, **a type only ever appears after `:`**. Everything else in a pattern describes values.

| Pattern | Matches | Binds |
|---|---|---|
| `_` | any value | nothing |
| `name` (lowercase) | any value | `name` |
| `'sym` | the symbol `'sym` | nothing |
| literal, e.g. `0`, `"yes"` | that value | nothing |
| `p: T` | values matching `p` that are of type `T` (a **type test**) | the bindings of `p` |
| `(p_0, ..., p_k)` | tuples with at least these positions, each matching its `p_i` | the bindings of each `p_i` |
| `(p_0, ..., p_k, .)` | tuples with exactly these positions | as above |
| `{ f_1 = p_1, ..., f_n = p_n }` | records with at least these fields, each matching its `p_i` | the bindings of each `p_i` |
| `{ f_1 = p_1, ..., f_n = p_n, . }` | records with exactly these visible fields | as above |
| `p \| q` | values matching `p` or `q` | the names bound on both sides (which must be the same) |

A field of a record pattern may carry a type, exactly as a field of an annotated declaration does: `{ x: Int = n }` matches a record whose field `x` is an `Int`, and binds that field to `n`.

**Record shorthand.** A field written without `= p` binds a name equal to the field:

- `{ x }` means `{ x = x }`;
- `{ x: Int }` means `{ x: Int = x }`.

So a record *type* written as a pattern binds its fields. This is what lets named functions declare typed parameters (§10.2): `sum { left: Int, right: Int } = left + right`. To test a field's type without binding it, write `{ x: Int = _ }`; to test a whole value against a record type without binding anything, write `_: { x: Int }`.

Rules:

- **Types only after `:`.** A type test is always written `p: T`. A type such as `Int`, `1..9`, `[1, 2, 3]` or `!Int` never stands alone in a pattern: to match any `Int`, write `_: Int`, and to match and bind one, write `n: Int`. This is why `{ x = Int }` is not a pattern: it would read as binding the *type* `Int` to `x`.
- **Extent of the type.** The type after `:` extends as far as it can, and ends at the first `->`, `,`, `=`, `)` or `}` that is not nested inside it. So `n: Int | Text` means `n: (Int | Text)`. To use a pattern `|` between two annotated patterns, parenthesize them: `(n: Int) | (n: Text)`.
- **Identifiers.** A lowercase identifier is always a binding and a quoted symbol is always a constant. An unquoted identifier never refers to an existing value, so a binding never accidentally compares against a constant in scope, and a constant never accidentally becomes a binding.
- **No repeated bindings.** A pattern may not bind the same name twice, except on the two sides of `|`.
- **Disjunction.** In `p | q`, both sides must bind the same set of names. A name bound on both sides has the union of its types from each side.
- **Open and closed.** Record and tuple patterns are open by default and closed with a trailing `.`, exactly as record and tuple types are (§7.2, §7.3, §9.2, §9.3). Tuple and record patterns are separate productions, consistent with tuples and records being distinct type formers (§9.1a).
- **Privacy.** A record pattern may name a private field only inside its defining module (§8.2). For the same reason, a type test against a type whose definition has private fields not visible at that point is a type error: it would observe the private field. Outside the defining module, such values are told apart by their visible structure or by narrowing (§13.4).
- **No arrow types in type tests.** A type test must be decidable from the value itself. Membership of a function value in an arrow type cannot be checked at runtime, so a type test naming an arrow type is a type error. (This is also why the type after `:` can safely stop at `->`.)

Example:

```text
describe : (Int | Text | 'none) -> Text =
  \x -> x ? {
    'none  -> "nothing",
    0      -> "zero",
    n: Int -> string(n),
    t      -> t
  }
```

The last arm's `t` has type `Text`; see §13.4.

### 13.3a Rest patterns

A rest pattern is the mirror image of a spread (§7.5, §9.5): where a spread puts the contents of one value into a larger one, a rest pattern takes a value apart and binds whatever the other sub-patterns did not name.

```text
{ x, ..others }      # record: binds x, and a record of all the other fields
(head, ..tail)       # tuple: binds position 0, and a tuple of all the later positions
```

Rules, common to both kinds:

- `..` in a pattern is followed by a binding identifier (or `_`), not by a general pattern.
- A pattern contains at most one rest, and it must be the last element.
- A pattern with a rest is open by nature (it matches any number of further fields or positions), so it cannot also carry a trailing `.`.

**Records.** `{ f_1 = p_1, ..., f_n = p_n, ..r }` matches any record with at least the fields `f_1, ..., f_n`, and binds `r` to a record holding every other field that is **visible at this point**. As with spread (§8.4), private fields of another module are not visible, so they are not captured: they are dropped, exactly as a spread would drop them. The module's own private fields are visible and are captured.

If the value reaching the arm (§13.4) has type:

```text
{ f_1: A_1, ..., f_n: A_n, g_1: B_1, ..., g_m: B_m }      # open
```

then:

```text
r : { g_1: B_1, ..., g_m: B_m }          # open: unknown further fields go into r
```

and if it is closed (`..., .`), then:

```text
r : { g_1: B_1, ..., g_m: B_m, . }       # closed-total
```

The closed case gives a closed-total `r`, not merely closed-public (§7.3a), because the only fields a closed-public row can hide are private fields of other modules, and those are precisely the ones the rest pattern drops. When nothing is left over, `r : {.}`.

**Tuples.** `(p_0, ..., p_{k-1}, ..t)` matches any tuple with at least `k` positions, and binds `t` to a tuple of the remaining positions, renumbered from `0`. If the value reaching the arm has type `(A_0, ..., A_{m-1})`, then `t : (A_k, ..., A_{m-1})`, open or closed as that type is. Tuples have no private positions (§9.1a), so nothing is dropped.

**Round trip.** Because rest is the mirror of spread, using the same shape as a literal rebuilds the value:

```text
v ? { { x, ..r } -> { x, ..r } }         # equals v, minus other modules' private fields
v ? { (h, ..t)   -> (h, ..t) }           # equals v exactly
```

The tuple case is well-typed even when `t` is open, because the spread is in final position (§9.5).

### 13.4 Narrowing

The type of a binding is the type of the values that can actually reach it. If the scrutinee has type `S` and the arms' patterns are `p_1, ..., p_n`, then the values that reach arm `i` are exactly:

```text
S_i = S & p_i & !(p_1 | ... | p_{i-1})
```

treating each pattern as the type it denotes. A binding in arm `i` has the type of its position within `S_i`.

**Bindings must be bounded.** Because there is no top type (§3.3), a binding needs a type that `S_i` actually determines. Binding a field or tuple position that the scrutinee's type does not mention, only admits through openness, is a type error, since that component could hold anything and "anything" has no type. Giving it a type in the pattern fixes this:

```text
v : { x: Int }            # open: v may have a field y, of unknown type

v ? { { x, y }      -> ... }    # type error: y's type is unbounded
v ? { { x, y: Int } -> ... }    # fine: y : Int, and the arm matches only if y is an Int
v ? { { x, ..r }    -> ... }    # fine: r : {} (unknown fields stay inside r's openness)
```

The last line shows why rest patterns (§13.3a) never have this problem: unknown components are captured as the openness of the rest binding, not given a type of their own. This is the same principle as for negation (§4.3): the type of anything must be bounded by something already known.

In particular, a bare identifier as the last arm is a catch-all whose type is whatever the earlier arms did not take:

```text
x : Int | Text | 'none

x ? {
  'none  -> ...,
  _: Int -> ...,
  rest   -> ...     # rest : Text
}
```

This is negation used within a bounding domain, as §4.3 requires: the domain is the scrutinee type `S`, so narrowing never needs a global top type.

### 13.5 Exhaustiveness and redundancy

Both checks are subtyping questions (§4.4) and use the same decision procedure as the rest of the type system (§3a.4).

- **Exhaustiveness.** A match is exhaustive when `S <: p_1 | ... | p_n`. A non-exhaustive match is a type error: the value of a `?`-expression must be defined for every value of its scrutinee.
- **Redundancy.** Arm `i` is redundant when `S_i = []`: no value can reach it. A redundant arm is a type error.

A bare identifier or `_` matches everything, so it makes any match exhaustive, and any arm after it is redundant. A bare identifier is an ordinary, irrefutable binding wherever it appears, not only in the last arm; placing arms after it is simply an instance of the redundancy rule.

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
A -> B // IO | Raise(ParseError)
```

This means the computation may perform either effect.

### 14.3 Effects are not values

Effects describe observable interaction or control behavior during evaluation. Examples include:

```text
IO
Raise(E)      # for any type E, §14.7
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

### 14.6 Built-in effects only

Version one has no user-defined effects. The effects available are a fixed set provided by the language and standard library, those listed in §14.3. The exact set, and the signatures of their operations, are **TBD** (§24). Programs handle these effects (§16), including resuming them (§16.5), but cannot declare new ones.

### 14.7 `Raise`

`Raise` takes one type parameter, the type of the value raised. Any type may be raised; there is no special error supertype:

```text
Raise.raise : E -> Never // Raise(E)
```

Two rules connect `Raise` to effect sets:

- **Merging.** `Raise(A) | Raise(B)` is the same effect as `Raise(A | B)`. A computation that may raise either kind of value has one `Raise` effect, and a handler clause `Raise.raise(error) -> ...` receives `error : A | B`.
- **Subtyping.** `Raise(A) <: Raise(B)` when `A <: B`. This follows from effect inclusion (§14.4) together with merging: `Raise(A) | Raise(B)` is `Raise(B)` when `A <: B`.

The result type of `Raise.raise` is `Never`, i.e. `[]` (§3.2), which is what makes it abortive (§16.5).

## 15. Effect operations

An effect operation may look like an ordinary function call:

```text
Raise.raise(error)
Random.next_int(range)
IO.read_file(path)
```

The compiler knows from the operation's built-in declaration that the call performs an effect. There are no user-defined effects in version one (§14.6).

Like every function, an operation takes exactly one argument (§10.1). For example:

```text
Raise.raise : E -> Never // Raise(E)
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

`Raise` is abortive: its result type is `Never` (§14.7), so it can never be resumed (§16.5). If it is handled, execution does not continue after the original raise point.

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

When an operation is handled, the handler clause can receive a continuation which, when called with a value, continues the original computation as if the operation call had returned that value.

Whether an operation can be resumed is decided by its type, not by a declaration. For an operation `Op : P -> Q`, the continuation has type `Q -> R` (§16.5.2). If `Q` is `Never`, i.e. `[]` (§3.2), no value exists to call it with, so the operation can never be resumed: it is **abortive**. Every operation whose result type is inhabited is **resumable**. `Raise.raise`, with result type `Never` (§14.7), is the standard abortive operation.

An implementation never needs to capture a continuation for an operation whose result type is `Never`, since it is statically known that none can be used.

### 16.5.1 Clause syntax

A clause may bind the continuation with a second, comma-separated binding after the operation pattern. For example, a test can supply the contents of a file instead of reading it:

```text
load_config { path = "app.toml" } // {
  IO.read_file(path), resume -> resume("port = 8080"),
}
```

The left-hand side of such a clause has the form:

```text
operation_pattern, resume_binding -> handler_expression
```

Rules:

- `resume_binding` is a binding identifier or `_`, not a general pattern: the continuation is a function value, and no structural pattern (§13.3) can match against one.
- This two-part form is a production of handler clauses only. It is not an extension of the general pattern grammar; tuple patterns elsewhere are unaffected.
- On the completion clause (§16.2) the second binding is a type error: a computation that has completed has no continuation.
- On a clause for an abortive operation such as `Raise.raise`, the second binding is permitted but useless: the continuation has type `[] -> R` and can never be called.
- The second binding is optional. A clause that omits it (or binds `_`) does not resume; its value becomes the result of the whole handled expression, exactly as for an abortive operation.

Once bound, `resume` is an ordinary unary function (§10.1) and is called with ordinary application, e.g. `resume("port = 8080")`.

Note the position of the comma. `IO.read_file(path), resume` binds the operation's single argument through `operation_pattern` and the continuation separately. It must not be confused with the form `IO.read_file(path, resume)`, an earlier sketch which was rejected: it implied that the continuation is a second argument of the operation, whereas every operation, like every function, takes exactly one argument (§10.1), and the continuation is supplied by the handler mechanism, not by the call.

### 16.5.2 Deep handlers

Handlers are **deep**. When a clause calls `resume`, the resumed computation remains under the same handler: any further operations it performs are handled by the same clauses. Consequently, `resume` returns the final result of the whole handled expression, not the next suspension point.

Typing. Let the handled expression be:

```text
h = e // { clauses }
```

and let `R` be the type of `h`: the union of the result types of all its clauses, including the completion clause (or, if there is no completion clause, of `e`'s own result type), per the union rule of §13.1. In a clause for an operation:

```text
Op : P -> Q // E_op
```

the bound continuation has type:

```text
resume : Q -> R // E
```

where `E` is the effect set of `h` itself: whatever effects of `e` this handler does not handle, plus the effects of the handler clauses (§14.5).

Example — fixing the outcome of random choices in a test:

```text
two_rolls : (.) -> (Int, Int) // Random =
  \_ -> (Random.next_int(6), Random.next_int(6)),

fixed : (Int, Int) =
  two_rolls() // {
    Random.next_int(_), resume -> resume(4),
  },
```

`fixed` is `(4, 4)`. The first `Random.next_int` is handled by the clause, which resumes with `4`. Because the handler is deep, the resumed computation is still handled by the same clause, so the second `Random.next_int` is too. The inner `resume` returns the completed result `(4, 4)`, and so does the outer one. Here `R` is `(Int, Int)`, and `resume : Int -> (Int, Int)`.

Because handlers are deep, the continuation type needs no recursion: it is a plain `Q -> R`. Explicitly recursive protocols such as `Sum = Int -> (Int | Sum)` (§3a.1) remain available for code that wants to hand a step-by-step protocol to its caller, but they are not how handlers work.

### 16.5.3 Multi-shot continuations

`resume` may be called zero, one, or many times. The language does not restrict this at the type level: doing so would require linear or affine types, which Tumul does not have.

Most practical effects (generators, streams, async, state) use each continuation at most once: each `resume` runs to the next operation, which produces a fresh continuation for that point. Calling the *same* continuation more than once is what makes backtracking, search, nondeterministic choice, and retrying with different arguments possible.

The example below uses a `Choose` effect. Version one programs cannot declare it (§14.6); it shows the pattern that multi-shot continuations enable, and which user-defined effects would make available later. With built-in effects, the same technique can, for instance, resume `Random.next_int` once for every possible value to enumerate all outcomes.

```text
Choose.choose : (.) -> Bool // Choose

all_outcomes : List(Outcome) =
  computation() // {
    Choose.choose(_), resume -> append { left = resume('true), right = resume('false) },
    value -> (value, ()),
  }
```

Each call to `resume` re-runs the remainder of the computation from the suspension point, **including its effects**. If the resumed code performs IO, that IO happens once per call to `resume`.

Multi-shot continuations are cheap and safe in Tumul because values are immutable (§1): capturing a continuation needs no copy of mutable state, and resuming it twice cannot corrupt shared data. An implementation may optimize continuations it can prove are used at most once; this is not observable.

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

### 17.2 No dedicated `for` syntax

There is no `for` construct. Iteration is expressed entirely through library functions such as `for_each`, `map`, and `fold` (§17.1); a hypothetical

```text
for color in colors {
  print color
}
```

is written directly as:

```text
for_each {
  values = colors,
  body = \color -> print color
}
```

This keeps the "no reserved word keywords" principle (§1) uncomplicated by a special-cased loop form, at the cost of that one extra layer of syntax for the common case.

### 17.3 Iteration effect

A dedicated `Iteration` effect is appropriate only for stateful, asynchronous, generator-based, or nondeterministic producers. Basic enumeration over immutable finite values is pure except for effects in the loop body.

## 18. Modules

### 18.1 Module tree

Modules are determined by file paths. No module declaration is required.

- A file is a module: `app/config.lang` is the module `app/config`.
- A directory with the same name as a module, next to its file, holds that module's submodules: `app/config/parser.lang` is the module `app/config/parser`, a submodule of `app/config`.
- The file may be omitted: a directory alone is a module with no declarations, containing only submodules.
- The directory may be omitted: a file alone is a leaf module, with no submodules.

```text
app/
  config.lang          # app/config
  config/
    parser.lang        # app/config/parser, a submodule of app/config
    _defaults.lang     # app/config/_defaults, a private submodule of app/config
  server/              # app/server: no file, submodules only
    routes.lang        # app/server/routes
```

A module's **parent** is the module one path segment up, and its **ancestors** are its parent, its parent's parent, and so on. Its **descendants** are the modules below it.

Submodules are not members of their parent: a declaration `parser` in `app/config.lang` and the submodule `app/config/parser` do not collide, because a submodule is only ever reached by its path (§18.3), never as `Config.parser`.

### 18.2 Roots and packages

Every module path starts with a root:

- `std` is the standard library.
- `ext/package_name` is the external package `package_name`. Each package's modules lie exactly under its `ext/package_name/` directory.
- Every other root belongs to the application.

`std` and `ext` are directory-only: they have no declarations of their own.

The code of a package can only exist under `ext/package_name`, and application code can only exist outside `std` and `ext`. So a module's subtree always lies within a single package (the standard library, one external package, or the application), and no code can add modules to another package's subtree. The visibility rule of §18.5 relies on this.

### 18.3 Imports

An import names a module by its full path, starting at a root, in one of two forms.

An import is a declaration (§11.1): it binds names, and it is separated from neighbouring declarations by a comma like any other. Imports usually open a module file, but may appear in any body; an import inside a struct literal is in scope only within that body.

Selected names, brought into scope unqualified:

```text
@std/text { trim, split },
```

An alias, through which the module's declarations are reached as `Alias.name`, and its types as `Alias.TypeName`:

```text
@ext/toml/parse = Toml,

document = Toml.parse(source),
```

Rules:

- The alias is a namespace identifier, so it is PascalCase (§2.2).
- A bare import such as `@std/text`, with neither selected names nor an alias, is not permitted. Importing every public name unqualified would crowd the scope, with no keywords to keep names apart, and there is no good default alias: `@std/text` would become `Text`, which is already a type.
- Paths are always absolute; there are no relative imports.
- An import may name a private module, or select a private declaration, only where that module or declaration is visible (§18.5).

Being visible does not put anything in scope: a module that can see its parent's private helper still imports it by name, like any other declaration.

### 18.4 Dependency management

Fetching external modules, dependency resolution, registries, lockfiles, versions, and placement are outside the language specification and compiler language semantics.

Source imports do not need to include dependency versions:

```text
@ext/toml/parse = Toml
```

The build system resolves `ext/toml` to a concrete package instance. The compiler must receive a unique internal identity for each resolved package instance, possibly based on a lockfile identity, content hash, package-store path, or build-graph identifier.

### 18.5 Private declarations and visibility

A top-level declaration whose name begins with `_` is **private**:

```text
_helper = \x -> x + 1,                 # private value
_Token = { kind: Text, text: Text },   # private type
```

A module whose file or directory name begins with `_` is a **private module**, treated as a private declaration of its parent:

```text
app/config/_defaults.lang              # a private declaration of app/config
```

**Rule:** a private declaration of module `M` is visible in `M` and in every descendant of `M`, and nowhere else. A module therefore sees its own private declarations and those of its parent and every other ancestor. It does not see the private declarations of its siblings, of their descendants, or of its own descendants.

Applied to the tree in §18.1:

- `app/config/parser` can use `app/config`'s private declarations, and can import `app/config/_defaults`, since that module is itself a private declaration of `app/config`.
- `app/config/parser` cannot see the private declarations *inside* `app/config/_defaults`: those belong to `_defaults`, and `parser` is its sibling, not its descendant.
- `app/server/routes` cannot import `app/config/_defaults` at all: it is not a descendant of `app/config`.
- `app/config` cannot see the private declarations of `app/config/parser`: visibility flows down the tree, never up.

Because a subtree never crosses a package boundary (§18.2), no package can see another package's private declarations.

This rule covers declarations only. Private field labels stay local to the module in which they are written, and are not visible to descendants (§8.2).

### 18.6 Import cycles

Modules within one package may import each other in cycles. This is common under §18.5: a parent imports its submodules' public declarations, and those submodules import the parent's private helpers. A package's modules are checked together as a unit, just as top-level declarations within a module may be mutually recursive (§3a).

Across packages, imports must not form a cycle: the application depends on packages, and packages depend on other packages, as a directed acyclic graph.

### 18.7 Module properties

Modules are:

- static;
- non-parameterized;
- non-generative;
- resolved before compilation;
- assigned one stable identity per application build.

A module's meaning must not depend on unrelated modules elsewhere in the build graph — see §24 for why a whole-program-derived numeric universe was rejected on these grounds.

### 18.8 A module is a body

The content of a module file is a body (§11.1) without braces: a comma-separated sequence of imports, type declarations, and value declarations, following the same scope rules as any struct literal (§11.5). A module is, in effect, a struct literal whose type declarations are also exported.

Three restrictions apply to module bodies only:

- **No `<<`.** A module is a set of declarations, not a single value.
- **No spreads.** A module's declarations are exactly those written in it.
- **Purity.** Evaluating a module's values must perform no effects: the effect set of a module body is empty. Importing a module therefore never performs IO, raises, or reads the clock.

A private declaration of a module (§18.5) is a private field of this struct. What makes it reachable from descendant modules, unlike private field labels in general (§8.2), is that a descendant names it together with its module, through an import path: `@app/config { _helper }`.

## 19. Equality

Equality compares complete runtime values. `==` requires both operands to be **closed-total** (§7.3a): a value type with no possible additional fields or positions of any kind, public or private, at any depth — precisely, a type `T` with `Close(T) = T` (§8.4b). This is a recursive requirement, not merely a check on the outermost row: a record whose own row is closed but which has an open field nested inside it is not closed-total, and `==` on it is a type error until that field, too, is closed.

When both operands' static types are already closed-total, no explicit action is needed — the comparison is well-formed as written. When an operand's static type is open, or closed-public but not provably closed-total (as with a value of a type carrying private fields, viewed from outside its defining module), `==` is a type error until each such operand is closed explicitly with `.*` (§8.4a).

This has the effect that a value's own defining module — where any private fields it carries are visible, and can therefore be included by `.*` — can compare it including that private structure, while code outside the defining module can only compare the erased, publicly-visible projection. This is what makes private-field structural branding (§8.3) unforgeable at the value level, not merely at the type level: an external caller cannot manufacture a value that is `==`-equal to a genuine branded value once compared inside the defining module, since it cannot reproduce a private field it can neither name nor construct.

Because `.*` is a static-type-directed, module-relative operation with no special-casing for privacy in its own definition (§8.4a), this whole account requires no exception to §8.2's visibility rules — equality simply requires closed-total operands like any other structural operation would, and closing follows visibility automatically.

## 20. Standard library

The language core should remain small. The standard library may define:

- `Number` as a recursive, arbitrary-precision type (§3a.5), with `Int` and other bounded numeric types defined as ranges (§6) over it;
- `Bool` as a symbol enumeration;
- `Text` as `List(Char)` (§3a.5);
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
| `:` | Gives a type: to a declaration, a record-type field, a pattern (§13.3), or a parenthesized expression (§11.6). Also the `else` separator of the Boolean conditional (§13.2), its only use not about types |
| `=` | Binds: a value or type declaration, a field in a struct literal or record pattern, or an import alias (§11) |
| `->` | Function type / lambda body separator |
| `\\` | Anonymous lambda |
| `//` | Effect annotation / effect handler |
| `\|` | Type union / effect union / pattern disjunction (§13.3) |
| `&` | Type intersection |
| `!` | Type negation (well-formed everywhere; resolves only within a bounding domain, §4.3) |
| `^` | Type symmetric difference / XOR (sugar over `|`, `&`, `!`, §4.5) |
| `?` | Pattern matching / Boolean conditional |
| `@` | Module reference/import |
| `<<` | Final expression from scoped record construction (not in module bodies, §18.8) |
| `..` | Spread/update — merge-by-name for records (§7.5), concatenation-by-position for tuples (§9.5); not interchangeable between the two kinds. In patterns, a rest pattern, the mirror of spread (§13.3a) |
| `.` | Field/position projection (`t.0`, `r.name`); also the closed-row marker when trailing in a type |
| `.*` | Close: erase to the statically-declared shape, producing a closed-total value (§8.4a) |
| `'` | Symbol literal |
| `#` | Comment |
| `,` | Separates the declarations of a body (§11.1), including a module file; also separates tuple positions, enumeration members, and type arguments |
| `{}` | Record construction/pattern; as a bare type, the open (unconstrained) record top (§9.4) |
| `()` | Tuple construction/pattern; as a bare type, the open (unconstrained) tuple top (§9.4) |
| `{.}` | The closed empty record type — the unit value in record notation (§9.4) |
| `(.)` | The closed empty tuple type — the unit value in tuple notation (§9.4) |
| `[]` | Finite enumeration / bottom type |

## 22. Combined example

```text
# app/config.lang

@std/text { trim },
@std/io { read_file },
@ext/toml/parse = Toml,

Port = 1..65535,

Config = {
  host: Text,
  port: Port,
  .
},

parse_config : Text -> Config // Raise(ParseError) =
  \text ->
    {
      document = Toml.parse(trim text),
      << {
        host = document.host,
        port = document.port
      }
    },

load_config : { path: Path } -> Config // IO | Raise(ParseError) =
  \{ path } ->
    {
      source = read_file path,
      << parse_config source
    },

try_load_config :
  { path: Path } -> Config | ParseError // IO =
  \{ path } ->
    load_config { path } // {
      Raise.raise(error) -> error,
      value -> value,
    },
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
- struct literals as bodies: `=` fields, field shorthand, local type declarations and imports (§11.1–11.3);
- scope: forward references only to types and functions, and the dependency check (§11.5);
- parenthesized type ascription `(e : T)` (§11.6);
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
- type application `Name(T_1, ..., T_n)` and parameterized type declarations (§11.4);
- recursive type declarations, with the guardedness check (§3a);
- function types;
- effect annotations.

### Stage 4: declarations

Implement:

```text
name = expression,
name : Type = expression,
TypeName = TypeExpression,
```

Use identifier casing to distinguish value and type declarations. The same declaration grammar serves struct literals and module files, which are both bodies (§11.1, §18.8). Type declarations may be mutually recursive subject to guardedness (§3a.1–3a.2).

### Stage 5: patterns and matching

Implement:

- wildcard;
- bindings;
- symbol constants and literals;
- type-test patterns `p: T`, with the type extending to the next `->`, `,`, `=`, `)` or `}` (§13.3);
- tuple patterns;
- record patterns `{ f = p }` and `{ f: T = p }`, including the shorthands `{ f }` and `{ f: T }` (kept distinct from tuple patterns, §9.1a);
- `|` patterns;
- rest patterns for records and tuples (§13.3a);
- `?`, including first-match arm order and the arm-union typing rule of §13.1;
- narrowing, exhaustiveness, and redundancy checks (§13.4, §13.5).

### Stage 6: effects

Implement:

- effect annotations;
- effect sets;
- operation calls;
- handler expressions;
- final normal-completion branches;
- the `, resume` continuation binding on resumable operation clauses (§16.5.1).

### Stage 7: modules

Implement:

- file-derived module paths and the module tree, including directory-only modules (§18.1);
- `@` imports, in the selected-names and alias forms (§18.3);
- private declarations and private modules, visible to the declaring module and its descendants (§18.5);
- import cycles within a package (§18.6);
- private-field identity, local to the module where a label is written (§8.2);
- module-relative visibility as consumed by `.*` (§8.4a) and `==` (§19).

## 3a. Recursive types

### 3a.1 Self-referential type declarations

A type declaration may refer to itself, directly or through other type declarations:

```text
List(A) = (.) | (A, List(A), .)
```

This reads as: a `List` of `A` is either the empty tuple `(.)`, or a closed pair of an `A` and another `List(A)`.

Unrestricted self-reference is not permitted. A recursive occurrence of a type name must be **guarded**: every recursive occurrence must appear strictly inside a closed tuple or an arrow (function) type, never as a bare alias and never inside a union or intersection on its own.

```text
List(A) = (.) | (A, List(A), .),   # legal: recursive occurrence is inside a closed tuple
Sum = Int -> (Int | Sum),         # legal: recursive occurrence is inside an arrow's output
X = X,                            # illegal: unguarded
X = X | Int,                      # illegal: unguarded
```

The tuple guard must specifically be a **closed** tuple (§9.3), not an open one: open tuples admit unknown trailing elements of unconstrained type, and combining that openness with self-reference would make the shape of recursive structures much harder to reason about for negligible practical gain. Recursive type definitions are expected to use closed tuples even though tuples are open by default (§9.2).

Arrow types are permitted as a guarding constructor in both the input (contravariant) and output (covariant) position, following Amadio & Cardelli's original treatment of recursive subtyping (§3a.4); the variance of the position must be tracked through the subtyping check, not merely the presence of the constructor.

### 3a.2 Mutual recursion

Recursion may span a group of declarations rather than a single one:

```text
Tree(A) = (A, Forest(A), .),               # a value and its children
Forest(A) = (.) | (Tree(A), Forest(A), .), # a list of trees
```

Here `Forest(A)` and `Tree(A)` refer to each other. The guardedness condition (§3a.1) applies to the group as a whole: every path from a type name back to itself, however many other declarations it passes through, must cross at least one closed tuple or arrow type.

### 3a.3 Finite values, infinite types

A recursive type declaration such as `List(A)` describes an infinite family of possible values — lists of every length. Any individual value is still finite: a list terminates in a finite number of steps at the empty tuple. Equality (§19), which compares complete runtime values, therefore always terminates on any actual value, even though the type itself has infinitely many inhabitants. Type-level questions (subtyping, exhaustiveness) and value-level questions (equality, pattern matching) are separate concerns; only the former needs special treatment for recursive types.

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
List(A) = (.) | (A, List(A), .)
```

```text
Text = List(Char),
Number = (Sign, Digit, List(Digit), .),   # illustrative only; exact digit-list encoding TBD (§24)
```

A numeral literal is not a separate concept layered on top of `Number` — it denotes a `Number` value directly, under whatever desugaring into the digit-list representation is settled (§24). This keeps the type system's primitive vocabulary small: enumerations (§5) and ranges (§6) remain finite constructs, and every unbounded type in the standard library is an instance of the one recursive mechanism defined here.

Note that recursion through an arrow type's output, as permitted by §3a.1, is what a hypothetical function-top (`[] -> Top`) would require — but since the language has no top type (§3.3), this guard form is used for genuinely recursive function protocols (step-by-step producers handed to a caller; state-machine/typestate APIs; parser combinators — note that effect-handler continuations do not need it, since handlers are deep, §16.5.2) rather than for constructing a universal type.

## 24. Intentionally unresolved questions

1. Type-level reflection syntax and static semantics for enumerations — what `values`, `size`, `ordinal`, and friends (§5.5) actually look like at the type-checking level, not just which operations should exist.
2. Enumeration order and duplicate-member behavior.
3. Exact digit-list encoding of `Number` (sign representation, leading zeros, digit order) and the desugaring rule from numeral literals to `Number` values.
4. The exact set of built-in effects and the signatures of their operations (§14.6).
5. Polymorphic function signatures: how type and effect variables in value signatures, such as `Element` and `E` in `for_each` (§17.1) or `E` in `Raise.raise` (§14.7), are introduced, and how parametric polymorphism combines with semantic subtyping and negation. Parameterized *types* are settled (§11.4); polymorphic *functions* are not. The theory is non-trivial; see Castagna, Nguyễn, Xu, Im, Lenglet, Padovani, *Polymorphic Functions with Set-Theoretic Types, Part 1: Syntax, Semantics, and Evaluation*, POPL 2014, and Castagna, Nguyễn, Xu, Abate, *Part 2: Local Type Inference and Type Reconstruction*, POPL 2015.
6. A module-qualified syntax that would let a descendant module name an ancestor's private field labels directly, instead of going through the ancestor's private helpers (§8.2). Not needed now; a possible later addition.

Resolved since the previous revision and removed from this list:

- The empty record/tuple notation (§9.4).
- Whether top-level declarations may be mutually recursive: yes, subject to guardedness (§3a.1–3a.2).
- Whether public values can explicitly hide or project fields: yes, via `.*` (§8.4a).
- The global top type: deliberately not adopted (§3.3).
- Deriving a numeric universe from whole-program range literals: rejected (§18.7).
- Whether a dedicated `for` syntax exists: no (§17.2).
- The formal typing rules for the close operator `.*`: a recursive `Close(T)` operation, terminating by the same state-repetition criterion as recursive-type subtyping, with closed-total characterized as `Close(T) = T` (§8.4b).
- Resumable handlers: continuations are bound by a separate `, resume` binding after the operation pattern, handlers are deep so `resume : Q -> R // E`, and continuations are multi-shot (§16.5).
- The pattern grammar and catch-all rules: patterns are set-theoretic like types, with type tests and `&`/`|`; arms match in order; bindings are narrowed by negation of earlier arms and must have a bounded type; exhaustiveness and redundancy are subtyping checks (§13.3–§13.5).
- Rest patterns, as the mirror of spread (§13.3a).
- Guards on match arms: considered and deliberately left out for now.
- The import grammar: two forms, selected names and alias; no bare imports; absolute paths only (§18.3).
- User-defined effects: none in version one; a fixed set of built-in effects (§14.6).
- Resumable vs. abortive operations: decided by the operation's result type, abortive exactly when it is `Never`; no declaration needed (§16.5).
- `Raise`: takes any type, with no error supertype; `Raise(A) | Raise(B)` is `Raise(A | B)`, and `Raise(A) <: Raise(B)` when `A <: B` (§14.7).
- Type parameter syntax: `Name(T_1, ..., T_n)`, a parameter list rather than a tuple type, with no juxtaposition form (§11.4).
- Declaration syntax: `=` binds and `:` gives a type, everywhere. Struct literals and module files are both bodies with one declaration grammar, types can be declared at any level, and declarations are separated by commas (§11.1–11.3, §18.8).
- Scope: no forward references, except to types and functions, and no value may depend on a later value through the functions it calls (§11.5).
- Type ascription in expressions: `(e : T)`, always parenthesized, so the Boolean conditional keeps `c ? a : b` (§11.6, §13.2).
- Types in patterns appear only after `:` (`n: Int`, `{ x: Int = n }`); bare type tests and pattern-level `&` are gone (§13.3).
- Private-module visibility, generalized to all private declarations: visible to the declaring module and its descendants only; packages confined to `ext/package_name`, so subtrees never cross packages; import cycles allowed within a package (§18.1–§18.6).
