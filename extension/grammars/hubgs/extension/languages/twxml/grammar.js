/**
 * @file TWXML grammar for tree-sitter
 */
module.exports = grammar({
  name: 'twxml',
  extras: $ => [/\s+/, $.comment],
  rules: {
    document: $ => repeat($._node),
    _node: $ => choice($.element, $.self_closing_element, $.text),
    element: $ => seq($.start_tag, repeat($._node), $.end_tag),
    start_tag: $ => seq('<', field('name', $.tag_name), repeat($.attribute), '>'),
    end_tag: $ => seq('</', field('name', $.tag_name), '>'),
    self_closing_element: $ => seq('<', field('name', $.tag_name), repeat($.attribute), '/>'),
    attribute: $ => seq(field('name', $.attribute_name), '=', field('value', $.attribute_value)),
    tag_name: _ => /[a-zA-Z0-9]+/,
    attribute_name: _ => /[a-zA-Z0-9-]+/,
    attribute_value: _ => choice(seq('"', /[^"]*/, '"'), seq("'", /[^']*/, "'")),
    text: _ => /[^<]+/,
    comment: _ => seq('<!--', repeat(/[^-]|-[^-]/), '-->'),
  }
});
