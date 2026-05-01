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

; Top-level functions
(function_signature
  name: (_) @name) @item

; Getters
(getter_signature
  "get" @context
  name: (_) @name) @item

; Setters
(setter_signature
  "set" @context
  name: (_) @name) @item

; Constructors (named and unnamed)
(constructor_signature
  name: (identifier) @name) @item

(factory_constructor_signature
  "factory" @context
  (identifier) @name) @item

; Static fields / constants
(static_final_declaration
  (identifier) @name) @item

(initialized_identifier
  (identifier) @name) @item
