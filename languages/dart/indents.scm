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
