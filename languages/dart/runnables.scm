; Detect top-level main() function as runnable Dart program
(
  (function_signature
    name: (identifier) @run
    (#eq? @run "main"))
  (#set! tag dart-main)
)

; Detect test('name', ...) calls as runnable tests
(
  (expression_statement
    (identifier) @_fn
    (#eq? @_fn "test")
    .
    (selector
      (argument_part
        (arguments
          (argument
            (string_literal) @run)))))
  (#set! tag dart-test)
)

; Detect testWidgets('name', ...) calls as runnable Flutter widget tests
(
  (expression_statement
    (identifier) @_fn
    (#eq? @_fn "testWidgets")
    .
    (selector
      (argument_part
        (arguments
          (argument
            (string_literal) @run)))))
  (#set! tag flutter-test)
)

; Detect blocTest('name', ...) calls from the bloc_test package
(
  (expression_statement
    (identifier) @_fn
    (#eq? @_fn "blocTest")
    .
    (selector
      (argument_part
        (arguments
          (argument
            (string_literal) @run)))))
  (#set! tag dart-test)
)

; Detect group('name', ...) calls — runnable test group
(
  (expression_statement
    (identifier) @_fn
    (#eq? @_fn "group")
    .
    (selector
      (argument_part
        (arguments
          (argument
            (string_literal) @run)))))
  (#set! tag dart-test-group)
)
