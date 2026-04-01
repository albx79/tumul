# Tumul Parser (minimal prototype)

import re

# AST Nodes as in interpreter
class Num:      # number literal
    def __init__(self, value): self.value = value
    def __repr__(self): return f"Num({self.value})"
class Str:      # string literal
    def __init__(self, value): self.value = value
    def __repr__(self): return f"Str({self.value!r})"
class Tag:      # tag literal
    def __init__(self, name): self.name = name
    def __repr__(self): return f"Tag('{self.name}')"
class Var:
    def __init__(self, name): self.name = name
    def __repr__(self): return f"Var({self.name})"
class ListVal:  # enum/list literal
    def __init__(self, elems): self.elems = elems
    def __repr__(self): return f"ListVal({self.elems})"
class TupleVal: # tuple/record
    def __init__(self, fields): self.fields = fields  # list of (label, val) or just [(None, val),...]
    def __repr__(self): return f"TupleVal({self.fields})"
class Lambda:
    def __init__(self, param, body): self.param = param; self.body = body
    def __repr__(self): return f"Lambda({self.param},{self.body})"
class App:
    def __init__(self, fn, arg): self.fn = fn; self.arg = arg
    def __repr__(self): return f"App({self.fn},{self.arg})"
class HandlerVal:
    def __init__(self, entries): self.entries = entries  # dict label: Lambda
    def __repr__(self): return "HandlerVal{" + ", ".join(f"{k}: {v}" for k,v in self.entries.items()) + "}"
class HandlerApp:
    def __init__(self, handler, expr): self.handler = handler; self.expr = expr
    def __repr__(self): return f"HandlerApp({self.handler}, {self.expr})"
class Def:
    def __init__(self, name, expr): self.name = name; self.expr = expr
    def __repr__(self): return f"Def({self.name} = {self.expr})"
class Match:
    def __init__(self, expr, cases): self.expr = expr; self.cases = cases  # [(pattern, expr)]
    def __repr__(self): return f"Match({self.expr}, {self.cases})"
class Block:
    def __init__(self, exprs): self.exprs = exprs
    def __repr__(self): return f"Block({self.exprs})"
class Pattern:
    def __init__(self, val): self.val = val
    def __repr__(self): return f"Pattern({self.val})"

TOKEN_SPEC = [
    ('NUM',      r'\d+(\.\d*)?'),
    ('STRING',   r'"([^"\\]|\\.)*"'),
    ('TAG',      r"'[a-z_][a-zA-Z0-9_]*"),
    ('DEF',      r'::|='),
    ('ARROW',    r'->'),
    ('LAMBDA',   r'\\'),
    ('MATCH',    r'\?'),
    ('BANG',     r'\!'),
    ('LPAREN',   r'\('),
    ('RPAREN',   r'\)'),
    ('LBRACK',   r'\['),
    ('RBRACK',   r'\]'),
    ('COMMA',    r','),
    ('COLON',    r':'),
    ('ID',       r'[a-z_][a-zA-Z0-9_]*'),
    ('NEWLINE',  r'\n'),
    ('SKIP',     r'[ \t]+'),
    ('MISMATCH', r'.')
]

MASTER_RE = re.compile('|'.join(f'(?P<{name}>{r})' for name,r in TOKEN_SPEC))

def tokenize(code):
    line = 1
    for mo in MASTER_RE.finditer(code):
        kind = mo.lastgroup
        value = mo.group() 
        if kind == 'NEWLINE':
            yield ('NEWLINE', value)
        elif kind == 'SKIP':
            continue
        elif kind == 'MISMATCH':
            raise SyntaxError(f'Unexpected: {value!r}')
        else:
            yield (kind, value)
    yield ('EOF', '')

