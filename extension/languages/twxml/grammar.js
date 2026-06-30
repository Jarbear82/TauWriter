/**
 * @file TWXML grammar for tree-sitter
 * Strict structural enforcement: <document><meta /><body>...</body></document>
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "twxml",
  extras: ($) => [/\s+/, $.comment],

  inline: ($) => [$._inner_node, $._node_content],

  rules: {
    // ------------------------------------------------------------------------
    // Root & Structural Blocks
    // ------------------------------------------------------------------------

    source_file: ($) => $.document_block,

    document_block: ($) =>
      seq(
        "<document>",
        repeat($.meta_tag), // zero or more <meta /> before body
        $.body_block, // exactly one <body> (required)
        "</document>",
      ),

    meta_tag: ($) =>
      seq("<", field("name", $.tag_name), repeat($.attribute), "/>"),

    body_block: ($) => seq("<body>", repeat($._inner_node), "</body>"),

    // ------------------------------------------------------------------------
    // Inner nodes (shared between document body and meta context)
    // ------------------------------------------------------------------------

    _inner_node: ($) => choice($.element, $.self_closing_element, $.text),

    element: ($) => seq($.start_tag, repeat($._node_content), $.end_tag),

    start_tag: ($) =>
      seq("<", field("name", $.tag_name), repeat($.attribute), ">"),
    end_tag: ($) => seq("</", field("name", $.tag_name), ">"),
    self_closing_element: ($) =>
      seq("<", field("name", $.tag_name), repeat($.attribute), "/>"),

    // Nested content inside elements
    _node_content: ($) => $._inner_node,

    // ------------------------------------------------------------------------
    // Attributes
    // ------------------------------------------------------------------------

    attribute: ($) =>
      seq(
        field("name", $.attribute_name),
        "=",
        field("value", $.attribute_value),
      ),

    tag_name: (_) => /[a-zA-Z0-9_-]+/,
    attribute_name: (_) => /[a-zA-Z0-9_-]+/,
    attribute_value: ($) =>
      choice(
        seq('"', token.immediate(/[^"]*/), '"'),
        seq("'", token.immediate(/[^']*/), "'"),
      ),

    // ------------------------------------------------------------------------
    // Text & Comments
    // ------------------------------------------------------------------------

    // ponytail: Only matches text containing at least one non-whitespace, non-bracket char. Pure whitespace is absorbed by extras.
    text: (_) => /[^<>\s]([^<>]*[^<>\s])?/,
    comment: (_) => seq("<!--", repeat(/[^-]|-[^-]/), "-->"),
  },
});
