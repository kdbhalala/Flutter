; Classes
(class_definition
  "class" @context
  name: (_) @name) @item

; Mixins
(mixin_declaration
  "mixin" @context
  name: (_) @name) @item

; Enums
(enum_declaration
  "enum" @context
  name: (_) @name) @item

; Enum constants (members inside enums)
(enum_constant
  name: (identifier) @name) @item

; Extensions
(extension_declaration
  "extension" @context
  name: (_) @name) @item

; Typedefs
(type_alias
  "typedef" @context
  (type_identifier) @name) @item

; Top-level functions and class methods (function_signature matches both levels)
(function_signature
  name: (_) @name) @item

; Getters (top-level and class-level)
(getter_signature
  "get" @context
  name: (_) @name) @item

; Setters (top-level and class-level)
(setter_signature
  "set" @context
  name: (_) @name) @item

; Constructors (named and unnamed)
(constructor_signature
  name: (identifier) @name) @item

(factory_constructor_signature
  "factory" @context
  (identifier) @name) @item

; Operator overloads (binary: ==, +, -, *, /, <, >, etc.)
(operator_signature
  "operator" @context
  (binary_operator) @name) @item

; Static fields / constants
(static_final_declaration
  (identifier) @name) @item
