; ===== Variables =====

(identifier) @variable

; ===== Keywords =====

; Control flow
[
  (assert_builtin)
  (break_builtin)
  (case_builtin)
  (rethrow_builtin)
  "case"
  "catch"
  "continue"
  "default"
  "do"
  "else"
  "finally"
  "for"
  "if"
  "in"
  "return"
  "switch"
  "throw"
  "try"
  "when"
  "while"
] @keyword.control

; Import / library directives
[
  (part_of_builtin)
  "deferred"
  "export"
  "hide"
  "import"
  "library"
  "part"
  "show"
] @keyword.import

; Async / generator keywords
[
  "async"
  "async*"
  "await"
  "sync*"
  "yield"
] @keyword.coroutine

; Declarations and modifiers
[
  (const_builtin)
  (final_builtin)
  (inferred_type)
  (void_type)
  "abstract"
  "as"
  "base"
  "class"
  "covariant"
  "dynamic"
  "enum"
  "extends"
  "extension"
  "external"
  "factory"
  "Function"
  "get"
  "implements"
  "interface"
  "is"
  "late"
  "mixin"
  "new"
  "on"
  "operator"
  "required"
  "sealed"
  "set"
  "static"
  "typedef"
  "with"
] @keyword

; Special built-in variables
(this) @variable.special
(super) @variable.special

; ===== Types =====

(type_identifier) @type

; Built-in Dart types
((type_identifier) @type.builtin
  (#match? @type.builtin "^(bool|double|Duration|dynamic|Enum|Error|Exception|Function|Future|int|Iterable|Iterator|List|Map|Never|Null|num|Object|Record|RegExp|Runes|Set|StackTrace|Stream|String|Symbol|Type|Uri)$"))

; Class / mixin / enum / extension definitions
(class_definition
  name: (identifier) @type)

(mixin_declaration
  (identifier) @type)

(enum_declaration
  name: (identifier) @type)

(extension_declaration
  name: (identifier) @type)

; Typedef aliases
(type_alias
  (type_identifier) @type)

; Scoped identifiers
(scoped_identifier
  scope: (identifier) @type)

((scoped_identifier
  scope: (identifier) @type
  name: (identifier) @type)
  (#match? @type "^[a-zA-Z]"))

; Capitalized identifiers heuristic
((identifier) @type
  (#match? @type "^_?[A-Z].*[a-z]"))

; ===== Constructors =====

(constructor_signature
  name: (identifier) @constructor)

(factory_constructor_signature
  (identifier) @constructor)

; ===== Functions / Methods =====

(function_signature
  name: (identifier) @function)

(getter_signature
  (identifier) @function)

(setter_signature
  name: (identifier) @function)

; Function calls (heuristic: lowercase identifier before argument list)
(((identifier) @function
  (#match? @function "^_?[a-z]"))
  .
  (selector
    .
    (argument_part))) @function

; Method calls via selectors
((selector
  (unconditional_assignable_selector
    (identifier) @function.method))
  .
  (selector
    (argument_part
      (arguments))))

((selector
  (conditional_assignable_selector
    (identifier) @function.method))
  .
  (selector
    (argument_part
      (arguments))))

(cascade_section
  (cascade_selector
    (identifier) @function.method)
  .
  (argument_part
    (arguments)))

; ===== Annotations =====

(annotation
  "@" @attribute
  name: (identifier) @attribute)

; ===== Properties =====

(unconditional_assignable_selector
  (identifier) @property)

(conditional_assignable_selector
  (identifier) @property)

(cascade_section
  (cascade_selector
    (identifier) @property))

; ===== Enum Constants =====

(enum_constant
  name: (identifier) @constant)

; ===== Parameters =====

(formal_parameter
  name: (identifier) @variable.parameter)

(named_argument
  (label
    (identifier) @variable.parameter))

; ===== Assignments =====

(assignment_expression
  left: (assignable_expression) @variable)

; ===== Template Substitutions & Escapes =====

(template_substitution
  "$" @punctuation.special
  "{" @punctuation.special
  "}" @punctuation.special) @none

(template_substitution
  "$" @punctuation.special
  (identifier_dollar_escaped) @variable) @none

(escape_sequence) @string.escape

; ===== Operators =====

[
  "@"
  "=>"
  ".."
  "??="
  "??"
  "=="
  "?"
  ":"
  "&&"
  "%"
  "<"
  ">"
  "="
  ">="
  "<="
  "||"
  "~/"
  (multiplicative_operator)
  (increment_operator)
  (is_operator)
  (prefix_operator)
  (equality_operator)
  (additive_operator)
] @operator

; Type argument / parameter brackets
(type_arguments
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

(type_parameters
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

; ===== Punctuation =====

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ";"
  "."
  ","
] @punctuation.delimiter

; ===== Literals =====

[
  (hex_integer_literal)
  (decimal_integer_literal)
  (decimal_floating_point_literal)
] @number

(string_literal) @string
(symbol_literal) @string.special.symbol
(true) @boolean
(false) @boolean
(null_literal) @constant.builtin

; ===== Comments =====

(comment) @comment
(documentation_comment) @comment.doc

; ===== Dart 3: Patterns & Switch Expressions =====

(switch_expression_case
  "=>" @operator)

(guard
  "when" @keyword.control)

(record_literal) @none
(record_type) @type
(record_type_named_field
  name: (identifier) @variable.parameter)

(variable_pattern
  (identifier) @variable)

(wildcard_pattern
  "_" @variable.special)

(rest_pattern
  "..." @operator)
