; Inject markdown into documentation comments for rich rendering
((documentation_comment) @injection.content
  (#set! injection.language "markdown")
  (#set! injection.include-children))
