/**
 * @file HubGS grammar for tree-sitter
 * @author Jarbear82
 * @author Gemini CLI
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const OP_PREC = {
  ARROW: 0,
  LOGICAL: 1,
  EQUALITY: 2,
  RELATIONAL: 3,
  ADD: 4,
  MULTIPLY: 5,
  UNARY: 6,
  MEMBER: 7,
};

module.exports = grammar({
  name: "hubgs",

  extras: ($) => [/\s/, $.comment],

  supertypes: ($) => [$._expression],
  word: ($) => $.identifier,

  rules: {
    // ------------------------------------------------------------------------
    // Top-Level Structure (Strict Ordering: IMPORTS -> DEFINITIONS -> INSTANCES)
    // ------------------------------------------------------------------------

    source_file: ($) =>
      seq(
        optional(seq($.imports_section, optional(","))),
        optional(seq($.definitions_section, optional(","))),
        optional($.instances_section),
      ),

    // ------------------------------------------------------------------------
    // Imports
    // ------------------------------------------------------------------------

    imports_section: ($) =>
      seq("IMPORTS", "[", commaSep($.import_statement), "]"),

    import_statement: ($) =>
      seq("[", commaSep1($.identifier), "]", "FROM", $.string),

    // ------------------------------------------------------------------------
    // Definitions
    // ------------------------------------------------------------------------

    definitions_section: ($) =>
      seq(
        "DEFINITIONS",
        "[",
        commaSep(
          choice($.fields_block, $.enums_block, $.structs_block, $.hubs_block),
        ),
        "]",
      ),

    fields_block: ($) => seq("FIELDS", "[", commaSep($.field_definition), "]"),

    field_definition: ($) => seq($.identifier, ":", $.type),

    enums_block: ($) => seq("ENUMS", "[", commaSep($.enum_definition), "]"),

    enum_definition: ($) => seq($.identifier, "{", commaSep($.identifier), "}"),

    structs_block: ($) =>
      seq("STRUCTS", "[", commaSep($.struct_definition), "]"),

    struct_definition: ($) =>
      seq($.identifier, "{", commaSep($.identifier), "}"),

    hubs_block: ($) => seq("HUBS", "[", commaSep($.hub_definition), "]"),

    extension_clause: ($) => seq("EXTENDS", "[", commaSep1($.identifier), "]"),

    hub_definition: ($) =>
      seq(
        $.identifier,
        optional($._extension),
        "{",
        commaSep(choice($.hub_field, $.hub_role, $.constraints_block)),
        "}",
      ),

    _extension: ($) => $.extension_clause,

    hub_field: ($) =>
      seq(
        $.identifier,
        optional(seq(optional("="), $.decorator)),
        repeat($.field_attribute),
      ),

    field_attribute: ($) => seq("@", choice("display", "background")),

    constraints_block: ($) =>
      seq("@constraints", "[", commaSep($._expression), "]"),

    hub_role: ($) =>
      seq(
        $.identifier,
        $.role_direction,
        "(",
        $.multiplicity,
        ")",
        "ALLOWS",
        "[",
        commaSep1($.identifier),
        "]",
      ),

    role_direction: (_) => choice("->", "<-", "<->", "-"),

    multiplicity: ($) =>
      choice($.number, "*", seq($.number, "..", choice($.number, "*"))),

    // ------------------------------------------------------------------------
    // Instances
    // ------------------------------------------------------------------------

    instances_section: ($) =>
      seq("INSTANCES", "[", commaSep($.instance_block), "]"),

    instance_block: ($) =>
      seq(
        field("ref", choice($.identifier, $.uuid)),
        ":",
        field("type", $.identifier),
        "{",
        commaSep($.instance_assignment),
        "}",
      ),

    instance_assignment: ($) => seq($.identifier, "=", $._expression),

    uuid: (_) => /[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}/,



    // ------------------------------------------------------------------------
    // Types & Decorators
    // ------------------------------------------------------------------------

    type: ($) => choice($.identifier, $.generic_type),

    generic_type: ($) => seq($.identifier, "<", commaSep1($.type), ">"),

    decorator: ($) =>
      seq(choice("@computed", "@default"), "(", $._expression, ")"),

    // ------------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------------

    _expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.member_expression,
        $.call_expression,
        $.arrow_function,
        $.identifier,
        $.uuid,
        $.number,
        $.string,
        $.template_string,
        $.array,
        $.boolean,
        $.parenthesized_expression,
      ),

    parenthesized_expression: ($) => seq("(", $._expression, ")"),

    array: ($) => seq("[", commaSep($._expression), "]"),

    member_expression: ($) =>
      prec(
        OP_PREC.MEMBER,
        seq(
          field("object", $._expression),
          ".",
          field("property", $.identifier),
        ),
      ),

    call_expression: ($) =>
      prec(
        OP_PREC.MEMBER,
        seq(
          field("function", $._expression),
          "(",
          commaSep($._expression),
          ")",
        ),
      ),

    arrow_function: ($) =>
      prec(
        OP_PREC.ARROW,
        seq(
          field("parameter", $.identifier),
          "=>",
          field("body", $._expression),
        ),
      ),

    unary_expression: ($) =>
      prec(
        OP_PREC.UNARY,
        seq(
          field("operator", choice("!", "-")),
          field("argument", $._expression),
        ),
      ),

    binary_expression: ($) =>
      choice(
        prec.left(
          OP_PREC.LOGICAL,
          seq(
            field("left", $._expression),
            field("operator", choice("&&", "||")),
            field("right", $._expression),
          ),
        ),
        prec.left(
          OP_PREC.EQUALITY,
          seq(
            field("left", $._expression),
            field("operator", choice("==", "!=")),
            field("right", $._expression),
          ),
        ),
        prec.left(
          OP_PREC.RELATIONAL,
          seq(
            field("left", $._expression),
            field("operator", choice("<", ">", "<=", ">=")),
            field("right", $._expression),
          ),
        ),
        prec.left(
          OP_PREC.ADD,
          seq(
            field("left", $._expression),
            field("operator", choice("+", "-")),
            field("right", $._expression),
          ),
        ),
        prec.left(
          OP_PREC.MULTIPLY,
          seq(
            field("left", $._expression),
            field("operator", choice("*", "/")),
            field("right", $._expression),
          ),
        ),
      ),

    // ------------------------------------------------------------------------
    // Literals & Primitives
    // ------------------------------------------------------------------------

    boolean: (_) => choice("true", "false"),

    // Supports English + international languages. Excludes math/syntax symbols.
    identifier: (_) => /[a-zA-Z_\u00A1-\u10FFFF][a-zA-Z0-9_\u00A1-\u10FFFF]*/,

    number: (_) => {
      const hex = /0[xX][0-9a-fA-F]+/;
      const binary = /0[bB][01]+/;
      const octal = /0[oO][0-7]+/;
      const decimal = /-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?/;
      return token(choice(hex, binary, octal, decimal));
    },

    string: ($) =>
      choice(
        // Standard single-line strings (newlines explicitly forbidden)
        seq('"', repeat(choice(/[^"\\\n]+/, $.escape_sequence)), '"'),
        seq("'", repeat(choice(/[^'\\\n]+/, $.escape_sequence)), "'"),

        // Multi-line triple-quoted strings (newlines naturally permitted)
        // The regex allows single or double quotes inside as long as they aren't three in a row
        seq(
          '"""',
          repeat(choice(/[^"\\]+/, /"[^"\\]/, /""[^"\\]/, $.escape_sequence)),
          '"""',
        ),
        seq(
          "'''",
          repeat(choice(/[^'\\]+/, /'[^'\\]/, /''[^'\\]/, $.escape_sequence)),
          "'''",
        ),
      ),

    template_string: ($) =>
      seq(
        "`",
        repeat(
          choice(
            /[^`$\\]+/,
            $.escape_sequence,
            seq("$", "{", $._expression, "}"),
          ),
        ),
        "`",
      ),

    escape_sequence: (_) =>
      token.immediate(seq("\\", /(\"|\'|\`|\\|\/|b|f|n|r|t|u[0-9a-fA-F]{4})/)),

    comment: (_) =>
      token(
        choice(seq("//", /.*/), seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")),
      ),
  },
});

/**
 * Creates a rule to match one or more of the rules separated by a comma.
 * Explicitly allows an optional trailing comma.
 *
 * @param {RuleOrLiteral} rule
 * @returns {SeqRule}
 */
function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)), optional(","));
}

/**
 * Creates a rule to optionally match one or more of the rules separated by a comma.
 *
 * @param {RuleOrLiteral} rule
 * @returns {ChoiceRule}
 */
function commaSep(rule) {
  return optional(commaSep1(rule));
}
