# Tumul

**Tumul** is a minimal, indentation-significant, ML-flavored language featuring:
- Structural records/tuples (no distinction, parentheses syntax)
- First-class union and intersection types
- First-class effect handlers (with punctuation, not keywords)
- Minimal, clean syntax

**Extension:** `.tu`

---

## Getting Started

### Requirements

- Python 3.7+
- No external packages required

### Usage

```bash
python run_tumul.py example.tu
```

---

## File Structure

- `parser.py` – Tumul parser (to AST)
- `interpreter.py` – Interpreter (evaluates AST)
- `run_tumul.py` – Entry point/wrapper
- `example.tu` – Sample Tumul source

---

## Example: `example.tu`

```tumul
get = \_ -> 3
set = \n -> print("set to " + n)

counter_handler = (
  get: \_ -> 0,
  set: \n -> print("counter set to " + n)
)

print_count = \_ ->
  n = get()
  print("The count is " + n)

main = counter_handler ! print_count()
main
```

---

## Language Features

- **Tuples/records:** `(x: 1, y: 2)` or `(1, 2)` (fields optional)
- **Tags:** `'ok`, `'err`
- **Handlers:** `(label: \param -> body, ...)`
- **Handler application:** `handler ! expr`
- **Functions:** `\x -> x + 1`
- **Pattern matching:** `x ? 'ok -> ..., _ -> ...`

---

Created by [@albx79](https://github.com/albx79).  
Experimental and evolving!
