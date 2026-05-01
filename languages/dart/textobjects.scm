; ===== Functions =====

; Top-level function with block body — inside is the block content
(function_signature
  (function_body
    (block
      "{" (_)* @function.inside "}"))) @function.around

; Top-level function with expression body (fat arrow)
(function_signature
  (function_body) @function.inside) @function.around

; Method signatures (includes constructors, getters, setters, operators)
(method_signature
  (function_signature
    (function_body
      (block
        "{" (_)* @function.inside "}"))))  @function.around

(method_signature
  (getter_signature
    (function_body
      (block
        "{" (_)* @function.inside "}")))) @function.around

(method_signature
  (setter_signature
    (function_body
      (block
        "{" (_)* @function.inside "}")))) @function.around

; Local function declarations
(local_function_declaration
  (function_signature
    (function_body
      (block
        "{" (_)* @function.inside "}")))) @function.around

; ===== Classes =====

; Class body content
(class_definition
  body: (class_body
    "{" (_)* @class.inside "}")) @class.around

; Mixin body content
(mixin_declaration
  body: (class_body
    "{" (_)* @class.inside "}")) @class.around

; Enum body content
(enum_declaration
  "{" (_)* @class.inside "}") @class.around

; Extension body content
(extension_declaration
  (class_body
    "{" (_)* @class.inside "}")) @class.around

; ===== Comments =====

; Adjacent doc comments grouped together
(documentation_comment)+ @comment.around

; Adjacent line comments grouped together
(comment)+ @comment.around
