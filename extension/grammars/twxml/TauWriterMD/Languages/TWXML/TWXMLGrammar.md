# TWXML Tree-Sitter Grammar
A lean, prose-focused XML grammar for TauWriter, designed to support IDE-style features and 1:1 Markdown rendering.

```js
/**
 * @file TWXML grammar for tree-sitter
 * @authors Jarbear82, Gemini CLI
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'twxml',

  extras: $ => [
    /\s+/,
    $.comment,
  ],

  rules: {
    document: $ => repeat($._node),

    _node: $ => choice(
      $.element,
      $.self_closing_element,
      $.text
    ),

    element: $ => seq(
      $.start_tag,
      repeat($._node),
      $.end_tag
    ),

    start_tag: $ => seq(
      '<',
      field('name', $.tag_name),
      repeat($.attribute),
      '>'
    ),

    end_tag: $ => seq(
      '</',
      field('name', $.tag_name),
      '>'
    ),

    self_closing_element: $ => seq(
      '<',
      field('name', $.tag_name),
      repeat($.attribute),
      '/>'
    ),

    attribute: $ => seq(
      field('name', $.attribute_name),
      '=',
      field('value', $.attribute_value)
    ),

    tag_name: _ => /[a-zA-Z0-9]+/,

    attribute_name: _ => /[a-zA-Z0-9-]+/,

    attribute_value: _ => choice(
      seq('"', /[^"]*/, '"'),
      seq("'", /[^']*/, "'")
    ),

    // Text node: anything that isn't a tag start/end or comment start
    text: _ => /[^<]+/,

    comment: _ => seq(
      '<!--',
      repeat(/[^-]|-[^-]/),
      '-->'
    ),
  }
});
```
