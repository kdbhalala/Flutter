; Block-level indentation for bracket pairs
(_
  "["
  "]" @end) @indent

(_
  "{"
  "}" @end) @indent

(_
  "("
  ")" @end) @indent

; Switch expressions and statements
(switch_statement
  "{"
  "}" @end) @indent

(switch_expression
  "{"
  "}" @end) @indent

; Cascade chains — indent the chained members
(cascade_section) @indent

; Class, mixin, enum, extension bodies
(class_body) @indent
(mixin_declaration
  (class_body) @indent)
(enum_declaration
  "{" "}" @end) @indent
(extension_declaration
  (class_body) @indent)

; Function / method bodies
(function_body
  (block) @indent)

; If / else if continuation — dedent on else
(if_statement
  consequence: (block) @indent)

; For, while, do-while single-body indent
(for_statement
  body: (block) @indent)
(while_statement
  body: (block) @indent)
(do_statement
  body: (block) @indent)

; Try / catch / finally
(try_statement
  body: (block) @indent)
(catch_clause
  body: (block) @indent)
(finally_clause
  (block) @indent)

; List / map / set literals
(list_literal
  "[" "]" @end) @indent
(map_literal
  "{" "}" @end) @indent
(set_or_map_literal
  "{" "}" @end) @indent

; Named argument lists and argument lists
(argument_part
  (arguments
    "(" ")" @end) @indent)