class Parser:
    def __init__(self, tokens):
        self.tokens = list(tokens)
        self.pos = 0

    def peek(self, k=0):
        if self.pos+k < len(self.tokens): return self.tokens[self.pos+k]
        return ('EOF', '')

    def next(self):
        t = self.peek()
        self.pos += 1
        return t

    def expect(self, kind):
        tok = self.next()
        if tok[0] != kind: raise SyntaxError(f"Expected {kind}, got {tok}")
        return tok

    def at(self, kind, value=None):
        tok = self.peek()
        return tok[0] == kind and (value is None or tok[1] == value)

    def parse(self):
        stmts = []
        while not self.at('EOF'):
            if self.at('NEWLINE'): self.next(); continue
            stmts.append(self.statement())
        return Block(stmts)
        
    def statement(self):
        if self.at('ID'):
            if self.peek(1)[0] == 'DEF':
                name = self.next()[1]
                if self.at('DEF', '='):
                    self.next()
                    rhs = self.expr()
                    return Def(name, rhs)
                elif self.at('DEF', '::'):
                    self.next()
                    while not self.at('NEWLINE',) and not self.at('EOF'):
                        self.next()
                    return None
        return self.expr()
    
    def expr(self):
        return self.handlerapp()
    
    def handlerapp(self):
        expr = self.match()
        while self.at('BANG'):
            self.next()
            handler = expr
            arg = self.match()
            expr = HandlerApp(handler, arg)
        return expr

    def match(self):
        expr = self.application()
        if self.at('MATCH'):
            self.next()
            cases = []
            if self.at('LPAREN'):
                self.next()
                while not self.at('RPAREN'):
                    pat, _ = self.pattern()
                    self.expect('ARROW')
                    branch = self.expr()
                    cases.append((pat, branch))
                    if self.at('COMMA'): self.next()
                self.expect('RPAREN')
            else:
                while True:
                    if self.at('NEWLINE'): self.next(); continue
                    if self.at('EOF') or self.at('ID') or self.at('DEF'): break
                    pat, _ = self.pattern()
                    self.expect('ARROW')
                    branch = self.expr()
                    cases.append((pat, branch))
                    if self.at('NEWLINE'): self.next()
                    if self.at('EOF'): break
            return Match(expr, cases)
        return expr

    def application(self):
        expr = self.atom()
        while self.at('LPAREN'):
            self.next()
            args = []
            if not self.at('RPAREN'):
                args.append(self.expr())
                while self.at('COMMA'): 
                    self.next(); args.append(self.expr())
            self.expect('RPAREN')
            expr = App(expr, args[0] if len(args) == 1 else TupleVal([(None, a) for a in args]))
        return expr

    def atom(self):
        if self.at('NUM'):
            v = float(self.next()[1])
            if v.is_integer(): v = int(v)
            return Num(v)
        if self.at('STRING'):
            val = self.next()[1]
            val = bytes(val[1:-1], "utf-8").decode("unicode_escape")
            return Str(val)
        if self.at('TAG'):
            return Tag(self.next()[1][1:])
        if self.at('ID'):
            return Var(self.next()[1])
        if self.at('LAMBDA'):
            self.next()
            param, _ = self.pattern()
            self.expect('ARROW')
            body = self.expr()
            return Lambda(param, body)
        if self.at('LPAREN'):
            self.next()
            if self.at('RPAREN'):
                self.next()
                return TupleVal([])
            entries = []
            while True:
                if self.at('TAG'):
                    lbl = self.next()[1][1:]
                    self.expect('COLON')
                    val = self.expr()
                    entries.append((lbl, val))
                elif self.at('ID'):
                    lbl = self.next()[1]
                    if self.at('COLON'):
                        self.next()
                        val = self.expr()
                        entries.append((lbl, val))
                    else:
                        val = Var(lbl)
                        entries.append((None, val))
                elif self.at('LAMBDA'):
                    val = self.expr()
                    entries.append((None, val))
                else:
                    val = self.expr()
                    entries.append((None, val))
                if self.at('COMMA'): self.next()
                else: break
            self.expect('RPAREN')
            if all(lbl and isinstance(val, Lambda) for lbl, val in entries):
                return HandlerVal(dict(entries))
            return TupleVal(entries)
        if self.at('LBRACK'):
            self.next()
            elems = []
            if not self.at('RBRACK'):
                elems.append(self.expr())
                while self.at('COMMA'):
                    self.next()
                    elems.append(self.expr())
            self.expect('RBRACK')
            return ListVal(elems)
        raise SyntaxError(f"Expected expression, got {self.peek()}")

    def pattern(self):
        if self.at('TAG'):
            return Pattern(Tag(self.next()[1][1:])), None
        if self.at('ID'):
            return Pattern(Var(self.next()[1])), None
        if self.at('NUM'):
            return Pattern(Num(int(self.next()[1]))), None
        if self.at('STRING'):
            return Pattern(Str(self.next()[1][1:-1])), None
        if self.at('LPAREN'):
            self.next()
            pats = []
            if not self.at('RPAREN'):
                pats.append(self.pattern()[0])
                while self.at('COMMA'):
                    self.next()
                    pats.append(self.pattern()[0])
            self.expect('RPAREN')
            return Pattern(TupleVal([(None, p) for p in pats])), None
        if self.at('ID', '_'):
            self.next()
            return Pattern("_"), None
        raise SyntaxError(f"Invalid pattern: {self.peek()}")

def demo_parse(code):
    tokens = list(tokenize(code))
    parser = Parser(tokens)
    ast = parser.parse()
    return ast

if __name__ == "__main__":
    code = """
status = 'ok
counter_handler = (get: \\_ -> 0, set: \\n -> print("set " + n))
main = counter_handler ! status
main
"""
    ast = demo_parse(code)
    from pprint import pprint
    pprint(ast)
