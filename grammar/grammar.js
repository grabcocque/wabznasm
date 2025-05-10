// See https://tree-sitter.github.io/tree-sitter/creating-parsers
// Precedence levels (higher number binds tighter)
const PREC = {
  ASSIGN: 0, // : assignment
  ADD: 1, // + -
  MUL: 2, // * / %
  EXP: 3, // ^ (right-assoc)
  UNARY: 4, // prefix -
  FACT: 5, // postfix !
  CALL: 6, // function calls
};

module.exports = grammar({
  name: "calc",
  extras: ($) => [/[\s\t\n\r]+/],

  rules: {
    // The top-level entry point - simplified to avoid conflicts
    source_file: ($) => $.statement,

    // Statement can be assignment or expression
    statement: ($) => choice(
      $.assignment,
      $.expression
    ),

    // Function assignment: name: {body} or name: expression
    assignment: ($) => prec.right(PREC.ASSIGN, seq(
      field("name", $.identifier),
      field("operator", ":"),
      field("value", choice($.function_body, $.expression))
    )),

    // Function body: {expression} or {[params] expression}
    function_body: ($) => seq(
      field("left_brace", "{"),
      optional(field("params", $.parameter_list)),
      field("body", $.expression),
      field("right_brace", "}")
    ),

    // Parameter list: [x;y;z]
    parameter_list: ($) => seq(
      field("left_bracket", "["),
      optional(seq(
        field("param", $.identifier),
        repeat(seq(field("separator", ";"), field("param", $.identifier)))
      )),
      field("right_bracket", "]")
    ),

    // Expressions with all operators, layered by precedence
    expression: ($) => $.additive,

    // Lowest precedence: addition and subtraction (left-assoc)
    additive: ($) =>
      choice(
        // a + b
        prec.left(
          PREC.ADD,
          seq(
            field("left", $.additive),
            field("operator", "+"),
            field("right", $.multiplicative)
          )
        ),
        // a - b
        prec.left(
          PREC.ADD,
          seq(
            field("left", $.additive),
            field("operator", "-"),
            field("right", $.multiplicative)
          )
        ),
        // fallback
        $.multiplicative
      ),

    // Multiplication and division (left-assoc)
    multiplicative: ($) =>
      choice(
        // a * b
        prec.left(
          PREC.MUL,
          seq(
            field("left", $.multiplicative),
            field("operator", "*"),
            field("right", $.unary)
          )
        ),
        // a / b
        prec.left(
          PREC.MUL,
          seq(
            field("left", $.multiplicative),
            field("operator", "/"),
            field("right", $.unary)
          )
        ),
        // a % b
        prec.left(
          PREC.MUL,
          seq(
            field("left", $.multiplicative),
            field("operator", "%"),
            field("right", $.unary)
          )
        ),
        // fallback
        $.unary
      ),

    // Prefix unary operators (right-assoc)
    unary: ($) =>
      choice(
        // -x
        prec.right(PREC.UNARY, seq(field("operator", "-"), field("operand", $.unary))),
        // fallback
        $.power
      ),

    // Exponentiation (right-assoc)
    power: ($) =>
      choice(
        // a ^ b
        prec.right(
          PREC.EXP,
          seq(
            field("base", $.postfix),
            field("operator", "^"),
            // Allow unary expressions (including negative) in exponent
            field("exponent", $.unary)
          )
        ),
        // fallback
        $.postfix
      ),

    // Postfix operators (left-assoc for chaining)
    postfix: ($) =>
      choice(
        // x!
        prec.left(PREC.FACT, seq(field("operand", $.postfix), field("operator", "!"))),
        // fallback
        $.primary
      ),

    // Atoms: numbers, identifiers, function calls, and parenthesized expressions
    primary: ($) =>
      choice(
        // function call with arguments: f[x;y]
        $.function_call,
        // identifier/variable reference
        $.identifier,
        // literal
        $.number,
        // (expression)
        seq(
          field("left_paren", "("),
          field("expression", $.expression),
          field("right_paren", ")")
        )
      ),

    // Function call: name[args] or name[]
    function_call: ($) => prec(PREC.CALL, seq(
      field("function", $.identifier),
      field("left_bracket", "["),
      optional(field("args", $.argument_list)),
      field("right_bracket", "]")
    )),

    // Argument list: expr;expr;expr
    argument_list: ($) => seq(
      field("arg", $.expression),
      repeat(seq(field("separator", ";"), field("arg", $.expression)))
    ),

    // Identifier (variable/function names)
    identifier: () => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Integer literals
    number: () => /\d+/,
  },
});
