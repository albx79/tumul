import sys
from parser import demo_parse
from interpreter import eval_expr, env

def main():
    if len(sys.argv) != 2:
        print("Usage: python run_tumul.py filename.tm")
        sys.exit(1)
    filename = sys.argv[1]
    with open(filename) as f:
        code = f.read()
    ast = demo_parse(code)
    result = eval_expr(ast, env)
    if result is not None:
        print(result)

if __name__ == "__main__":
    main()
