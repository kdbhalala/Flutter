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

; Cascade chains — indent the chained members
(cascade_section) @indent
