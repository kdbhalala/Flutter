("(" @open
  ")" @close)

("[" @open
  "]" @close)

("{" @open
  "}" @close)

; Angle brackets are used for generics but also as comparison operators,
; so opt out of rainbow coloring to avoid false positives.
(("<" @open
  ">" @close)
  (#set! rainbow.exclude))

("\"" @open
  "\"" @close)

("'" @open
  "'" @close)
