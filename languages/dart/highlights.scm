; ===== Variables =====

(identifier) @variable

; ===== Keywords =====

; Control flow keywords
[
  (assert_builtin)
  (break_builtin)
  (rethrow_builtin)
  "break"
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
  "deferred"
  "export"
  "hide"
  "import"
  "library"
  "part"
  (part_of_builtin)
  "show"
] @keyword.import

; Async / generator keywords
[
  "async"
  "async*"
  "await"
  "sync*"
  "yield"
] @keyword

; Declaration and modifier keywords
[
  "abstract"
  "as"
  "base"
  "class"
  (const_builtin)
  "covariant"
  "enum"
  "extends"
  "extension"
  "external"
  "factory"
  "final"
  "Function"
  "get"
  "implements"
  (inferred_type)
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
  "super"
  "this"
  "typedef"
  "var"
  (void_type)
  "with"
] @keyword

; "this" and "super" are special built-in variables, not plain keywords
(this) @variable.special
(super) @variable.special

; ===== Types =====

(type_identifier) @type

; Built-in Dart types
((type_identifier) @type.builtin
  (#match? @type.builtin "^(bool|double|Duration|dynamic|Enum|Error|Exception|Function|Future|int|Iterable|Iterator|List|Map|Never|Null|num|Object|Record|RegExp|Runes|Set|StackTrace|Stream|String|Symbol|Type|Uri)$"))

; Class / mixin / enum / extension names
(class_definition
  name: (identifier) @type)

(mixin_declaration
  name: (identifier) @type)

(enum_declaration
  name: (identifier) @type)

(extension_declaration
  name: (identifier) @type)

; Typedef targets
(type_alias
  (type_identifier) @type)

; Scoped identifier scope is usually a type (e.g. MyClass.method)
(scoped_identifier
  scope: (identifier) @type)

((scoped_identifier
  scope: (identifier) @type
  name: (identifier) @type)
  (#match? @type "^[a-zA-Z]"))

; Capitalized identifiers are typically types
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

; Function calls (heuristic: lowercase identifier immediately before argument list)
(((identifier) @function
  (#match? @function "^_?[a-z]"))
  .
  (selector
    .
    (argument_part))) @function

; Method calls via chained selectors
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

((unconditional_assignable_selector
  (identifier) @function.method)
  .
  (selector
    (argument_part
      (arguments))))

((conditional_assignable_selector
  (identifier) @function.method)
  .
  (selector
    (argument_part
      (arguments))))

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

; ===== Enum Members =====

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

; ===== Operators =====

(template_substitution
  "$" @punctuation.special
  "{" @punctuation.special
  "}" @punctuation.special) @none

(template_substitution
  "$" @punctuation.special
  (identifier_dollar_escaped) @variable) @none

(escape_sequence) @string.escape

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

; Type argument / parameter angle brackets are punctuation
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

; Keywords - definitions
[
  (case_builtin)
  (void_type)
  "late"
  "required"
  "extension"
  "on"
  "class"
  "enum"
  "extends"
  "in"
  "is"
  "new"
  "super"
  "with"
  "Function"
] @keyword.definition

"return" @keyword.return

[
  (part_of_builtin)
  "deferred"
  "factory"
  "get"
  "implements"
  "interface"
  "library"
  "operator"
  "mixin"
  "part"
  "set"
  "typedef"
] @keyword

[
  "async"
  "async*"
  "sync*"
  "await"
  "yield"
] @keyword.coroutine

[
  (const_builtin)
  (final_builtin)
  "abstract"
  "covariant"
  "dynamic"
  "external"
  "static"
  "final"
  "base"
  "sealed"
] @keyword.modifier

((identifier) @variable.builtin
  (#any-of? @variable.builtin
    "abstract" "as" "covariant" "deferred" "dynamic" "export" "external" "factory" "Function" "get"
    "implements" "import" "interface" "library" "operator" "mixin" "part" "set" "static" "typedef"))

[
  "if"
  "else"
  "switch"
  "default"
  "case"
] @keyword.conditional

[
  "try"
  "throw"
  "catch"
  "finally"
  (break_statement)
] @keyword.exception

[
  "do"
  "while"
  "continue"
  "for"
] @keyword.repeat
