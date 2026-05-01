; String scopes — disable some bracket auto-closing inside strings
[
  (string_literal)
] @string

; Comment scopes — inclusive so the scope extends to end-of-line for line comments
(comment) @comment.inclusive
(documentation_comment) @comment.inclusive
