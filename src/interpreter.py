# Tumul Interpreter (core, minimal)

import re
from collections import namedtuple

class Num: 
    def __init__(self, value): self.value = value
class Str: 
    def __init__(self, value): self.value = value
class Tag: 
    def __init__(self, name): self.name = name
class Var: 
    def __init__(self, name): self.name = name
class ListVal: 
    def __init__(self, elems): self.elems = elems
class TupleVal: 
    def __init__(self, fields): self.fields = fields
class Lambda: 
    def __init__(self, param, body): self.param = param; self.body = body
class App: 
    def __init__(self, fn, arg): self.fn = fn; self.arg = arg
class HandlerApp: 
    def __init__(self, handler, expr): self.handler = handler; self.expr = expr
class Def: 
    def __init__(self, name, expr): self.name = name; self.expr = expr
class Block: 
    def __init__(self, exprs): self.exprs = exprs
class Match: 
    def __init__(self, expr, cases): self.expr = expr; self.cases = cases
class HandlerVal:  
    def __init__(self, entries): self.entries = entries

class Env(dict):
    def extend(self):
        e = Env(self)
        return e

def eval_expr(expr, env, handlers=None):
    if handlers is None: handlers = []
    if isinstance(expr, Num):     return expr.value
    if isinstance(expr, Str):     return expr.value
    if isinstance(expr, Tag):     return expr
    if isinstance(expr, Var):
        if expr.name in env:  return env[expr.name]
        else: raise Exception(f"undefined var {expr.name}")
    if isinstance(expr, ListVal): return [eval_expr(e, env, handlers) for e in expr.elems]
    if isinstance(expr, TupleVal):
        return {lbl: eval_expr(val, env, handlers) for (lbl, val) in expr.fields}
    if isinstance(expr, Lambda):  return expr
    if isinstance(expr, App):
        fn = eval_expr(expr.fn, env, handlers)
        arg = eval_expr(expr.arg, env, handlers)
        if isinstance(fn, Lambda):
            newenv = env.extend(); newenv[fn.param] = arg
            return eval_expr(fn.body, newenv, handlers)
        else:
            raise Exception("trying to apply something not a lambda")
    if isinstance(expr, HandlerVal): return expr
    if isinstance(expr, HandlerApp):
        handler_val = eval_expr(expr.handler, env, handlers)
        handlers = [handler_val] + handlers
        return eval_expr(expr.expr, env, handlers)
    if isinstance(expr, Def):
        env[expr.name] = eval_expr(expr.expr, env, handlers)
        return None
    if isinstance(expr, Block):
        result = None
        for e in expr.exprs: result = eval_expr(e, env, handlers)
        return result
    if isinstance(expr, Match):
        val = eval_expr(expr.expr, env, handlers)
        for pat, body in expr.cases:
            if match_val(pat, val):
                return eval_expr(body, env, handlers)
        raise Exception(f"no match for {val} in {expr}")
    raise Exception(f"unknown expr: {expr}")

def match_val(pat, val):
    if isinstance(pat, Tag) and isinstance(val, Tag): return pat.name == val.name
    if isinstance(pat, Num) and isinstance(val, (int, float)): return pat.value == val
    if isinstance(pat, Str) and isinstance(val, str): return pat.value == val
    if pat == "_": return True
    return False

# Built-in environment
env = Env()
env['print'] = print
