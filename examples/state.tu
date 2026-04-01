# Example Tumul program: state counting

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
