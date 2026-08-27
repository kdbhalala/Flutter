; ===== Functions =====

; Functions and methods with block body
(function_body
  (block
    "{" (_)* @function.inside "}")) @function.around

; Functions and methods with expression body (fat arrow)
(function_body
  "=>"
  (_)* @function.inside) @function.around

; ===== Classes =====

; Class body content
(class_definition
  body: (class_body
    "{" (_)* @class.inside "}")) @class.around

; Mixin body content
(mixin_declaration
  (class_body
    "{" (_)* @class.inside "}")) @class.around

; Enum body content
(enum_declaration
  body: (enum_body
    "{" (_)* @class.inside "}")) @class.around

; Extension body content
(extension_declaration
  body: (extension_body
    "{" (_)* @class.inside "}")) @class.around

; ===== Comments =====

; Adjacent doc comments grouped together
(documentation_comment)+ @comment.around

; Adjacent line comments grouped together
(comment)+ @comment.around
