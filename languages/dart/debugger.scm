; Formal parameter declarations (standard, optional, named, constructor/super initializers)
(formal_parameter
  (identifier) @debug-variable)

(constructor_param
  (identifier) @debug-variable)

(super_formal_parameter
  (identifier) @debug-variable)

; Variable declarations (local variables, fields, top-level, and static variables)
(initialized_variable_definition
  name: (identifier) @debug-variable)

(initialized_identifier
  (identifier) @debug-variable)

(static_final_declaration
  (identifier) @debug-variable)

(identifier_list
  (identifier) @debug-variable)

; Special variable `this`
(this) @debug-variable

; For and for-in loop variables
(for_loop_parts
  name: (identifier) @debug-variable)

(for_loop_parts
  (identifier) @debug-variable)

; Catch clause exception and stack trace parameters
(catch_parameters
  (identifier) @debug-variable)

; Pattern variables (Dart 3 pattern matching)
(variable_pattern
  (identifier) @debug-variable)

; Assignment expressions (left-hand side targets)
(assignment_expression
  left: (assignable_expression
    (identifier) @debug-variable))

(assignment_expression_without_cascade
  left: (assignable_expression
    (identifier) @debug-variable))

(pattern_assignment
  (identifier) @debug-variable)

; Variables in expressions (inline evaluation and inspect-on-hover)
(return_statement
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(parenthesized_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(argument
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(named_argument
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(conditional_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(if_null_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(unconditional_assignable_selector
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(conditional_assignable_selector
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(cascade_selector
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(additive_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(multiplicative_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(relational_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(equality_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(logical_and_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(logical_or_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(shift_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(bitwise_and_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(bitwise_or_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(bitwise_xor_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(unary_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(await_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(type_cast_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

(type_test_expression
  (identifier) @debug-variable
  (#not-match? @debug-variable "^[A-Z]"))

; String interpolation variables
(template_substitution
  (identifier_dollar_escaped) @debug-variable)

; Debug scopes
(block) @debug-scope
(function_body) @debug-scope
(program) @debug-scope
