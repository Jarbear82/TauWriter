#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 279
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 110
#define ALIAS_COUNT 0
#define TOKEN_COUNT 59
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 8
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 5

enum ts_symbol_identifiers {
  anon_sym_COMMA = 1,
  anon_sym_IMPORTS = 2,
  anon_sym_LBRACK = 3,
  anon_sym_RBRACK = 4,
  anon_sym_FROM = 5,
  anon_sym_DEFINITIONS = 6,
  anon_sym_FIELDS = 7,
  anon_sym_COLON = 8,
  anon_sym_ENUMS = 9,
  anon_sym_LBRACE = 10,
  anon_sym_RBRACE = 11,
  anon_sym_STRUCTS = 12,
  anon_sym_HUBS = 13,
  anon_sym_EQ = 14,
  anon_sym_LPAREN = 15,
  anon_sym_RPAREN = 16,
  anon_sym_ALLOWS = 17,
  anon_sym_DASH_GT = 18,
  anon_sym_LT_DASH = 19,
  anon_sym_LT_DASH_GT = 20,
  anon_sym_DASH = 21,
  anon_sym_STAR = 22,
  anon_sym_DOT_DOT = 23,
  anon_sym_INSTANCES = 24,
  anon_sym_ATmetadata = 25,
  anon_sym_LT = 26,
  anon_sym_GT = 27,
  anon_sym_ATcomputed = 28,
  anon_sym_ATdefault = 29,
  anon_sym_DOT = 30,
  anon_sym_BANG = 31,
  anon_sym_AMP_AMP = 32,
  anon_sym_PIPE_PIPE = 33,
  anon_sym_EQ_EQ = 34,
  anon_sym_BANG_EQ = 35,
  anon_sym_PLUS = 36,
  anon_sym_SLASH = 37,
  anon_sym_true = 38,
  anon_sym_false = 39,
  sym_identifier = 40,
  sym_number = 41,
  anon_sym_DQUOTE = 42,
  aux_sym_string_token1 = 43,
  anon_sym_SQUOTE = 44,
  aux_sym_string_token2 = 45,
  anon_sym_DQUOTE_DQUOTE_DQUOTE = 46,
  aux_sym_string_token3 = 47,
  aux_sym_string_token4 = 48,
  aux_sym_string_token5 = 49,
  anon_sym_SQUOTE_SQUOTE_SQUOTE = 50,
  aux_sym_string_token6 = 51,
  aux_sym_string_token7 = 52,
  aux_sym_string_token8 = 53,
  anon_sym_BQUOTE = 54,
  aux_sym_template_string_token1 = 55,
  anon_sym_DOLLAR = 56,
  sym_escape_sequence = 57,
  sym_comment = 58,
  sym_document = 59,
  sym_imports_section = 60,
  sym_import_statement = 61,
  sym_definitions_section = 62,
  sym_fields_block = 63,
  sym_field_definition = 64,
  sym_enums_block = 65,
  sym_enum_definition = 66,
  sym_structs_block = 67,
  sym_struct_definition = 68,
  sym_hubs_block = 69,
  sym_hub_definition = 70,
  sym_hub_field = 71,
  sym_hub_role = 72,
  sym_role_direction = 73,
  sym_multiplicity = 74,
  sym_instances_section = 75,
  sym_instance_block = 76,
  sym_instance_assignment = 77,
  sym_metadata_block = 78,
  sym_type = 79,
  sym_generic_type = 80,
  sym_decorator = 81,
  sym__expression = 82,
  sym_parenthesized_expression = 83,
  sym_array = 84,
  sym_member_expression = 85,
  sym_unary_expression = 86,
  sym_binary_expression = 87,
  sym_boolean = 88,
  sym_string = 89,
  sym_template_string = 90,
  aux_sym_document_repeat1 = 91,
  aux_sym_imports_section_repeat1 = 92,
  aux_sym_import_statement_repeat1 = 93,
  aux_sym_definitions_section_repeat1 = 94,
  aux_sym_fields_block_repeat1 = 95,
  aux_sym_enums_block_repeat1 = 96,
  aux_sym_structs_block_repeat1 = 97,
  aux_sym_hubs_block_repeat1 = 98,
  aux_sym_hub_definition_repeat1 = 99,
  aux_sym_instances_section_repeat1 = 100,
  aux_sym_instance_block_repeat1 = 101,
  aux_sym_metadata_block_repeat1 = 102,
  aux_sym_generic_type_repeat1 = 103,
  aux_sym_array_repeat1 = 104,
  aux_sym_string_repeat1 = 105,
  aux_sym_string_repeat2 = 106,
  aux_sym_string_repeat3 = 107,
  aux_sym_string_repeat4 = 108,
  aux_sym_template_string_repeat1 = 109,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_COMMA] = ",",
  [anon_sym_IMPORTS] = "IMPORTS",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [anon_sym_FROM] = "FROM",
  [anon_sym_DEFINITIONS] = "DEFINITIONS",
  [anon_sym_FIELDS] = "FIELDS",
  [anon_sym_COLON] = ":",
  [anon_sym_ENUMS] = "ENUMS",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_STRUCTS] = "STRUCTS",
  [anon_sym_HUBS] = "HUBS",
  [anon_sym_EQ] = "=",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [anon_sym_ALLOWS] = "ALLOWS",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_LT_DASH] = "<-",
  [anon_sym_LT_DASH_GT] = "<->",
  [anon_sym_DASH] = "-",
  [anon_sym_STAR] = "*",
  [anon_sym_DOT_DOT] = "..",
  [anon_sym_INSTANCES] = "INSTANCES",
  [anon_sym_ATmetadata] = "@metadata",
  [anon_sym_LT] = "<",
  [anon_sym_GT] = ">",
  [anon_sym_ATcomputed] = "@computed",
  [anon_sym_ATdefault] = "@default",
  [anon_sym_DOT] = ".",
  [anon_sym_BANG] = "!",
  [anon_sym_AMP_AMP] = "&&",
  [anon_sym_PIPE_PIPE] = "||",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_PLUS] = "+",
  [anon_sym_SLASH] = "/",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [sym_identifier] = "identifier",
  [sym_number] = "number",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym_string_token1] = "string_token1",
  [anon_sym_SQUOTE] = "'",
  [aux_sym_string_token2] = "string_token2",
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = "\"\"\"",
  [aux_sym_string_token3] = "string_token3",
  [aux_sym_string_token4] = "string_token4",
  [aux_sym_string_token5] = "string_token5",
  [anon_sym_SQUOTE_SQUOTE_SQUOTE] = "'''",
  [aux_sym_string_token6] = "string_token6",
  [aux_sym_string_token7] = "string_token7",
  [aux_sym_string_token8] = "string_token8",
  [anon_sym_BQUOTE] = "`",
  [aux_sym_template_string_token1] = "template_string_token1",
  [anon_sym_DOLLAR] = "$",
  [sym_escape_sequence] = "escape_sequence",
  [sym_comment] = "comment",
  [sym_document] = "document",
  [sym_imports_section] = "imports_section",
  [sym_import_statement] = "import_statement",
  [sym_definitions_section] = "definitions_section",
  [sym_fields_block] = "fields_block",
  [sym_field_definition] = "field_definition",
  [sym_enums_block] = "enums_block",
  [sym_enum_definition] = "enum_definition",
  [sym_structs_block] = "structs_block",
  [sym_struct_definition] = "struct_definition",
  [sym_hubs_block] = "hubs_block",
  [sym_hub_definition] = "hub_definition",
  [sym_hub_field] = "hub_field",
  [sym_hub_role] = "hub_role",
  [sym_role_direction] = "role_direction",
  [sym_multiplicity] = "multiplicity",
  [sym_instances_section] = "instances_section",
  [sym_instance_block] = "instance_block",
  [sym_instance_assignment] = "instance_assignment",
  [sym_metadata_block] = "metadata_block",
  [sym_type] = "type",
  [sym_generic_type] = "generic_type",
  [sym_decorator] = "decorator",
  [sym__expression] = "_expression",
  [sym_parenthesized_expression] = "parenthesized_expression",
  [sym_array] = "array",
  [sym_member_expression] = "member_expression",
  [sym_unary_expression] = "unary_expression",
  [sym_binary_expression] = "binary_expression",
  [sym_boolean] = "boolean",
  [sym_string] = "string",
  [sym_template_string] = "template_string",
  [aux_sym_document_repeat1] = "document_repeat1",
  [aux_sym_imports_section_repeat1] = "imports_section_repeat1",
  [aux_sym_import_statement_repeat1] = "import_statement_repeat1",
  [aux_sym_definitions_section_repeat1] = "definitions_section_repeat1",
  [aux_sym_fields_block_repeat1] = "fields_block_repeat1",
  [aux_sym_enums_block_repeat1] = "enums_block_repeat1",
  [aux_sym_structs_block_repeat1] = "structs_block_repeat1",
  [aux_sym_hubs_block_repeat1] = "hubs_block_repeat1",
  [aux_sym_hub_definition_repeat1] = "hub_definition_repeat1",
  [aux_sym_instances_section_repeat1] = "instances_section_repeat1",
  [aux_sym_instance_block_repeat1] = "instance_block_repeat1",
  [aux_sym_metadata_block_repeat1] = "metadata_block_repeat1",
  [aux_sym_generic_type_repeat1] = "generic_type_repeat1",
  [aux_sym_array_repeat1] = "array_repeat1",
  [aux_sym_string_repeat1] = "string_repeat1",
  [aux_sym_string_repeat2] = "string_repeat2",
  [aux_sym_string_repeat3] = "string_repeat3",
  [aux_sym_string_repeat4] = "string_repeat4",
  [aux_sym_template_string_repeat1] = "template_string_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_IMPORTS] = anon_sym_IMPORTS,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_FROM] = anon_sym_FROM,
  [anon_sym_DEFINITIONS] = anon_sym_DEFINITIONS,
  [anon_sym_FIELDS] = anon_sym_FIELDS,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_ENUMS] = anon_sym_ENUMS,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_STRUCTS] = anon_sym_STRUCTS,
  [anon_sym_HUBS] = anon_sym_HUBS,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_ALLOWS] = anon_sym_ALLOWS,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_LT_DASH] = anon_sym_LT_DASH,
  [anon_sym_LT_DASH_GT] = anon_sym_LT_DASH_GT,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_DOT_DOT] = anon_sym_DOT_DOT,
  [anon_sym_INSTANCES] = anon_sym_INSTANCES,
  [anon_sym_ATmetadata] = anon_sym_ATmetadata,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_ATcomputed] = anon_sym_ATcomputed,
  [anon_sym_ATdefault] = anon_sym_ATdefault,
  [anon_sym_DOT] = anon_sym_DOT,
  [anon_sym_BANG] = anon_sym_BANG,
  [anon_sym_AMP_AMP] = anon_sym_AMP_AMP,
  [anon_sym_PIPE_PIPE] = anon_sym_PIPE_PIPE,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [sym_identifier] = sym_identifier,
  [sym_number] = sym_number,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym_string_token1] = aux_sym_string_token1,
  [anon_sym_SQUOTE] = anon_sym_SQUOTE,
  [aux_sym_string_token2] = aux_sym_string_token2,
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = anon_sym_DQUOTE_DQUOTE_DQUOTE,
  [aux_sym_string_token3] = aux_sym_string_token3,
  [aux_sym_string_token4] = aux_sym_string_token4,
  [aux_sym_string_token5] = aux_sym_string_token5,
  [anon_sym_SQUOTE_SQUOTE_SQUOTE] = anon_sym_SQUOTE_SQUOTE_SQUOTE,
  [aux_sym_string_token6] = aux_sym_string_token6,
  [aux_sym_string_token7] = aux_sym_string_token7,
  [aux_sym_string_token8] = aux_sym_string_token8,
  [anon_sym_BQUOTE] = anon_sym_BQUOTE,
  [aux_sym_template_string_token1] = aux_sym_template_string_token1,
  [anon_sym_DOLLAR] = anon_sym_DOLLAR,
  [sym_escape_sequence] = sym_escape_sequence,
  [sym_comment] = sym_comment,
  [sym_document] = sym_document,
  [sym_imports_section] = sym_imports_section,
  [sym_import_statement] = sym_import_statement,
  [sym_definitions_section] = sym_definitions_section,
  [sym_fields_block] = sym_fields_block,
  [sym_field_definition] = sym_field_definition,
  [sym_enums_block] = sym_enums_block,
  [sym_enum_definition] = sym_enum_definition,
  [sym_structs_block] = sym_structs_block,
  [sym_struct_definition] = sym_struct_definition,
  [sym_hubs_block] = sym_hubs_block,
  [sym_hub_definition] = sym_hub_definition,
  [sym_hub_field] = sym_hub_field,
  [sym_hub_role] = sym_hub_role,
  [sym_role_direction] = sym_role_direction,
  [sym_multiplicity] = sym_multiplicity,
  [sym_instances_section] = sym_instances_section,
  [sym_instance_block] = sym_instance_block,
  [sym_instance_assignment] = sym_instance_assignment,
  [sym_metadata_block] = sym_metadata_block,
  [sym_type] = sym_type,
  [sym_generic_type] = sym_generic_type,
  [sym_decorator] = sym_decorator,
  [sym__expression] = sym__expression,
  [sym_parenthesized_expression] = sym_parenthesized_expression,
  [sym_array] = sym_array,
  [sym_member_expression] = sym_member_expression,
  [sym_unary_expression] = sym_unary_expression,
  [sym_binary_expression] = sym_binary_expression,
  [sym_boolean] = sym_boolean,
  [sym_string] = sym_string,
  [sym_template_string] = sym_template_string,
  [aux_sym_document_repeat1] = aux_sym_document_repeat1,
  [aux_sym_imports_section_repeat1] = aux_sym_imports_section_repeat1,
  [aux_sym_import_statement_repeat1] = aux_sym_import_statement_repeat1,
  [aux_sym_definitions_section_repeat1] = aux_sym_definitions_section_repeat1,
  [aux_sym_fields_block_repeat1] = aux_sym_fields_block_repeat1,
  [aux_sym_enums_block_repeat1] = aux_sym_enums_block_repeat1,
  [aux_sym_structs_block_repeat1] = aux_sym_structs_block_repeat1,
  [aux_sym_hubs_block_repeat1] = aux_sym_hubs_block_repeat1,
  [aux_sym_hub_definition_repeat1] = aux_sym_hub_definition_repeat1,
  [aux_sym_instances_section_repeat1] = aux_sym_instances_section_repeat1,
  [aux_sym_instance_block_repeat1] = aux_sym_instance_block_repeat1,
  [aux_sym_metadata_block_repeat1] = aux_sym_metadata_block_repeat1,
  [aux_sym_generic_type_repeat1] = aux_sym_generic_type_repeat1,
  [aux_sym_array_repeat1] = aux_sym_array_repeat1,
  [aux_sym_string_repeat1] = aux_sym_string_repeat1,
  [aux_sym_string_repeat2] = aux_sym_string_repeat2,
  [aux_sym_string_repeat3] = aux_sym_string_repeat3,
  [aux_sym_string_repeat4] = aux_sym_string_repeat4,
  [aux_sym_template_string_repeat1] = aux_sym_template_string_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_IMPORTS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_FROM] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DEFINITIONS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_FIELDS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ENUMS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STRUCTS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_HUBS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ALLOWS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_INSTANCES] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATmetadata] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATcomputed] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATdefault] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AMP_AMP] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_token2] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_token3] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token4] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token5] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SQUOTE_SQUOTE_SQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_token6] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token7] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token8] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_BQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_template_string_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_DOLLAR] = {
    .visible = true,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_document] = {
    .visible = true,
    .named = true,
  },
  [sym_imports_section] = {
    .visible = true,
    .named = true,
  },
  [sym_import_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_definitions_section] = {
    .visible = true,
    .named = true,
  },
  [sym_fields_block] = {
    .visible = true,
    .named = true,
  },
  [sym_field_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_enums_block] = {
    .visible = true,
    .named = true,
  },
  [sym_enum_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_structs_block] = {
    .visible = true,
    .named = true,
  },
  [sym_struct_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_hubs_block] = {
    .visible = true,
    .named = true,
  },
  [sym_hub_definition] = {
    .visible = true,
    .named = true,
  },
  [sym_hub_field] = {
    .visible = true,
    .named = true,
  },
  [sym_hub_role] = {
    .visible = true,
    .named = true,
  },
  [sym_role_direction] = {
    .visible = true,
    .named = true,
  },
  [sym_multiplicity] = {
    .visible = true,
    .named = true,
  },
  [sym_instances_section] = {
    .visible = true,
    .named = true,
  },
  [sym_instance_block] = {
    .visible = true,
    .named = true,
  },
  [sym_instance_assignment] = {
    .visible = true,
    .named = true,
  },
  [sym_metadata_block] = {
    .visible = true,
    .named = true,
  },
  [sym_type] = {
    .visible = true,
    .named = true,
  },
  [sym_generic_type] = {
    .visible = true,
    .named = true,
  },
  [sym_decorator] = {
    .visible = true,
    .named = true,
  },
  [sym__expression] = {
    .visible = false,
    .named = true,
  },
  [sym_parenthesized_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_array] = {
    .visible = true,
    .named = true,
  },
  [sym_member_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_unary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_binary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_boolean] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym_template_string] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_document_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_imports_section_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_import_statement_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_definitions_section_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fields_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_enums_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_structs_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_hubs_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_hub_definition_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_instances_section_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_instance_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_metadata_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_generic_type_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat3] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat4] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_template_string_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_argument = 1,
  field_left = 2,
  field_object = 3,
  field_operator = 4,
  field_property = 5,
  field_ref = 6,
  field_right = 7,
  field_type = 8,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_argument] = "argument",
  [field_left] = "left",
  [field_object] = "object",
  [field_operator] = "operator",
  [field_property] = "property",
  [field_ref] = "ref",
  [field_right] = "right",
  [field_type] = "type",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 2},
  [2] = {.index = 2, .length = 2},
  [3] = {.index = 4, .length = 3},
  [4] = {.index = 7, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_ref, 0},
    {field_type, 2},
  [2] =
    {field_argument, 1},
    {field_operator, 0},
  [4] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [7] =
    {field_object, 0},
    {field_property, 2},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 164,
  [165] = 165,
  [166] = 166,
  [167] = 167,
  [168] = 168,
  [169] = 169,
  [170] = 170,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 175,
  [176] = 176,
  [177] = 177,
  [178] = 178,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 182,
  [183] = 183,
  [184] = 184,
  [185] = 185,
  [186] = 186,
  [187] = 187,
  [188] = 188,
  [189] = 189,
  [190] = 190,
  [191] = 191,
  [192] = 192,
  [193] = 193,
  [194] = 194,
  [195] = 195,
  [196] = 196,
  [197] = 197,
  [198] = 198,
  [199] = 199,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 205,
  [206] = 206,
  [207] = 207,
  [208] = 208,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 213,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 217,
  [218] = 218,
  [219] = 219,
  [220] = 220,
  [221] = 221,
  [222] = 222,
  [223] = 223,
  [224] = 224,
  [225] = 225,
  [226] = 226,
  [227] = 227,
  [228] = 228,
  [229] = 229,
  [230] = 230,
  [231] = 231,
  [232] = 232,
  [233] = 233,
  [234] = 234,
  [235] = 235,
  [236] = 236,
  [237] = 237,
  [238] = 238,
  [239] = 239,
  [240] = 240,
  [241] = 241,
  [242] = 242,
  [243] = 243,
  [244] = 244,
  [245] = 245,
  [246] = 246,
  [247] = 247,
  [248] = 248,
  [249] = 249,
  [250] = 250,
  [251] = 251,
  [252] = 252,
  [253] = 253,
  [254] = 254,
  [255] = 255,
  [256] = 256,
  [257] = 257,
  [258] = 258,
  [259] = 259,
  [260] = 260,
  [261] = 261,
  [262] = 262,
  [263] = 263,
  [264] = 264,
  [265] = 265,
  [266] = 266,
  [267] = 267,
  [268] = 268,
  [269] = 269,
  [270] = 270,
  [271] = 271,
  [272] = 272,
  [273] = 273,
  [274] = 274,
  [275] = 275,
  [276] = 276,
  [277] = 277,
  [278] = 278,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(108);
      ADVANCE_MAP(
        '!', 155,
        '"', 229,
        '$', 269,
        '&', 11,
        '\'', 237,
        '(', 131,
        ')', 132,
        '*', 142,
        '+', 160,
        ',', 109,
        '-', 139,
        '.', 153,
        '/', 161,
        '0', 220,
        ':', 120,
        '<', 148,
        '=', 130,
        '>', 149,
        '@', 76,
        'A', 178,
        'D', 169,
        'E', 183,
        'F', 173,
        'H', 208,
        'I', 180,
        'S', 203,
        '[', 112,
        '\\', 93,
        ']', 113,
        '`', 262,
        'f', 212,
        't', 216,
        '{', 123,
        '|', 95,
        '}', 124,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(106);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(222);
      if (('B' <= lookahead && lookahead <= 'Z') ||
          ('_' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(2);
      if (lookahead == '\'') ADVANCE(236);
      if (lookahead == '/') ADVANCE(238);
      if (lookahead == '\\') ADVANCE(93);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(241);
      if (lookahead != 0) ADVANCE(243);
      END_STATE();
    case 2:
      if (lookahead == '\n') SKIP(2);
      if (lookahead == '\'') ADVANCE(236);
      if (lookahead == '/') ADVANCE(238);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(241);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(243);
      END_STATE();
    case 3:
      if (lookahead == '\n') SKIP(4);
      if (lookahead == '"') ADVANCE(228);
      if (lookahead == '/') ADVANCE(230);
      if (lookahead == '\\') ADVANCE(93);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(233);
      if (lookahead != 0) ADVANCE(235);
      END_STATE();
    case 4:
      if (lookahead == '\n') SKIP(4);
      if (lookahead == '"') ADVANCE(228);
      if (lookahead == '/') ADVANCE(230);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(233);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(235);
      END_STATE();
    case 5:
      ADVANCE_MAP(
        '!', 154,
        '"', 229,
        '\'', 237,
        '(', 131,
        ')', 132,
        '-', 140,
        '.', 21,
        '/', 16,
        '0', 220,
        '=', 129,
        '[', 112,
        ']', 113,
        '`', 262,
        'f', 212,
        't', 216,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(222);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('_' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 6:
      if (lookahead == '"') ADVANCE(244);
      END_STATE();
    case 7:
      if (lookahead == '"') ADVANCE(244);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(252);
      END_STATE();
    case 8:
      if (lookahead == '"') ADVANCE(9);
      if (lookahead == '/') ADVANCE(246);
      if (lookahead == '\\') ADVANCE(93);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(249);
      if (lookahead != 0) ADVANCE(250);
      END_STATE();
    case 9:
      if (lookahead == '"') ADVANCE(7);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(251);
      END_STATE();
    case 10:
      if (lookahead == '$') ADVANCE(269);
      if (lookahead == '/') ADVANCE(264);
      if (lookahead == '\\') ADVANCE(93);
      if (lookahead == '`') ADVANCE(262);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(267);
      if (lookahead != 0) ADVANCE(268);
      END_STATE();
    case 11:
      if (lookahead == '&') ADVANCE(156);
      END_STATE();
    case 12:
      if (lookahead == '\'') ADVANCE(253);
      END_STATE();
    case 13:
      if (lookahead == '\'') ADVANCE(253);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(261);
      END_STATE();
    case 14:
      if (lookahead == '\'') ADVANCE(15);
      if (lookahead == '/') ADVANCE(255);
      if (lookahead == '\\') ADVANCE(93);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(258);
      if (lookahead != 0) ADVANCE(259);
      END_STATE();
    case 15:
      if (lookahead == '\'') ADVANCE(13);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(260);
      END_STATE();
    case 16:
      if (lookahead == '*') ADVANCE(18);
      if (lookahead == '/') ADVANCE(272);
      END_STATE();
    case 17:
      if (lookahead == '*') ADVANCE(17);
      if (lookahead == '/') ADVANCE(271);
      if (lookahead != 0) ADVANCE(18);
      END_STATE();
    case 18:
      if (lookahead == '*') ADVANCE(17);
      if (lookahead != 0) ADVANCE(18);
      END_STATE();
    case 19:
      ADVANCE_MAP(
        ',', 109,
        '-', 141,
        '/', 16,
        '<', 20,
        '=', 129,
        '>', 149,
        '@', 85,
        ']', 113,
        '}', 124,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(19);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 20:
      if (lookahead == '-') ADVANCE(136);
      END_STATE();
    case 21:
      if (lookahead == '.') ADVANCE(143);
      END_STATE();
    case 22:
      if (lookahead == '=') ADVANCE(159);
      END_STATE();
    case 23:
      if (lookahead == '=') ADVANCE(158);
      END_STATE();
    case 24:
      if (lookahead == 'A') ADVANCE(45);
      END_STATE();
    case 25:
      if (lookahead == 'B') ADVANCE(54);
      END_STATE();
    case 26:
      if (lookahead == 'C') ADVANCE(31);
      END_STATE();
    case 27:
      if (lookahead == 'C') ADVANCE(67);
      END_STATE();
    case 28:
      if (lookahead == 'D') ADVANCE(57);
      END_STATE();
    case 29:
      if (lookahead == 'E') ADVANCE(32);
      END_STATE();
    case 30:
      if (lookahead == 'E') ADVANCE(37);
      END_STATE();
    case 31:
      if (lookahead == 'E') ADVANCE(60);
      END_STATE();
    case 32:
      if (lookahead == 'F') ADVANCE(34);
      END_STATE();
    case 33:
      if (lookahead == 'I') ADVANCE(30);
      if (lookahead == 'R') ADVANCE(49);
      END_STATE();
    case 34:
      if (lookahead == 'I') ADVANCE(44);
      END_STATE();
    case 35:
      if (lookahead == 'I') ADVANCE(65);
      END_STATE();
    case 36:
      if (lookahead == 'I') ADVANCE(50);
      END_STATE();
    case 37:
      if (lookahead == 'L') ADVANCE(28);
      END_STATE();
    case 38:
      if (lookahead == 'L') ADVANCE(39);
      END_STATE();
    case 39:
      if (lookahead == 'L') ADVANCE(47);
      END_STATE();
    case 40:
      if (lookahead == 'M') ADVANCE(51);
      if (lookahead == 'N') ADVANCE(62);
      END_STATE();
    case 41:
      if (lookahead == 'M') ADVANCE(114);
      END_STATE();
    case 42:
      if (lookahead == 'M') ADVANCE(55);
      END_STATE();
    case 43:
      if (lookahead == 'N') ADVANCE(69);
      END_STATE();
    case 44:
      if (lookahead == 'N') ADVANCE(35);
      END_STATE();
    case 45:
      if (lookahead == 'N') ADVANCE(26);
      END_STATE();
    case 46:
      if (lookahead == 'N') ADVANCE(61);
      END_STATE();
    case 47:
      if (lookahead == 'O') ADVANCE(71);
      END_STATE();
    case 48:
      if (lookahead == 'O') ADVANCE(53);
      END_STATE();
    case 49:
      if (lookahead == 'O') ADVANCE(41);
      END_STATE();
    case 50:
      if (lookahead == 'O') ADVANCE(46);
      END_STATE();
    case 51:
      if (lookahead == 'P') ADVANCE(48);
      END_STATE();
    case 52:
      if (lookahead == 'R') ADVANCE(70);
      END_STATE();
    case 53:
      if (lookahead == 'R') ADVANCE(66);
      END_STATE();
    case 54:
      if (lookahead == 'S') ADVANCE(127);
      END_STATE();
    case 55:
      if (lookahead == 'S') ADVANCE(121);
      END_STATE();
    case 56:
      if (lookahead == 'S') ADVANCE(133);
      END_STATE();
    case 57:
      if (lookahead == 'S') ADVANCE(118);
      END_STATE();
    case 58:
      if (lookahead == 'S') ADVANCE(110);
      END_STATE();
    case 59:
      if (lookahead == 'S') ADVANCE(125);
      END_STATE();
    case 60:
      if (lookahead == 'S') ADVANCE(144);
      END_STATE();
    case 61:
      if (lookahead == 'S') ADVANCE(116);
      END_STATE();
    case 62:
      if (lookahead == 'S') ADVANCE(64);
      END_STATE();
    case 63:
      if (lookahead == 'T') ADVANCE(52);
      END_STATE();
    case 64:
      if (lookahead == 'T') ADVANCE(24);
      END_STATE();
    case 65:
      if (lookahead == 'T') ADVANCE(36);
      END_STATE();
    case 66:
      if (lookahead == 'T') ADVANCE(58);
      END_STATE();
    case 67:
      if (lookahead == 'T') ADVANCE(59);
      END_STATE();
    case 68:
      if (lookahead == 'U') ADVANCE(25);
      END_STATE();
    case 69:
      if (lookahead == 'U') ADVANCE(42);
      END_STATE();
    case 70:
      if (lookahead == 'U') ADVANCE(27);
      END_STATE();
    case 71:
      if (lookahead == 'W') ADVANCE(56);
      END_STATE();
    case 72:
      if (lookahead == 'a') ADVANCE(78);
      END_STATE();
    case 73:
      if (lookahead == 'a') ADVANCE(146);
      END_STATE();
    case 74:
      if (lookahead == 'a') ADVANCE(92);
      END_STATE();
    case 75:
      if (lookahead == 'a') ADVANCE(91);
      END_STATE();
    case 76:
      if (lookahead == 'c') ADVANCE(86);
      if (lookahead == 'd') ADVANCE(79);
      if (lookahead == 'm') ADVANCE(80);
      END_STATE();
    case 77:
      if (lookahead == 'd') ADVANCE(150);
      END_STATE();
    case 78:
      if (lookahead == 'd') ADVANCE(75);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(82);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(89);
      END_STATE();
    case 81:
      if (lookahead == 'e') ADVANCE(77);
      END_STATE();
    case 82:
      if (lookahead == 'f') ADVANCE(74);
      END_STATE();
    case 83:
      if (lookahead == 'l') ADVANCE(88);
      END_STATE();
    case 84:
      if (lookahead == 'm') ADVANCE(87);
      END_STATE();
    case 85:
      if (lookahead == 'm') ADVANCE(80);
      END_STATE();
    case 86:
      if (lookahead == 'o') ADVANCE(84);
      END_STATE();
    case 87:
      if (lookahead == 'p') ADVANCE(94);
      END_STATE();
    case 88:
      if (lookahead == 't') ADVANCE(151);
      END_STATE();
    case 89:
      if (lookahead == 't') ADVANCE(72);
      END_STATE();
    case 90:
      if (lookahead == 't') ADVANCE(81);
      END_STATE();
    case 91:
      if (lookahead == 't') ADVANCE(73);
      END_STATE();
    case 92:
      if (lookahead == 'u') ADVANCE(83);
      END_STATE();
    case 93:
      ADVANCE_MAP(
        'u', 105,
        '"', 270,
        '\'', 270,
        '/', 270,
        '\\', 270,
        '`', 270,
        'b', 270,
        'f', 270,
        'n', 270,
        'r', 270,
        't', 270,
      );
      END_STATE();
    case 94:
      if (lookahead == 'u') ADVANCE(90);
      END_STATE();
    case 95:
      if (lookahead == '|') ADVANCE(157);
      END_STATE();
    case 96:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(100);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(226);
      END_STATE();
    case 97:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(224);
      END_STATE();
    case 98:
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(225);
      END_STATE();
    case 99:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(223);
      END_STATE();
    case 100:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(226);
      END_STATE();
    case 101:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(270);
      END_STATE();
    case 102:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(227);
      END_STATE();
    case 103:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(101);
      END_STATE();
    case 104:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(103);
      END_STATE();
    case 105:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(104);
      END_STATE();
    case 106:
      if (eof) ADVANCE(108);
      ADVANCE_MAP(
        '!', 155,
        '"', 229,
        '$', 269,
        '&', 11,
        '\'', 237,
        '(', 131,
        ')', 132,
        '*', 142,
        '+', 160,
        ',', 109,
        '-', 139,
        '.', 153,
        '/', 161,
        '0', 220,
        ':', 120,
        '<', 148,
        '=', 130,
        '>', 149,
        '@', 76,
        'A', 178,
        'D', 169,
        'E', 183,
        'F', 173,
        'H', 208,
        'I', 180,
        'S', 203,
        '[', 112,
        ']', 113,
        '`', 262,
        'f', 212,
        't', 216,
        '{', 123,
        '|', 95,
        '}', 124,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(106);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(222);
      if (('B' <= lookahead && lookahead <= 'Z') ||
          ('_' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 107:
      if (eof) ADVANCE(108);
      ADVANCE_MAP(
        '!', 22,
        '&', 11,
        ')', 132,
        '*', 142,
        '+', 160,
        ',', 109,
        '-', 138,
        '.', 152,
        '/', 161,
        '<', 147,
        '=', 23,
        '>', 149,
        'A', 38,
        'D', 29,
        'E', 43,
        'F', 33,
        'H', 68,
        'I', 40,
        'S', 63,
        ']', 113,
        '|', 95,
        '}', 124,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(107);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(anon_sym_IMPORTS);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(anon_sym_IMPORTS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_FROM);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(anon_sym_FROM);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(anon_sym_DEFINITIONS);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(anon_sym_DEFINITIONS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(anon_sym_FIELDS);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(anon_sym_FIELDS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(anon_sym_ENUMS);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(anon_sym_ENUMS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(anon_sym_STRUCTS);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(anon_sym_STRUCTS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(anon_sym_HUBS);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(anon_sym_HUBS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(158);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(anon_sym_ALLOWS);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(anon_sym_ALLOWS);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(anon_sym_LT_DASH);
      if (lookahead == '>') ADVANCE(137);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(anon_sym_LT_DASH_GT);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '0') ADVANCE(221);
      if (lookahead == '>') ADVANCE(135);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(222);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '0') ADVANCE(221);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(222);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '>') ADVANCE(135);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(anon_sym_DOT_DOT);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(anon_sym_INSTANCES);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(anon_sym_INSTANCES);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(anon_sym_ATmetadata);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(anon_sym_LT);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '-') ADVANCE(136);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(anon_sym_GT);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(anon_sym_ATcomputed);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(anon_sym_ATdefault);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (lookahead == '.') ADVANCE(143);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(anon_sym_BANG);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(anon_sym_BANG);
      if (lookahead == '=') ADVANCE(159);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(anon_sym_AMP_AMP);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(anon_sym_PIPE_PIPE);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(18);
      if (lookahead == '/') ADVANCE(272);
      END_STATE();
    case 162:
      ACCEPT_TOKEN(anon_sym_true);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(anon_sym_false);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'A') ADVANCE(185);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 165:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'B') ADVANCE(194);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'C') ADVANCE(171);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'C') ADVANCE(207);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'D') ADVANCE(197);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'E') ADVANCE(172);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'E') ADVANCE(177);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'E') ADVANCE(200);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'F') ADVANCE(174);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'I') ADVANCE(170);
      if (lookahead == 'R') ADVANCE(189);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'I') ADVANCE(184);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 175:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'I') ADVANCE(205);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 176:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'I') ADVANCE(190);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 177:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'L') ADVANCE(168);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'L') ADVANCE(179);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'L') ADVANCE(187);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'M') ADVANCE(191);
      if (lookahead == 'N') ADVANCE(202);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'M') ADVANCE(115);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'M') ADVANCE(195);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'N') ADVANCE(209);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'N') ADVANCE(175);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'N') ADVANCE(166);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'N') ADVANCE(201);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'O') ADVANCE(211);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'O') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'O') ADVANCE(181);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'O') ADVANCE(186);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'P') ADVANCE(188);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'R') ADVANCE(210);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'R') ADVANCE(206);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(128);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(122);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(134);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(119);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(111);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 199:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(126);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 200:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(145);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(117);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'S') ADVANCE(204);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'T') ADVANCE(192);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'T') ADVANCE(164);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 205:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'T') ADVANCE(176);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 206:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'T') ADVANCE(198);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'T') ADVANCE(199);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'U') ADVANCE(165);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 209:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'U') ADVANCE(182);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'U') ADVANCE(167);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'W') ADVANCE(196);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 212:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(215);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(162);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 214:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(163);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(217);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(218);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(214);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z') ||
          (0xa1 <= lookahead && lookahead <= 0x10ff)) ADVANCE(219);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(sym_number);
      ADVANCE_MAP(
        '.', 99,
        'B', 97,
        'b', 97,
        'E', 96,
        'e', 96,
        'O', 98,
        'o', 98,
        'X', 102,
        'x', 102,
      );
      END_STATE();
    case 221:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(99);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(96);
      END_STATE();
    case 222:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(99);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(96);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(222);
      END_STATE();
    case 223:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(96);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(223);
      END_STATE();
    case 224:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(224);
      END_STATE();
    case 225:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(225);
      END_STATE();
    case 226:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(226);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(227);
      END_STATE();
    case 228:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 229:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      if (lookahead == '"') ADVANCE(6);
      END_STATE();
    case 230:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '*') ADVANCE(232);
      if (lookahead == '/') ADVANCE(234);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(235);
      END_STATE();
    case 231:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '*') ADVANCE(231);
      if (lookahead == '/') ADVANCE(235);
      if (lookahead == '\n' ||
          lookahead == '"' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(232);
      END_STATE();
    case 232:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '*') ADVANCE(231);
      if (lookahead == '\n' ||
          lookahead == '"' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(232);
      END_STATE();
    case 233:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '/') ADVANCE(230);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(233);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(235);
      END_STATE();
    case 234:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '"' ||
          lookahead == '\\') ADVANCE(272);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(234);
      END_STATE();
    case 235:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(235);
      END_STATE();
    case 236:
      ACCEPT_TOKEN(anon_sym_SQUOTE);
      END_STATE();
    case 237:
      ACCEPT_TOKEN(anon_sym_SQUOTE);
      if (lookahead == '\'') ADVANCE(12);
      END_STATE();
    case 238:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '*') ADVANCE(240);
      if (lookahead == '/') ADVANCE(242);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(243);
      END_STATE();
    case 239:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '*') ADVANCE(239);
      if (lookahead == '/') ADVANCE(243);
      if (lookahead == '\n' ||
          lookahead == '\'' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(240);
      END_STATE();
    case 240:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '*') ADVANCE(239);
      if (lookahead == '\n' ||
          lookahead == '\'' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(240);
      END_STATE();
    case 241:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '/') ADVANCE(238);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(241);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(243);
      END_STATE();
    case 242:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '\'' ||
          lookahead == '\\') ADVANCE(272);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(242);
      END_STATE();
    case 243:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(243);
      END_STATE();
    case 244:
      ACCEPT_TOKEN(anon_sym_DQUOTE_DQUOTE_DQUOTE);
      END_STATE();
    case 245:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '\n') ADVANCE(250);
      if (lookahead == '"' ||
          lookahead == '\\') ADVANCE(272);
      if (lookahead != 0) ADVANCE(245);
      END_STATE();
    case 246:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '*') ADVANCE(248);
      if (lookahead == '/') ADVANCE(245);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(250);
      END_STATE();
    case 247:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '*') ADVANCE(247);
      if (lookahead == '/') ADVANCE(250);
      if (lookahead == '"' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(248);
      END_STATE();
    case 248:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '*') ADVANCE(247);
      if (lookahead == '"' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(248);
      END_STATE();
    case 249:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '/') ADVANCE(246);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(249);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(250);
      END_STATE();
    case 250:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(250);
      END_STATE();
    case 251:
      ACCEPT_TOKEN(aux_sym_string_token4);
      END_STATE();
    case 252:
      ACCEPT_TOKEN(aux_sym_string_token5);
      END_STATE();
    case 253:
      ACCEPT_TOKEN(anon_sym_SQUOTE_SQUOTE_SQUOTE);
      END_STATE();
    case 254:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead == '\n') ADVANCE(259);
      if (lookahead == '\'' ||
          lookahead == '\\') ADVANCE(272);
      if (lookahead != 0) ADVANCE(254);
      END_STATE();
    case 255:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead == '*') ADVANCE(257);
      if (lookahead == '/') ADVANCE(254);
      if (lookahead != 0 &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(259);
      END_STATE();
    case 256:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead == '*') ADVANCE(256);
      if (lookahead == '/') ADVANCE(259);
      if (lookahead == '\'' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(257);
      END_STATE();
    case 257:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead == '*') ADVANCE(256);
      if (lookahead == '\'' ||
          lookahead == '\\') ADVANCE(18);
      if (lookahead != 0) ADVANCE(257);
      END_STATE();
    case 258:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead == '/') ADVANCE(255);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(258);
      if (lookahead != 0 &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(259);
      END_STATE();
    case 259:
      ACCEPT_TOKEN(aux_sym_string_token6);
      if (lookahead != 0 &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(259);
      END_STATE();
    case 260:
      ACCEPT_TOKEN(aux_sym_string_token7);
      END_STATE();
    case 261:
      ACCEPT_TOKEN(aux_sym_string_token8);
      END_STATE();
    case 262:
      ACCEPT_TOKEN(anon_sym_BQUOTE);
      END_STATE();
    case 263:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead == '\n') ADVANCE(268);
      if (lookahead == '$' ||
          lookahead == '\\' ||
          lookahead == '`') ADVANCE(272);
      if (lookahead != 0) ADVANCE(263);
      END_STATE();
    case 264:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead == '*') ADVANCE(266);
      if (lookahead == '/') ADVANCE(263);
      if (lookahead != 0 &&
          lookahead != '$' &&
          lookahead != '\\' &&
          lookahead != '`') ADVANCE(268);
      END_STATE();
    case 265:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead == '*') ADVANCE(265);
      if (lookahead == '/') ADVANCE(268);
      if (lookahead == '$' ||
          lookahead == '\\' ||
          lookahead == '`') ADVANCE(18);
      if (lookahead != 0) ADVANCE(266);
      END_STATE();
    case 266:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead == '*') ADVANCE(265);
      if (lookahead == '$' ||
          lookahead == '\\' ||
          lookahead == '`') ADVANCE(18);
      if (lookahead != 0) ADVANCE(266);
      END_STATE();
    case 267:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead == '/') ADVANCE(264);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(267);
      if (lookahead != 0 &&
          lookahead != '$' &&
          lookahead != '\\' &&
          lookahead != '`') ADVANCE(268);
      END_STATE();
    case 268:
      ACCEPT_TOKEN(aux_sym_template_string_token1);
      if (lookahead != 0 &&
          lookahead != '$' &&
          lookahead != '\\' &&
          lookahead != '`') ADVANCE(268);
      END_STATE();
    case 269:
      ACCEPT_TOKEN(anon_sym_DOLLAR);
      END_STATE();
    case 270:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 271:
      ACCEPT_TOKEN(sym_comment);
      END_STATE();
    case 272:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(272);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 107},
  [2] = {.lex_state = 5},
  [3] = {.lex_state = 5},
  [4] = {.lex_state = 5},
  [5] = {.lex_state = 5},
  [6] = {.lex_state = 5},
  [7] = {.lex_state = 5},
  [8] = {.lex_state = 5},
  [9] = {.lex_state = 5},
  [10] = {.lex_state = 5},
  [11] = {.lex_state = 5},
  [12] = {.lex_state = 5},
  [13] = {.lex_state = 5},
  [14] = {.lex_state = 5},
  [15] = {.lex_state = 5},
  [16] = {.lex_state = 5},
  [17] = {.lex_state = 107},
  [18] = {.lex_state = 107},
  [19] = {.lex_state = 107},
  [20] = {.lex_state = 107},
  [21] = {.lex_state = 107},
  [22] = {.lex_state = 107},
  [23] = {.lex_state = 107},
  [24] = {.lex_state = 107},
  [25] = {.lex_state = 107},
  [26] = {.lex_state = 107},
  [27] = {.lex_state = 107},
  [28] = {.lex_state = 107},
  [29] = {.lex_state = 107},
  [30] = {.lex_state = 107},
  [31] = {.lex_state = 107},
  [32] = {.lex_state = 107},
  [33] = {.lex_state = 107},
  [34] = {.lex_state = 107},
  [35] = {.lex_state = 107},
  [36] = {.lex_state = 107},
  [37] = {.lex_state = 107},
  [38] = {.lex_state = 107},
  [39] = {.lex_state = 107},
  [40] = {.lex_state = 107},
  [41] = {.lex_state = 107},
  [42] = {.lex_state = 107},
  [43] = {.lex_state = 107},
  [44] = {.lex_state = 107},
  [45] = {.lex_state = 19},
  [46] = {.lex_state = 107},
  [47] = {.lex_state = 107},
  [48] = {.lex_state = 8},
  [49] = {.lex_state = 14},
  [50] = {.lex_state = 8},
  [51] = {.lex_state = 107},
  [52] = {.lex_state = 14},
  [53] = {.lex_state = 14},
  [54] = {.lex_state = 8},
  [55] = {.lex_state = 19},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 10},
  [58] = {.lex_state = 0},
  [59] = {.lex_state = 10},
  [60] = {.lex_state = 10},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 19},
  [63] = {.lex_state = 19},
  [64] = {.lex_state = 1},
  [65] = {.lex_state = 107},
  [66] = {.lex_state = 19},
  [67] = {.lex_state = 19},
  [68] = {.lex_state = 10},
  [69] = {.lex_state = 1},
  [70] = {.lex_state = 1},
  [71] = {.lex_state = 19},
  [72] = {.lex_state = 19},
  [73] = {.lex_state = 3},
  [74] = {.lex_state = 19},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 3},
  [77] = {.lex_state = 19},
  [78] = {.lex_state = 3},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 19},
  [81] = {.lex_state = 0},
  [82] = {.lex_state = 19},
  [83] = {.lex_state = 19},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
  [86] = {.lex_state = 0},
  [87] = {.lex_state = 19},
  [88] = {.lex_state = 0},
  [89] = {.lex_state = 0},
  [90] = {.lex_state = 19},
  [91] = {.lex_state = 19},
  [92] = {.lex_state = 0},
  [93] = {.lex_state = 0},
  [94] = {.lex_state = 19},
  [95] = {.lex_state = 0},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 0},
  [99] = {.lex_state = 0},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 0},
  [102] = {.lex_state = 0},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 0},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 19},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 19},
  [113] = {.lex_state = 0},
  [114] = {.lex_state = 19},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 19},
  [117] = {.lex_state = 19},
  [118] = {.lex_state = 19},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 19},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 19},
  [125] = {.lex_state = 0},
  [126] = {.lex_state = 0},
  [127] = {.lex_state = 19},
  [128] = {.lex_state = 0},
  [129] = {.lex_state = 0},
  [130] = {.lex_state = 0},
  [131] = {.lex_state = 19},
  [132] = {.lex_state = 0},
  [133] = {.lex_state = 0},
  [134] = {.lex_state = 0},
  [135] = {.lex_state = 0},
  [136] = {.lex_state = 0},
  [137] = {.lex_state = 0},
  [138] = {.lex_state = 0},
  [139] = {.lex_state = 0},
  [140] = {.lex_state = 0},
  [141] = {.lex_state = 0},
  [142] = {.lex_state = 0},
  [143] = {.lex_state = 19},
  [144] = {.lex_state = 0},
  [145] = {.lex_state = 19},
  [146] = {.lex_state = 0},
  [147] = {.lex_state = 0},
  [148] = {.lex_state = 0},
  [149] = {.lex_state = 0},
  [150] = {.lex_state = 0},
  [151] = {.lex_state = 0},
  [152] = {.lex_state = 0},
  [153] = {.lex_state = 0},
  [154] = {.lex_state = 0},
  [155] = {.lex_state = 0},
  [156] = {.lex_state = 0},
  [157] = {.lex_state = 19},
  [158] = {.lex_state = 19},
  [159] = {.lex_state = 0},
  [160] = {.lex_state = 19},
  [161] = {.lex_state = 0},
  [162] = {.lex_state = 0},
  [163] = {.lex_state = 19},
  [164] = {.lex_state = 0},
  [165] = {.lex_state = 19},
  [166] = {.lex_state = 19},
  [167] = {.lex_state = 19},
  [168] = {.lex_state = 0},
  [169] = {.lex_state = 0},
  [170] = {.lex_state = 19},
  [171] = {.lex_state = 0},
  [172] = {.lex_state = 0},
  [173] = {.lex_state = 0},
  [174] = {.lex_state = 0},
  [175] = {.lex_state = 0},
  [176] = {.lex_state = 0},
  [177] = {.lex_state = 19},
  [178] = {.lex_state = 0},
  [179] = {.lex_state = 19},
  [180] = {.lex_state = 0},
  [181] = {.lex_state = 19},
  [182] = {.lex_state = 0},
  [183] = {.lex_state = 0},
  [184] = {.lex_state = 0},
  [185] = {.lex_state = 0},
  [186] = {.lex_state = 0},
  [187] = {.lex_state = 0},
  [188] = {.lex_state = 0},
  [189] = {.lex_state = 0},
  [190] = {.lex_state = 0},
  [191] = {.lex_state = 0},
  [192] = {.lex_state = 0},
  [193] = {.lex_state = 0},
  [194] = {.lex_state = 0},
  [195] = {.lex_state = 0},
  [196] = {.lex_state = 0},
  [197] = {.lex_state = 0},
  [198] = {.lex_state = 0},
  [199] = {.lex_state = 0},
  [200] = {.lex_state = 0},
  [201] = {.lex_state = 0},
  [202] = {.lex_state = 19},
  [203] = {.lex_state = 0},
  [204] = {.lex_state = 0},
  [205] = {.lex_state = 0},
  [206] = {.lex_state = 19},
  [207] = {.lex_state = 0},
  [208] = {.lex_state = 5},
  [209] = {.lex_state = 0},
  [210] = {.lex_state = 0},
  [211] = {.lex_state = 0},
  [212] = {.lex_state = 0},
  [213] = {.lex_state = 0},
  [214] = {.lex_state = 0},
  [215] = {.lex_state = 19},
  [216] = {.lex_state = 0},
  [217] = {.lex_state = 19},
  [218] = {.lex_state = 0},
  [219] = {.lex_state = 0},
  [220] = {.lex_state = 0},
  [221] = {.lex_state = 0},
  [222] = {.lex_state = 0},
  [223] = {.lex_state = 19},
  [224] = {.lex_state = 0},
  [225] = {.lex_state = 0},
  [226] = {.lex_state = 0},
  [227] = {.lex_state = 0},
  [228] = {.lex_state = 0},
  [229] = {.lex_state = 19},
  [230] = {.lex_state = 0},
  [231] = {.lex_state = 0},
  [232] = {.lex_state = 0},
  [233] = {.lex_state = 0},
  [234] = {.lex_state = 0},
  [235] = {.lex_state = 0},
  [236] = {.lex_state = 0},
  [237] = {.lex_state = 0},
  [238] = {.lex_state = 19},
  [239] = {.lex_state = 0},
  [240] = {.lex_state = 0},
  [241] = {.lex_state = 0},
  [242] = {.lex_state = 0},
  [243] = {.lex_state = 0},
  [244] = {.lex_state = 107},
  [245] = {.lex_state = 0},
  [246] = {.lex_state = 0},
  [247] = {.lex_state = 19},
  [248] = {.lex_state = 0},
  [249] = {.lex_state = 107},
  [250] = {.lex_state = 0},
  [251] = {.lex_state = 0},
  [252] = {.lex_state = 0},
  [253] = {.lex_state = 107},
  [254] = {.lex_state = 19},
  [255] = {.lex_state = 0},
  [256] = {.lex_state = 0},
  [257] = {.lex_state = 0},
  [258] = {.lex_state = 0},
  [259] = {.lex_state = 0},
  [260] = {.lex_state = 19},
  [261] = {.lex_state = 0},
  [262] = {.lex_state = 5},
  [263] = {.lex_state = 107},
  [264] = {.lex_state = 0},
  [265] = {.lex_state = 0},
  [266] = {.lex_state = 19},
  [267] = {.lex_state = 5},
  [268] = {.lex_state = 19},
  [269] = {.lex_state = 0},
  [270] = {.lex_state = 0},
  [271] = {.lex_state = 19},
  [272] = {.lex_state = 0},
  [273] = {.lex_state = 0},
  [274] = {.lex_state = 0},
  [275] = {.lex_state = 0},
  [276] = {.lex_state = 5},
  [277] = {.lex_state = 0},
  [278] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_IMPORTS] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_FROM] = ACTIONS(1),
    [anon_sym_DEFINITIONS] = ACTIONS(1),
    [anon_sym_FIELDS] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_ENUMS] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_STRUCTS] = ACTIONS(1),
    [anon_sym_HUBS] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_ALLOWS] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_LT_DASH] = ACTIONS(1),
    [anon_sym_LT_DASH_GT] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_DOT_DOT] = ACTIONS(1),
    [anon_sym_INSTANCES] = ACTIONS(1),
    [anon_sym_ATmetadata] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_ATcomputed] = ACTIONS(1),
    [anon_sym_ATdefault] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [anon_sym_BANG] = ACTIONS(1),
    [anon_sym_AMP_AMP] = ACTIONS(1),
    [anon_sym_PIPE_PIPE] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [sym_identifier] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [anon_sym_SQUOTE] = ACTIONS(1),
    [anon_sym_DQUOTE_DQUOTE_DQUOTE] = ACTIONS(1),
    [anon_sym_SQUOTE_SQUOTE_SQUOTE] = ACTIONS(1),
    [anon_sym_BQUOTE] = ACTIONS(1),
    [anon_sym_DOLLAR] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_document] = STATE(269),
    [sym_imports_section] = STATE(147),
    [sym_definitions_section] = STATE(147),
    [sym_instances_section] = STATE(147),
    [ts_builtin_sym_end] = ACTIONS(5),
    [anon_sym_IMPORTS] = ACTIONS(7),
    [anon_sym_DEFINITIONS] = ACTIONS(9),
    [anon_sym_INSTANCES] = ACTIONS(11),
    [sym_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(15), 1,
      anon_sym_RBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(25), 1,
      sym_identifier,
    ACTIONS(27), 1,
      sym_number,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(35), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [55] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(25), 1,
      sym_identifier,
    ACTIONS(27), 1,
      sym_number,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(39), 1,
      anon_sym_RBRACK,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(35), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [110] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(41), 1,
      anon_sym_RBRACK,
    ACTIONS(43), 1,
      sym_identifier,
    ACTIONS(45), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(34), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [165] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(47), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(38), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [217] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(51), 1,
      sym_identifier,
    ACTIONS(53), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(39), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [269] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(55), 1,
      sym_identifier,
    ACTIONS(57), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(37), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [321] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(59), 1,
      sym_identifier,
    ACTIONS(61), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(25), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [373] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(63), 1,
      sym_identifier,
    ACTIONS(65), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(26), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [425] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(67), 1,
      sym_identifier,
    ACTIONS(69), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(28), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [477] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(71), 1,
      sym_identifier,
    ACTIONS(73), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(40), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [529] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(25), 1,
      sym_identifier,
    ACTIONS(27), 1,
      sym_number,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(35), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [581] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(75), 1,
      sym_identifier,
    ACTIONS(77), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(21), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [633] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(79), 1,
      sym_identifier,
    ACTIONS(81), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(33), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [685] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(83), 1,
      sym_identifier,
    ACTIONS(85), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(36), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [737] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(13), 1,
      anon_sym_LBRACK,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_DASH,
    ACTIONS(21), 1,
      anon_sym_BANG,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(37), 1,
      anon_sym_BQUOTE,
    ACTIONS(87), 1,
      sym_identifier,
    ACTIONS(89), 1,
      sym_number,
    ACTIONS(23), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(19), 9,
      sym__expression,
      sym_parenthesized_expression,
      sym_array,
      sym_member_expression,
      sym_unary_expression,
      sym_binary_expression,
      sym_boolean,
      sym_string,
      sym_template_string,
  [789] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(93), 1,
      anon_sym_SLASH,
    ACTIONS(91), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [810] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(97), 1,
      anon_sym_SLASH,
    ACTIONS(95), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [831] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(103), 1,
      anon_sym_SLASH,
    ACTIONS(99), 11,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [854] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(107), 1,
      anon_sym_SLASH,
    ACTIONS(105), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [875] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(109), 8,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
  [902] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(119), 1,
      anon_sym_SLASH,
    ACTIONS(117), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [923] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(123), 1,
      anon_sym_SLASH,
    ACTIONS(121), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [944] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(127), 1,
      anon_sym_SLASH,
    ACTIONS(125), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [965] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(109), 10,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [990] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(129), 1,
      anon_sym_SLASH,
    ACTIONS(109), 11,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1013] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(133), 1,
      anon_sym_SLASH,
    ACTIONS(131), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1034] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(109), 6,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1063] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(139), 1,
      anon_sym_SLASH,
    ACTIONS(137), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1084] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(143), 1,
      anon_sym_SLASH,
    ACTIONS(141), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1105] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(147), 1,
      anon_sym_SLASH,
    ACTIONS(145), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1126] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(151), 1,
      anon_sym_SLASH,
    ACTIONS(149), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
      anon_sym_RPAREN,
      anon_sym_DASH,
      anon_sym_STAR,
      anon_sym_DOT,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_PLUS,
  [1147] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(153), 1,
      anon_sym_COMMA,
    ACTIONS(155), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      aux_sym_metadata_block_repeat1,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1181] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(159), 1,
      anon_sym_COMMA,
    ACTIONS(161), 1,
      anon_sym_RBRACK,
    STATE(101), 1,
      aux_sym_array_repeat1,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1215] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(163), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [1244] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(165), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [1273] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(167), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [1302] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(169), 1,
      anon_sym_RPAREN,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1330] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(171), 1,
      anon_sym_RPAREN,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1358] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(101), 1,
      anon_sym_DOT,
    ACTIONS(113), 1,
      anon_sym_STAR,
    ACTIONS(115), 1,
      anon_sym_SLASH,
    ACTIONS(173), 1,
      anon_sym_RBRACE,
    ACTIONS(111), 2,
      anon_sym_DASH,
      anon_sym_PLUS,
    ACTIONS(135), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(157), 2,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [1386] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(175), 1,
      anon_sym_RBRACK,
    ACTIONS(177), 1,
      anon_sym_FIELDS,
    ACTIONS(179), 1,
      anon_sym_ENUMS,
    ACTIONS(181), 1,
      anon_sym_STRUCTS,
    ACTIONS(183), 1,
      anon_sym_HUBS,
    STATE(138), 4,
      sym_fields_block,
      sym_enums_block,
      sym_structs_block,
      sym_hubs_block,
  [1411] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(177), 1,
      anon_sym_FIELDS,
    ACTIONS(179), 1,
      anon_sym_ENUMS,
    ACTIONS(181), 1,
      anon_sym_STRUCTS,
    ACTIONS(183), 1,
      anon_sym_HUBS,
    ACTIONS(185), 1,
      anon_sym_RBRACK,
    STATE(230), 4,
      sym_fields_block,
      sym_enums_block,
      sym_structs_block,
      sym_hubs_block,
  [1436] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(177), 1,
      anon_sym_FIELDS,
    ACTIONS(179), 1,
      anon_sym_ENUMS,
    ACTIONS(181), 1,
      anon_sym_STRUCTS,
    ACTIONS(183), 1,
      anon_sym_HUBS,
    ACTIONS(187), 1,
      anon_sym_RBRACK,
    STATE(230), 4,
      sym_fields_block,
      sym_enums_block,
      sym_structs_block,
      sym_hubs_block,
  [1461] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(177), 1,
      anon_sym_FIELDS,
    ACTIONS(179), 1,
      anon_sym_ENUMS,
    ACTIONS(181), 1,
      anon_sym_STRUCTS,
    ACTIONS(183), 1,
      anon_sym_HUBS,
    STATE(230), 4,
      sym_fields_block,
      sym_enums_block,
      sym_structs_block,
      sym_hubs_block,
  [1483] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(191), 1,
      anon_sym_EQ,
    STATE(277), 1,
      sym_role_direction,
    ACTIONS(189), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
    ACTIONS(193), 2,
      anon_sym_DASH_GT,
      anon_sym_LT_DASH_GT,
    ACTIONS(195), 2,
      anon_sym_LT_DASH,
      anon_sym_DASH,
  [1505] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      anon_sym_IMPORTS,
    ACTIONS(9), 1,
      anon_sym_DEFINITIONS,
    ACTIONS(11), 1,
      anon_sym_INSTANCES,
    ACTIONS(197), 1,
      ts_builtin_sym_end,
    STATE(218), 3,
      sym_imports_section,
      sym_definitions_section,
      sym_instances_section,
  [1526] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      anon_sym_IMPORTS,
    ACTIONS(9), 1,
      anon_sym_DEFINITIONS,
    ACTIONS(11), 1,
      anon_sym_INSTANCES,
    ACTIONS(199), 1,
      ts_builtin_sym_end,
    STATE(218), 3,
      sym_imports_section,
      sym_definitions_section,
      sym_instances_section,
  [1547] = 5,
    ACTIONS(201), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(205), 1,
      sym_escape_sequence,
    ACTIONS(207), 1,
      sym_comment,
    STATE(54), 1,
      aux_sym_string_repeat3,
    ACTIONS(203), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1565] = 5,
    ACTIONS(201), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(211), 1,
      sym_escape_sequence,
    STATE(52), 1,
      aux_sym_string_repeat4,
    ACTIONS(209), 3,
      aux_sym_string_token6,
      aux_sym_string_token7,
      aux_sym_string_token8,
  [1583] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(213), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(218), 1,
      sym_escape_sequence,
    STATE(50), 1,
      aux_sym_string_repeat3,
    ACTIONS(215), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1601] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      anon_sym_IMPORTS,
    ACTIONS(9), 1,
      anon_sym_DEFINITIONS,
    ACTIONS(11), 1,
      anon_sym_INSTANCES,
    STATE(218), 3,
      sym_imports_section,
      sym_definitions_section,
      sym_instances_section,
  [1619] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(221), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(225), 1,
      sym_escape_sequence,
    STATE(53), 1,
      aux_sym_string_repeat4,
    ACTIONS(223), 3,
      aux_sym_string_token6,
      aux_sym_string_token7,
      aux_sym_string_token8,
  [1637] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(227), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    ACTIONS(232), 1,
      sym_escape_sequence,
    STATE(53), 1,
      aux_sym_string_repeat4,
    ACTIONS(229), 3,
      aux_sym_string_token6,
      aux_sym_string_token7,
      aux_sym_string_token8,
  [1655] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(221), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(237), 1,
      sym_escape_sequence,
    STATE(50), 1,
      aux_sym_string_repeat3,
    ACTIONS(235), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1673] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(239), 1,
      anon_sym_RBRACE,
    ACTIONS(241), 1,
      anon_sym_ATmetadata,
    ACTIONS(243), 1,
      sym_identifier,
    STATE(153), 1,
      sym_metadata_block,
    STATE(198), 1,
      sym_instance_assignment,
  [1692] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    STATE(176), 1,
      sym_string,
  [1711] = 6,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(245), 1,
      anon_sym_BQUOTE,
    ACTIONS(247), 1,
      aux_sym_template_string_token1,
    ACTIONS(249), 1,
      anon_sym_DOLLAR,
    ACTIONS(251), 1,
      sym_escape_sequence,
    STATE(59), 1,
      aux_sym_template_string_repeat1,
  [1730] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    STATE(172), 1,
      sym_string,
  [1749] = 6,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(253), 1,
      anon_sym_BQUOTE,
    ACTIONS(255), 1,
      aux_sym_template_string_token1,
    ACTIONS(258), 1,
      anon_sym_DOLLAR,
    ACTIONS(261), 1,
      sym_escape_sequence,
    STATE(59), 1,
      aux_sym_template_string_repeat1,
  [1768] = 6,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(249), 1,
      anon_sym_DOLLAR,
    ACTIONS(264), 1,
      anon_sym_BQUOTE,
    ACTIONS(266), 1,
      aux_sym_template_string_token1,
    ACTIONS(268), 1,
      sym_escape_sequence,
    STATE(57), 1,
      aux_sym_template_string_repeat1,
  [1787] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(29), 1,
      anon_sym_DQUOTE,
    ACTIONS(31), 1,
      anon_sym_SQUOTE,
    ACTIONS(33), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    ACTIONS(35), 1,
      anon_sym_SQUOTE_SQUOTE_SQUOTE,
    STATE(203), 1,
      sym_string,
  [1806] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(241), 1,
      anon_sym_ATmetadata,
    ACTIONS(243), 1,
      sym_identifier,
    ACTIONS(270), 1,
      anon_sym_RBRACE,
    STATE(153), 1,
      sym_metadata_block,
    STATE(198), 1,
      sym_instance_assignment,
  [1825] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(241), 1,
      anon_sym_ATmetadata,
    ACTIONS(243), 1,
      sym_identifier,
    ACTIONS(272), 1,
      anon_sym_RBRACE,
    STATE(136), 1,
      sym_instance_assignment,
    STATE(153), 1,
      sym_metadata_block,
  [1844] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(274), 1,
      anon_sym_SQUOTE,
    ACTIONS(276), 1,
      aux_sym_string_token2,
    ACTIONS(279), 1,
      sym_escape_sequence,
    STATE(64), 1,
      aux_sym_string_repeat2,
  [1860] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(284), 1,
      anon_sym_LT,
    ACTIONS(282), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_GT,
  [1872] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(286), 1,
      anon_sym_RBRACE,
    ACTIONS(288), 1,
      sym_identifier,
    STATE(128), 2,
      sym_hub_field,
      sym_hub_role,
  [1886] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(241), 1,
      anon_sym_ATmetadata,
    ACTIONS(243), 1,
      sym_identifier,
    STATE(153), 1,
      sym_metadata_block,
    STATE(198), 1,
      sym_instance_assignment,
  [1902] = 3,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(292), 1,
      sym_escape_sequence,
    ACTIONS(290), 3,
      anon_sym_BQUOTE,
      aux_sym_template_string_token1,
      anon_sym_DOLLAR,
  [1914] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(221), 1,
      anon_sym_SQUOTE,
    ACTIONS(294), 1,
      aux_sym_string_token2,
    ACTIONS(296), 1,
      sym_escape_sequence,
    STATE(64), 1,
      aux_sym_string_repeat2,
  [1930] = 5,
    ACTIONS(201), 1,
      anon_sym_SQUOTE,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(298), 1,
      aux_sym_string_token2,
    ACTIONS(300), 1,
      sym_escape_sequence,
    STATE(69), 1,
      aux_sym_string_repeat2,
  [1946] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(302), 1,
      anon_sym_GT,
    ACTIONS(304), 1,
      sym_identifier,
    STATE(109), 1,
      sym_generic_type,
    STATE(228), 1,
      sym_type,
  [1962] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(304), 1,
      sym_identifier,
    ACTIONS(306), 1,
      anon_sym_GT,
    STATE(109), 1,
      sym_generic_type,
    STATE(228), 1,
      sym_type,
  [1978] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(221), 1,
      anon_sym_DQUOTE,
    ACTIONS(308), 1,
      aux_sym_string_token1,
    ACTIONS(310), 1,
      sym_escape_sequence,
    STATE(76), 1,
      aux_sym_string_repeat1,
  [1994] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      sym_identifier,
    ACTIONS(312), 1,
      anon_sym_RBRACE,
    STATE(186), 2,
      sym_hub_field,
      sym_hub_role,
  [2008] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(314), 1,
      anon_sym_COMMA,
    STATE(75), 1,
      aux_sym_import_statement_repeat1,
    ACTIONS(317), 2,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
  [2022] = 5,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(319), 1,
      anon_sym_DQUOTE,
    ACTIONS(321), 1,
      aux_sym_string_token1,
    ACTIONS(324), 1,
      sym_escape_sequence,
    STATE(76), 1,
      aux_sym_string_repeat1,
  [2038] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      sym_identifier,
    ACTIONS(327), 1,
      anon_sym_RBRACE,
    STATE(186), 2,
      sym_hub_field,
      sym_hub_role,
  [2052] = 5,
    ACTIONS(201), 1,
      anon_sym_DQUOTE,
    ACTIONS(207), 1,
      sym_comment,
    ACTIONS(329), 1,
      aux_sym_string_token1,
    ACTIONS(331), 1,
      sym_escape_sequence,
    STATE(73), 1,
      aux_sym_string_repeat1,
  [2068] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(185), 1,
      anon_sym_RBRACK,
    ACTIONS(333), 1,
      anon_sym_COMMA,
    STATE(95), 1,
      aux_sym_definitions_section_repeat1,
  [2081] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(304), 1,
      sym_identifier,
    STATE(109), 1,
      sym_generic_type,
    STATE(200), 1,
      sym_type,
  [2094] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(335), 1,
      anon_sym_COMMA,
    ACTIONS(337), 1,
      anon_sym_RBRACK,
    STATE(113), 1,
      aux_sym_fields_block_repeat1,
  [2107] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(337), 1,
      anon_sym_RBRACK,
    ACTIONS(339), 1,
      sym_identifier,
    STATE(196), 1,
      sym_field_definition,
  [2120] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(341), 1,
      anon_sym_RBRACK,
    ACTIONS(343), 1,
      sym_identifier,
    STATE(193), 1,
      sym_enum_definition,
  [2133] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(345), 1,
      anon_sym_COMMA,
    ACTIONS(347), 1,
      anon_sym_RBRACK,
    STATE(75), 1,
      aux_sym_import_statement_repeat1,
  [2146] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(341), 1,
      anon_sym_RBRACK,
    ACTIONS(349), 1,
      anon_sym_COMMA,
    STATE(119), 1,
      aux_sym_enums_block_repeat1,
  [2159] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(317), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RBRACE,
  [2168] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 1,
      anon_sym_RBRACK,
    ACTIONS(353), 1,
      sym_identifier,
    STATE(188), 1,
      sym_struct_definition,
  [2181] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(355), 1,
      anon_sym_COMMA,
    ACTIONS(358), 1,
      anon_sym_RBRACK,
    STATE(88), 1,
      aux_sym_instances_section_repeat1,
  [2194] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(351), 1,
      anon_sym_RBRACK,
    ACTIONS(360), 1,
      anon_sym_COMMA,
    STATE(125), 1,
      aux_sym_structs_block_repeat1,
  [2207] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_RBRACK,
    ACTIONS(364), 1,
      sym_identifier,
    STATE(226), 1,
      sym_instance_block,
  [2220] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(366), 1,
      anon_sym_RBRACK,
    ACTIONS(368), 1,
      sym_identifier,
    STATE(168), 1,
      sym_hub_definition,
  [2233] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(370), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_GT,
  [2242] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(366), 1,
      anon_sym_RBRACK,
    ACTIONS(372), 1,
      anon_sym_COMMA,
    STATE(132), 1,
      aux_sym_hubs_block_repeat1,
  [2255] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      sym_identifier,
    STATE(186), 2,
      sym_hub_field,
      sym_hub_role,
  [2266] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(374), 1,
      anon_sym_COMMA,
    ACTIONS(377), 1,
      anon_sym_RBRACK,
    STATE(95), 1,
      aux_sym_definitions_section_repeat1,
  [2279] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(379), 1,
      anon_sym_COMMA,
    ACTIONS(382), 1,
      anon_sym_GT,
    STATE(96), 1,
      aux_sym_generic_type_repeat1,
  [2292] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(384), 1,
      anon_sym_COMMA,
    ACTIONS(386), 1,
      anon_sym_RBRACK,
    STATE(93), 1,
      aux_sym_hubs_block_repeat1,
  [2305] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(302), 1,
      anon_sym_GT,
    ACTIONS(388), 1,
      anon_sym_COMMA,
    STATE(96), 1,
      aux_sym_generic_type_repeat1,
  [2318] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(390), 1,
      anon_sym_COMMA,
    ACTIONS(392), 1,
      anon_sym_RBRACK,
    STATE(89), 1,
      aux_sym_structs_block_repeat1,
  [2331] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(394), 1,
      anon_sym_COMMA,
    ACTIONS(396), 1,
      anon_sym_RBRACK,
    STATE(85), 1,
      aux_sym_enums_block_repeat1,
  [2344] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(39), 1,
      anon_sym_RBRACK,
    ACTIONS(398), 1,
      anon_sym_COMMA,
    STATE(139), 1,
      aux_sym_array_repeat1,
  [2357] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(400), 1,
      anon_sym_COMMA,
    ACTIONS(402), 1,
      anon_sym_RBRACK,
    STATE(81), 1,
      aux_sym_fields_block_repeat1,
  [2370] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(404), 1,
      anon_sym_COMMA,
    ACTIONS(407), 1,
      anon_sym_RBRACK,
    STATE(103), 1,
      aux_sym_imports_section_repeat1,
  [2383] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(409), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_GT,
  [2392] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      anon_sym_LBRACK,
    ACTIONS(413), 1,
      anon_sym_RBRACK,
    STATE(242), 1,
      sym_import_statement,
  [2405] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(415), 1,
      anon_sym_COMMA,
    ACTIONS(418), 1,
      anon_sym_RBRACE,
    STATE(106), 1,
      aux_sym_instance_block_repeat1,
  [2418] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(420), 1,
      anon_sym_COMMA,
    ACTIONS(422), 1,
      anon_sym_RBRACK,
    STATE(75), 1,
      aux_sym_import_statement_repeat1,
  [2431] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(424), 1,
      anon_sym_COMMA,
    ACTIONS(426), 1,
      anon_sym_RBRACK,
    STATE(88), 1,
      aux_sym_instances_section_repeat1,
  [2444] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_GT,
  [2453] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      sym_identifier,
    ACTIONS(426), 1,
      anon_sym_RBRACK,
    STATE(226), 1,
      sym_instance_block,
  [2466] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(428), 3,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_GT,
  [2475] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(339), 1,
      sym_identifier,
    ACTIONS(430), 1,
      anon_sym_RBRACK,
    STATE(196), 1,
      sym_field_definition,
  [2488] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(432), 1,
      anon_sym_COMMA,
    ACTIONS(435), 1,
      anon_sym_RBRACK,
    STATE(113), 1,
      aux_sym_fields_block_repeat1,
  [2501] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(368), 1,
      sym_identifier,
    ACTIONS(437), 1,
      anon_sym_RBRACK,
    STATE(97), 1,
      sym_hub_definition,
  [2514] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(439), 1,
      anon_sym_COMMA,
    ACTIONS(441), 1,
      anon_sym_RBRACE,
    STATE(148), 1,
      aux_sym_import_statement_repeat1,
  [2527] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(353), 1,
      sym_identifier,
    ACTIONS(443), 1,
      anon_sym_RBRACK,
    STATE(99), 1,
      sym_struct_definition,
  [2540] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(343), 1,
      sym_identifier,
    ACTIONS(445), 1,
      anon_sym_RBRACK,
    STATE(100), 1,
      sym_enum_definition,
  [2553] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(343), 1,
      sym_identifier,
    ACTIONS(447), 1,
      anon_sym_RBRACK,
    STATE(193), 1,
      sym_enum_definition,
  [2566] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(449), 1,
      anon_sym_COMMA,
    ACTIONS(452), 1,
      anon_sym_RBRACK,
    STATE(119), 1,
      aux_sym_enums_block_repeat1,
  [2579] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(339), 1,
      sym_identifier,
    ACTIONS(454), 1,
      anon_sym_RBRACK,
    STATE(102), 1,
      sym_field_definition,
  [2592] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(456), 1,
      anon_sym_COMMA,
    ACTIONS(458), 1,
      anon_sym_RBRACE,
    STATE(149), 1,
      aux_sym_import_statement_repeat1,
  [2605] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(460), 1,
      anon_sym_COMMA,
    ACTIONS(463), 1,
      anon_sym_RBRACE,
    STATE(122), 1,
      aux_sym_hub_definition_repeat1,
  [2618] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(465), 1,
      anon_sym_COMMA,
    ACTIONS(467), 1,
      anon_sym_RBRACK,
    STATE(103), 1,
      aux_sym_imports_section_repeat1,
  [2631] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(353), 1,
      sym_identifier,
    ACTIONS(469), 1,
      anon_sym_RBRACK,
    STATE(188), 1,
      sym_struct_definition,
  [2644] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(471), 1,
      anon_sym_COMMA,
    ACTIONS(474), 1,
      anon_sym_RBRACK,
    STATE(125), 1,
      aux_sym_structs_block_repeat1,
  [2657] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(476), 1,
      anon_sym_STAR,
    ACTIONS(478), 1,
      sym_number,
    STATE(257), 1,
      sym_multiplicity,
  [2670] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(304), 1,
      sym_identifier,
    STATE(109), 1,
      sym_generic_type,
    STATE(228), 1,
      sym_type,
  [2683] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(480), 1,
      anon_sym_COMMA,
    ACTIONS(482), 1,
      anon_sym_RBRACE,
    STATE(152), 1,
      aux_sym_hub_definition_repeat1,
  [2696] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(484), 1,
      anon_sym_COMMA,
    ACTIONS(486), 1,
      anon_sym_GT,
    STATE(98), 1,
      aux_sym_generic_type_repeat1,
  [2709] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      anon_sym_LBRACK,
    ACTIONS(467), 1,
      anon_sym_RBRACK,
    STATE(242), 1,
      sym_import_statement,
  [2722] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(368), 1,
      sym_identifier,
    ACTIONS(488), 1,
      anon_sym_RBRACK,
    STATE(168), 1,
      sym_hub_definition,
  [2735] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(490), 1,
      anon_sym_COMMA,
    ACTIONS(493), 1,
      anon_sym_RBRACK,
    STATE(132), 1,
      aux_sym_hubs_block_repeat1,
  [2748] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(495), 1,
      anon_sym_COMMA,
    ACTIONS(497), 1,
      anon_sym_RBRACK,
    STATE(107), 1,
      aux_sym_import_statement_repeat1,
  [2761] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(499), 1,
      ts_builtin_sym_end,
    ACTIONS(501), 1,
      anon_sym_COMMA,
    STATE(134), 1,
      aux_sym_document_repeat1,
  [2774] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(504), 1,
      anon_sym_COMMA,
    ACTIONS(506), 1,
      anon_sym_RBRACE,
    STATE(140), 1,
      aux_sym_metadata_block_repeat1,
  [2787] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(508), 1,
      anon_sym_COMMA,
    ACTIONS(510), 1,
      anon_sym_RBRACE,
    STATE(146), 1,
      aux_sym_instance_block_repeat1,
  [2800] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(512), 1,
      anon_sym_COMMA,
    ACTIONS(514), 1,
      anon_sym_RBRACK,
    STATE(108), 1,
      aux_sym_instances_section_repeat1,
  [2813] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(516), 1,
      anon_sym_COMMA,
    ACTIONS(518), 1,
      anon_sym_RBRACK,
    STATE(79), 1,
      aux_sym_definitions_section_repeat1,
  [2826] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(163), 1,
      anon_sym_RBRACK,
    ACTIONS(520), 1,
      anon_sym_COMMA,
    STATE(139), 1,
      aux_sym_array_repeat1,
  [2839] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(523), 1,
      anon_sym_COMMA,
    ACTIONS(526), 1,
      anon_sym_RBRACE,
    STATE(140), 1,
      aux_sym_metadata_block_repeat1,
  [2852] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(528), 1,
      anon_sym_COMMA,
    ACTIONS(530), 1,
      anon_sym_RBRACK,
    STATE(123), 1,
      aux_sym_imports_section_repeat1,
  [2865] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(199), 1,
      ts_builtin_sym_end,
    ACTIONS(532), 1,
      anon_sym_COMMA,
    STATE(134), 1,
      aux_sym_document_repeat1,
  [2878] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      sym_identifier,
    ACTIONS(534), 1,
      anon_sym_RBRACK,
    STATE(137), 1,
      sym_instance_block,
  [2891] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      anon_sym_LBRACK,
    ACTIONS(536), 1,
      anon_sym_RBRACK,
    STATE(141), 1,
      sym_import_statement,
  [2904] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(304), 1,
      sym_identifier,
    STATE(109), 1,
      sym_generic_type,
    STATE(129), 1,
      sym_type,
  [2917] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(239), 1,
      anon_sym_RBRACE,
    ACTIONS(538), 1,
      anon_sym_COMMA,
    STATE(106), 1,
      aux_sym_instance_block_repeat1,
  [2930] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(540), 1,
      ts_builtin_sym_end,
    ACTIONS(542), 1,
      anon_sym_COMMA,
    STATE(142), 1,
      aux_sym_document_repeat1,
  [2943] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(544), 1,
      anon_sym_COMMA,
    ACTIONS(546), 1,
      anon_sym_RBRACE,
    STATE(75), 1,
      aux_sym_import_statement_repeat1,
  [2956] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(548), 1,
      anon_sym_COMMA,
    ACTIONS(550), 1,
      anon_sym_RBRACE,
    STATE(75), 1,
      aux_sym_import_statement_repeat1,
  [2969] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(183), 1,
      sym_decorator,
    ACTIONS(552), 2,
      anon_sym_ATcomputed,
      anon_sym_ATdefault,
  [2980] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(554), 1,
      anon_sym_COMMA,
    ACTIONS(556), 1,
      anon_sym_RBRACK,
    STATE(84), 1,
      aux_sym_import_statement_repeat1,
  [2993] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(327), 1,
      anon_sym_RBRACE,
    ACTIONS(558), 1,
      anon_sym_COMMA,
    STATE(122), 1,
      aux_sym_hub_definition_repeat1,
  [3006] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(560), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3014] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(562), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3022] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(564), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3030] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(566), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3038] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(353), 1,
      sym_identifier,
    STATE(188), 1,
      sym_struct_definition,
  [3048] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(546), 1,
      anon_sym_RBRACE,
    ACTIONS(568), 1,
      sym_identifier,
  [3058] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(570), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3066] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      sym_identifier,
    ACTIONS(572), 1,
      anon_sym_RBRACK,
  [3076] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(574), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3084] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(576), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3092] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(343), 1,
      sym_identifier,
    STATE(193), 1,
      sym_enum_definition,
  [3102] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(578), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3110] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(368), 1,
      sym_identifier,
    STATE(168), 1,
      sym_hub_definition,
  [3120] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(580), 1,
      anon_sym_RBRACE,
    ACTIONS(582), 1,
      sym_identifier,
  [3130] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(550), 1,
      anon_sym_RBRACE,
    ACTIONS(568), 1,
      sym_identifier,
  [3140] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(493), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3148] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(584), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3156] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(339), 1,
      sym_identifier,
    STATE(196), 1,
      sym_field_definition,
  [3166] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(586), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3174] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(588), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3182] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(590), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3190] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(592), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3198] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(594), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3206] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(596), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3214] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(347), 1,
      anon_sym_RBRACK,
    ACTIONS(568), 1,
      sym_identifier,
  [3224] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(598), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3232] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      sym_identifier,
    ACTIONS(600), 1,
      anon_sym_RBRACE,
  [3242] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(602), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3250] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      sym_identifier,
    ACTIONS(604), 1,
      anon_sym_RBRACE,
  [3260] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(606), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3268] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(608), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3276] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(610), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3284] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(612), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3292] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(463), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3300] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(614), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3308] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(474), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3316] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(616), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3324] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(618), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3332] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(620), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3340] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(622), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3348] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(452), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3356] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(624), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3364] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(626), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3372] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(435), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3380] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(628), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3388] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(418), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3396] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(630), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3404] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(632), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3412] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(634), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3420] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(636), 1,
      anon_sym_RBRACE,
    ACTIONS(638), 1,
      sym_identifier,
  [3430] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(640), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3438] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(642), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3446] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(644), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3454] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      sym_identifier,
    STATE(226), 1,
      sym_instance_block,
  [3464] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(646), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3472] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(648), 1,
      anon_sym_RPAREN,
    ACTIONS(650), 1,
      anon_sym_DOT_DOT,
  [3482] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(652), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3490] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(654), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3498] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(656), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3506] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(658), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3514] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(660), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3522] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(662), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3530] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(664), 1,
      anon_sym_RBRACE,
    ACTIONS(666), 1,
      sym_identifier,
  [3540] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(668), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3548] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(670), 1,
      anon_sym_RBRACE,
    ACTIONS(672), 1,
      sym_identifier,
  [3558] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(499), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3566] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(674), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3574] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(411), 1,
      anon_sym_LBRACK,
    STATE(242), 1,
      sym_import_statement,
  [3584] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(676), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3592] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(678), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3600] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      sym_identifier,
    ACTIONS(680), 1,
      anon_sym_RBRACK,
  [3610] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(682), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3618] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(684), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3626] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(358), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3634] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(686), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3642] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(382), 2,
      anon_sym_COMMA,
      anon_sym_GT,
  [3650] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(506), 1,
      anon_sym_RBRACE,
    ACTIONS(638), 1,
      sym_identifier,
  [3660] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(377), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3668] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3676] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(690), 2,
      anon_sym_STAR,
      sym_number,
  [3684] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(692), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [3692] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(694), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3700] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(696), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3708] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(698), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3716] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(700), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3724] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(422), 1,
      anon_sym_RBRACK,
    ACTIONS(568), 1,
      sym_identifier,
  [3734] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(702), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3742] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(704), 2,
      ts_builtin_sym_end,
      anon_sym_COMMA,
  [3750] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(706), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3758] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(407), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [3766] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(708), 1,
      anon_sym_COLON,
  [3773] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(710), 1,
      anon_sym_FROM,
  [3780] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(712), 1,
      anon_sym_LBRACE,
  [3787] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(714), 1,
      anon_sym_LBRACE,
  [3794] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(716), 1,
      sym_identifier,
  [3801] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(718), 1,
      anon_sym_LBRACE,
  [3808] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(720), 1,
      anon_sym_ALLOWS,
  [3815] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(722), 1,
      anon_sym_RPAREN,
  [3822] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(724), 1,
      anon_sym_LBRACK,
  [3829] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(726), 1,
      anon_sym_LBRACE,
  [3836] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(728), 1,
      anon_sym_FROM,
  [3843] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(730), 1,
      sym_identifier,
  [3850] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(732), 1,
      anon_sym_LBRACE,
  [3857] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(734), 1,
      anon_sym_COLON,
  [3864] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(736), 1,
      anon_sym_RPAREN,
  [3871] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(738), 1,
      anon_sym_LBRACK,
  [3878] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(740), 1,
      anon_sym_LBRACK,
  [3885] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(742), 1,
      sym_identifier,
  [3892] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(648), 1,
      anon_sym_RPAREN,
  [3899] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(744), 1,
      anon_sym_EQ,
  [3906] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(746), 1,
      anon_sym_FROM,
  [3913] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(748), 1,
      anon_sym_LBRACK,
  [3920] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(750), 1,
      anon_sym_LBRACK,
  [3927] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      sym_identifier,
  [3934] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(752), 1,
      anon_sym_EQ,
  [3941] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(754), 1,
      sym_identifier,
  [3948] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(756), 1,
      ts_builtin_sym_end,
  [3955] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(758), 1,
      anon_sym_LPAREN,
  [3962] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(638), 1,
      sym_identifier,
  [3969] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(760), 1,
      anon_sym_LBRACK,
  [3976] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(762), 1,
      anon_sym_LBRACE,
  [3983] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(764), 1,
      anon_sym_LBRACK,
  [3990] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(766), 1,
      anon_sym_LBRACK,
  [3997] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(768), 1,
      anon_sym_EQ,
  [4004] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(770), 1,
      anon_sym_LPAREN,
  [4011] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(772), 1,
      anon_sym_LPAREN,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 55,
  [SMALL_STATE(4)] = 110,
  [SMALL_STATE(5)] = 165,
  [SMALL_STATE(6)] = 217,
  [SMALL_STATE(7)] = 269,
  [SMALL_STATE(8)] = 321,
  [SMALL_STATE(9)] = 373,
  [SMALL_STATE(10)] = 425,
  [SMALL_STATE(11)] = 477,
  [SMALL_STATE(12)] = 529,
  [SMALL_STATE(13)] = 581,
  [SMALL_STATE(14)] = 633,
  [SMALL_STATE(15)] = 685,
  [SMALL_STATE(16)] = 737,
  [SMALL_STATE(17)] = 789,
  [SMALL_STATE(18)] = 810,
  [SMALL_STATE(19)] = 831,
  [SMALL_STATE(20)] = 854,
  [SMALL_STATE(21)] = 875,
  [SMALL_STATE(22)] = 902,
  [SMALL_STATE(23)] = 923,
  [SMALL_STATE(24)] = 944,
  [SMALL_STATE(25)] = 965,
  [SMALL_STATE(26)] = 990,
  [SMALL_STATE(27)] = 1013,
  [SMALL_STATE(28)] = 1034,
  [SMALL_STATE(29)] = 1063,
  [SMALL_STATE(30)] = 1084,
  [SMALL_STATE(31)] = 1105,
  [SMALL_STATE(32)] = 1126,
  [SMALL_STATE(33)] = 1147,
  [SMALL_STATE(34)] = 1181,
  [SMALL_STATE(35)] = 1215,
  [SMALL_STATE(36)] = 1244,
  [SMALL_STATE(37)] = 1273,
  [SMALL_STATE(38)] = 1302,
  [SMALL_STATE(39)] = 1330,
  [SMALL_STATE(40)] = 1358,
  [SMALL_STATE(41)] = 1386,
  [SMALL_STATE(42)] = 1411,
  [SMALL_STATE(43)] = 1436,
  [SMALL_STATE(44)] = 1461,
  [SMALL_STATE(45)] = 1483,
  [SMALL_STATE(46)] = 1505,
  [SMALL_STATE(47)] = 1526,
  [SMALL_STATE(48)] = 1547,
  [SMALL_STATE(49)] = 1565,
  [SMALL_STATE(50)] = 1583,
  [SMALL_STATE(51)] = 1601,
  [SMALL_STATE(52)] = 1619,
  [SMALL_STATE(53)] = 1637,
  [SMALL_STATE(54)] = 1655,
  [SMALL_STATE(55)] = 1673,
  [SMALL_STATE(56)] = 1692,
  [SMALL_STATE(57)] = 1711,
  [SMALL_STATE(58)] = 1730,
  [SMALL_STATE(59)] = 1749,
  [SMALL_STATE(60)] = 1768,
  [SMALL_STATE(61)] = 1787,
  [SMALL_STATE(62)] = 1806,
  [SMALL_STATE(63)] = 1825,
  [SMALL_STATE(64)] = 1844,
  [SMALL_STATE(65)] = 1860,
  [SMALL_STATE(66)] = 1872,
  [SMALL_STATE(67)] = 1886,
  [SMALL_STATE(68)] = 1902,
  [SMALL_STATE(69)] = 1914,
  [SMALL_STATE(70)] = 1930,
  [SMALL_STATE(71)] = 1946,
  [SMALL_STATE(72)] = 1962,
  [SMALL_STATE(73)] = 1978,
  [SMALL_STATE(74)] = 1994,
  [SMALL_STATE(75)] = 2008,
  [SMALL_STATE(76)] = 2022,
  [SMALL_STATE(77)] = 2038,
  [SMALL_STATE(78)] = 2052,
  [SMALL_STATE(79)] = 2068,
  [SMALL_STATE(80)] = 2081,
  [SMALL_STATE(81)] = 2094,
  [SMALL_STATE(82)] = 2107,
  [SMALL_STATE(83)] = 2120,
  [SMALL_STATE(84)] = 2133,
  [SMALL_STATE(85)] = 2146,
  [SMALL_STATE(86)] = 2159,
  [SMALL_STATE(87)] = 2168,
  [SMALL_STATE(88)] = 2181,
  [SMALL_STATE(89)] = 2194,
  [SMALL_STATE(90)] = 2207,
  [SMALL_STATE(91)] = 2220,
  [SMALL_STATE(92)] = 2233,
  [SMALL_STATE(93)] = 2242,
  [SMALL_STATE(94)] = 2255,
  [SMALL_STATE(95)] = 2266,
  [SMALL_STATE(96)] = 2279,
  [SMALL_STATE(97)] = 2292,
  [SMALL_STATE(98)] = 2305,
  [SMALL_STATE(99)] = 2318,
  [SMALL_STATE(100)] = 2331,
  [SMALL_STATE(101)] = 2344,
  [SMALL_STATE(102)] = 2357,
  [SMALL_STATE(103)] = 2370,
  [SMALL_STATE(104)] = 2383,
  [SMALL_STATE(105)] = 2392,
  [SMALL_STATE(106)] = 2405,
  [SMALL_STATE(107)] = 2418,
  [SMALL_STATE(108)] = 2431,
  [SMALL_STATE(109)] = 2444,
  [SMALL_STATE(110)] = 2453,
  [SMALL_STATE(111)] = 2466,
  [SMALL_STATE(112)] = 2475,
  [SMALL_STATE(113)] = 2488,
  [SMALL_STATE(114)] = 2501,
  [SMALL_STATE(115)] = 2514,
  [SMALL_STATE(116)] = 2527,
  [SMALL_STATE(117)] = 2540,
  [SMALL_STATE(118)] = 2553,
  [SMALL_STATE(119)] = 2566,
  [SMALL_STATE(120)] = 2579,
  [SMALL_STATE(121)] = 2592,
  [SMALL_STATE(122)] = 2605,
  [SMALL_STATE(123)] = 2618,
  [SMALL_STATE(124)] = 2631,
  [SMALL_STATE(125)] = 2644,
  [SMALL_STATE(126)] = 2657,
  [SMALL_STATE(127)] = 2670,
  [SMALL_STATE(128)] = 2683,
  [SMALL_STATE(129)] = 2696,
  [SMALL_STATE(130)] = 2709,
  [SMALL_STATE(131)] = 2722,
  [SMALL_STATE(132)] = 2735,
  [SMALL_STATE(133)] = 2748,
  [SMALL_STATE(134)] = 2761,
  [SMALL_STATE(135)] = 2774,
  [SMALL_STATE(136)] = 2787,
  [SMALL_STATE(137)] = 2800,
  [SMALL_STATE(138)] = 2813,
  [SMALL_STATE(139)] = 2826,
  [SMALL_STATE(140)] = 2839,
  [SMALL_STATE(141)] = 2852,
  [SMALL_STATE(142)] = 2865,
  [SMALL_STATE(143)] = 2878,
  [SMALL_STATE(144)] = 2891,
  [SMALL_STATE(145)] = 2904,
  [SMALL_STATE(146)] = 2917,
  [SMALL_STATE(147)] = 2930,
  [SMALL_STATE(148)] = 2943,
  [SMALL_STATE(149)] = 2956,
  [SMALL_STATE(150)] = 2969,
  [SMALL_STATE(151)] = 2980,
  [SMALL_STATE(152)] = 2993,
  [SMALL_STATE(153)] = 3006,
  [SMALL_STATE(154)] = 3014,
  [SMALL_STATE(155)] = 3022,
  [SMALL_STATE(156)] = 3030,
  [SMALL_STATE(157)] = 3038,
  [SMALL_STATE(158)] = 3048,
  [SMALL_STATE(159)] = 3058,
  [SMALL_STATE(160)] = 3066,
  [SMALL_STATE(161)] = 3076,
  [SMALL_STATE(162)] = 3084,
  [SMALL_STATE(163)] = 3092,
  [SMALL_STATE(164)] = 3102,
  [SMALL_STATE(165)] = 3110,
  [SMALL_STATE(166)] = 3120,
  [SMALL_STATE(167)] = 3130,
  [SMALL_STATE(168)] = 3140,
  [SMALL_STATE(169)] = 3148,
  [SMALL_STATE(170)] = 3156,
  [SMALL_STATE(171)] = 3166,
  [SMALL_STATE(172)] = 3174,
  [SMALL_STATE(173)] = 3182,
  [SMALL_STATE(174)] = 3190,
  [SMALL_STATE(175)] = 3198,
  [SMALL_STATE(176)] = 3206,
  [SMALL_STATE(177)] = 3214,
  [SMALL_STATE(178)] = 3224,
  [SMALL_STATE(179)] = 3232,
  [SMALL_STATE(180)] = 3242,
  [SMALL_STATE(181)] = 3250,
  [SMALL_STATE(182)] = 3260,
  [SMALL_STATE(183)] = 3268,
  [SMALL_STATE(184)] = 3276,
  [SMALL_STATE(185)] = 3284,
  [SMALL_STATE(186)] = 3292,
  [SMALL_STATE(187)] = 3300,
  [SMALL_STATE(188)] = 3308,
  [SMALL_STATE(189)] = 3316,
  [SMALL_STATE(190)] = 3324,
  [SMALL_STATE(191)] = 3332,
  [SMALL_STATE(192)] = 3340,
  [SMALL_STATE(193)] = 3348,
  [SMALL_STATE(194)] = 3356,
  [SMALL_STATE(195)] = 3364,
  [SMALL_STATE(196)] = 3372,
  [SMALL_STATE(197)] = 3380,
  [SMALL_STATE(198)] = 3388,
  [SMALL_STATE(199)] = 3396,
  [SMALL_STATE(200)] = 3404,
  [SMALL_STATE(201)] = 3412,
  [SMALL_STATE(202)] = 3420,
  [SMALL_STATE(203)] = 3430,
  [SMALL_STATE(204)] = 3438,
  [SMALL_STATE(205)] = 3446,
  [SMALL_STATE(206)] = 3454,
  [SMALL_STATE(207)] = 3464,
  [SMALL_STATE(208)] = 3472,
  [SMALL_STATE(209)] = 3482,
  [SMALL_STATE(210)] = 3490,
  [SMALL_STATE(211)] = 3498,
  [SMALL_STATE(212)] = 3506,
  [SMALL_STATE(213)] = 3514,
  [SMALL_STATE(214)] = 3522,
  [SMALL_STATE(215)] = 3530,
  [SMALL_STATE(216)] = 3540,
  [SMALL_STATE(217)] = 3548,
  [SMALL_STATE(218)] = 3558,
  [SMALL_STATE(219)] = 3566,
  [SMALL_STATE(220)] = 3574,
  [SMALL_STATE(221)] = 3584,
  [SMALL_STATE(222)] = 3592,
  [SMALL_STATE(223)] = 3600,
  [SMALL_STATE(224)] = 3610,
  [SMALL_STATE(225)] = 3618,
  [SMALL_STATE(226)] = 3626,
  [SMALL_STATE(227)] = 3634,
  [SMALL_STATE(228)] = 3642,
  [SMALL_STATE(229)] = 3650,
  [SMALL_STATE(230)] = 3660,
  [SMALL_STATE(231)] = 3668,
  [SMALL_STATE(232)] = 3676,
  [SMALL_STATE(233)] = 3684,
  [SMALL_STATE(234)] = 3692,
  [SMALL_STATE(235)] = 3700,
  [SMALL_STATE(236)] = 3708,
  [SMALL_STATE(237)] = 3716,
  [SMALL_STATE(238)] = 3724,
  [SMALL_STATE(239)] = 3734,
  [SMALL_STATE(240)] = 3742,
  [SMALL_STATE(241)] = 3750,
  [SMALL_STATE(242)] = 3758,
  [SMALL_STATE(243)] = 3766,
  [SMALL_STATE(244)] = 3773,
  [SMALL_STATE(245)] = 3780,
  [SMALL_STATE(246)] = 3787,
  [SMALL_STATE(247)] = 3794,
  [SMALL_STATE(248)] = 3801,
  [SMALL_STATE(249)] = 3808,
  [SMALL_STATE(250)] = 3815,
  [SMALL_STATE(251)] = 3822,
  [SMALL_STATE(252)] = 3829,
  [SMALL_STATE(253)] = 3836,
  [SMALL_STATE(254)] = 3843,
  [SMALL_STATE(255)] = 3850,
  [SMALL_STATE(256)] = 3857,
  [SMALL_STATE(257)] = 3864,
  [SMALL_STATE(258)] = 3871,
  [SMALL_STATE(259)] = 3878,
  [SMALL_STATE(260)] = 3885,
  [SMALL_STATE(261)] = 3892,
  [SMALL_STATE(262)] = 3899,
  [SMALL_STATE(263)] = 3906,
  [SMALL_STATE(264)] = 3913,
  [SMALL_STATE(265)] = 3920,
  [SMALL_STATE(266)] = 3927,
  [SMALL_STATE(267)] = 3934,
  [SMALL_STATE(268)] = 3941,
  [SMALL_STATE(269)] = 3948,
  [SMALL_STATE(270)] = 3955,
  [SMALL_STATE(271)] = 3962,
  [SMALL_STATE(272)] = 3969,
  [SMALL_STATE(273)] = 3976,
  [SMALL_STATE(274)] = 3983,
  [SMALL_STATE(275)] = 3990,
  [SMALL_STATE(276)] = 3997,
  [SMALL_STATE(277)] = 4004,
  [SMALL_STATE(278)] = 4011,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(272),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(275),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(274),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(78),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(70),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(37),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [65] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [67] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [71] = {.entry = {.count = 1, .reusable = false}}, SHIFT(40),
  [73] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [75] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [79] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [83] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [85] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [87] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [89] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [91] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 4, 0, 0),
  [93] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 4, 0, 0),
  [95] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 2, 0, 0),
  [97] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 2, 0, 0),
  [99] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_expression, 2, 0, 2),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(254),
  [103] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_expression, 2, 0, 2),
  [105] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 3, 0, 0),
  [107] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 3, 0, 0),
  [109] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_expression, 3, 0, 3),
  [111] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [115] = {.entry = {.count = 1, .reusable = false}}, SHIFT(9),
  [117] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3, 0, 0),
  [119] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 3, 0, 0),
  [121] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parenthesized_expression, 3, 0, 0),
  [123] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_parenthesized_expression, 3, 0, 0),
  [125] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template_string, 3, 0, 0),
  [127] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_template_string, 3, 0, 0),
  [129] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_expression, 3, 0, 3),
  [131] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_member_expression, 3, 0, 4),
  [133] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_member_expression, 3, 0, 4),
  [135] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template_string, 2, 0, 0),
  [139] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_template_string, 2, 0, 0),
  [141] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 5, 0, 0),
  [143] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 5, 0, 0),
  [145] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1, 0, 0),
  [147] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1, 0, 0),
  [149] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 2, 0, 0),
  [151] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 2, 0, 0),
  [153] = {.entry = {.count = 1, .reusable = true}}, SHIFT(229),
  [155] = {.entry = {.count = 1, .reusable = true}}, SHIFT(224),
  [157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [163] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0),
  [165] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_metadata_block_repeat1, 4, 0, 0),
  [167] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_assignment, 3, 0, 0),
  [169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(233),
  [171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [173] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(201),
  [177] = {.entry = {.count = 1, .reusable = true}}, SHIFT(265),
  [179] = {.entry = {.count = 1, .reusable = true}}, SHIFT(264),
  [181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(259),
  [183] = {.entry = {.count = 1, .reusable = true}}, SHIFT(258),
  [185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(231),
  [187] = {.entry = {.count = 1, .reusable = true}}, SHIFT(212),
  [189] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_field, 1, 0, 0),
  [191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(150),
  [193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(278),
  [195] = {.entry = {.count = 1, .reusable = false}}, SHIFT(278),
  [197] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 3, 0, 0),
  [199] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 2, 0, 0),
  [201] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [203] = {.entry = {.count = 1, .reusable = false}}, SHIFT(54),
  [205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [207] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [209] = {.entry = {.count = 1, .reusable = false}}, SHIFT(52),
  [211] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [213] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat3, 2, 0, 0),
  [215] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat3, 2, 0, 0), SHIFT_REPEAT(50),
  [218] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_repeat3, 2, 0, 0), SHIFT_REPEAT(50),
  [221] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [223] = {.entry = {.count = 1, .reusable = false}}, SHIFT(53),
  [225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [227] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat4, 2, 0, 0),
  [229] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat4, 2, 0, 0), SHIFT_REPEAT(53),
  [232] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_repeat4, 2, 0, 0), SHIFT_REPEAT(53),
  [235] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [237] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [239] = {.entry = {.count = 1, .reusable = true}}, SHIFT(197),
  [241] = {.entry = {.count = 1, .reusable = true}}, SHIFT(273),
  [243] = {.entry = {.count = 1, .reusable = true}}, SHIFT(276),
  [245] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [247] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [249] = {.entry = {.count = 1, .reusable = false}}, SHIFT(255),
  [251] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [253] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_template_string_repeat1, 2, 0, 0),
  [255] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_string_repeat1, 2, 0, 0), SHIFT_REPEAT(59),
  [258] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_string_repeat1, 2, 0, 0), SHIFT_REPEAT(255),
  [261] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_string_repeat1, 2, 0, 0), SHIFT_REPEAT(59),
  [264] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [266] = {.entry = {.count = 1, .reusable = false}}, SHIFT(57),
  [268] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [270] = {.entry = {.count = 1, .reusable = true}}, SHIFT(225),
  [272] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [274] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat2, 2, 0, 0),
  [276] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat2, 2, 0, 0), SHIFT_REPEAT(64),
  [279] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_repeat2, 2, 0, 0), SHIFT_REPEAT(64),
  [282] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 1, 0, 0),
  [284] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [286] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [288] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [290] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_template_string_repeat1, 4, 0, 0),
  [292] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_template_string_repeat1, 4, 0, 0),
  [294] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [296] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [298] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [300] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [302] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [304] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [306] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [308] = {.entry = {.count = 1, .reusable = false}}, SHIFT(76),
  [310] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [312] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [314] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_import_statement_repeat1, 2, 0, 0), SHIFT_REPEAT(266),
  [317] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_import_statement_repeat1, 2, 0, 0),
  [319] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0),
  [321] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0), SHIFT_REPEAT(76),
  [324] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0), SHIFT_REPEAT(76),
  [327] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [329] = {.entry = {.count = 1, .reusable = false}}, SHIFT(73),
  [331] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [333] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [335] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [337] = {.entry = {.count = 1, .reusable = true}}, SHIFT(199),
  [339] = {.entry = {.count = 1, .reusable = true}}, SHIFT(243),
  [341] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [343] = {.entry = {.count = 1, .reusable = true}}, SHIFT(245),
  [345] = {.entry = {.count = 1, .reusable = true}}, SHIFT(160),
  [347] = {.entry = {.count = 1, .reusable = true}}, SHIFT(173),
  [349] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [351] = {.entry = {.count = 1, .reusable = true}}, SHIFT(191),
  [353] = {.entry = {.count = 1, .reusable = true}}, SHIFT(246),
  [355] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_instances_section_repeat1, 2, 0, 0), SHIFT_REPEAT(206),
  [358] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_instances_section_repeat1, 2, 0, 0),
  [360] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [362] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [364] = {.entry = {.count = 1, .reusable = true}}, SHIFT(256),
  [366] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [368] = {.entry = {.count = 1, .reusable = true}}, SHIFT(248),
  [370] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_generic_type, 5, 0, 0),
  [372] = {.entry = {.count = 1, .reusable = true}}, SHIFT(131),
  [374] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_definitions_section_repeat1, 2, 0, 0), SHIFT_REPEAT(44),
  [377] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_definitions_section_repeat1, 2, 0, 0),
  [379] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_generic_type_repeat1, 2, 0, 0), SHIFT_REPEAT(127),
  [382] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_generic_type_repeat1, 2, 0, 0),
  [384] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [386] = {.entry = {.count = 1, .reusable = true}}, SHIFT(213),
  [388] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [390] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [392] = {.entry = {.count = 1, .reusable = true}}, SHIFT(214),
  [394] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [396] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [398] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [400] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [402] = {.entry = {.count = 1, .reusable = true}}, SHIFT(219),
  [404] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_imports_section_repeat1, 2, 0, 0), SHIFT_REPEAT(220),
  [407] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_imports_section_repeat1, 2, 0, 0),
  [409] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_generic_type, 4, 0, 0),
  [411] = {.entry = {.count = 1, .reusable = true}}, SHIFT(268),
  [413] = {.entry = {.count = 1, .reusable = true}}, SHIFT(221),
  [415] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_instance_block_repeat1, 2, 0, 0), SHIFT_REPEAT(67),
  [418] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_instance_block_repeat1, 2, 0, 0),
  [420] = {.entry = {.count = 1, .reusable = true}}, SHIFT(223),
  [422] = {.entry = {.count = 1, .reusable = true}}, SHIFT(253),
  [424] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [426] = {.entry = {.count = 1, .reusable = true}}, SHIFT(227),
  [428] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_generic_type, 6, 0, 0),
  [430] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [432] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fields_block_repeat1, 2, 0, 0), SHIFT_REPEAT(170),
  [435] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_fields_block_repeat1, 2, 0, 0),
  [437] = {.entry = {.count = 1, .reusable = true}}, SHIFT(235),
  [439] = {.entry = {.count = 1, .reusable = true}}, SHIFT(158),
  [441] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [443] = {.entry = {.count = 1, .reusable = true}}, SHIFT(237),
  [445] = {.entry = {.count = 1, .reusable = true}}, SHIFT(239),
  [447] = {.entry = {.count = 1, .reusable = true}}, SHIFT(161),
  [449] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_enums_block_repeat1, 2, 0, 0), SHIFT_REPEAT(163),
  [452] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_enums_block_repeat1, 2, 0, 0),
  [454] = {.entry = {.count = 1, .reusable = true}}, SHIFT(241),
  [456] = {.entry = {.count = 1, .reusable = true}}, SHIFT(167),
  [458] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [460] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_hub_definition_repeat1, 2, 0, 0), SHIFT_REPEAT(94),
  [463] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_hub_definition_repeat1, 2, 0, 0),
  [465] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [467] = {.entry = {.count = 1, .reusable = true}}, SHIFT(240),
  [469] = {.entry = {.count = 1, .reusable = true}}, SHIFT(156),
  [471] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_structs_block_repeat1, 2, 0, 0), SHIFT_REPEAT(157),
  [474] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_structs_block_repeat1, 2, 0, 0),
  [476] = {.entry = {.count = 1, .reusable = true}}, SHIFT(261),
  [478] = {.entry = {.count = 1, .reusable = true}}, SHIFT(208),
  [480] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [482] = {.entry = {.count = 1, .reusable = true}}, SHIFT(162),
  [484] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [486] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(164),
  [490] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_hubs_block_repeat1, 2, 0, 0), SHIFT_REPEAT(165),
  [493] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_hubs_block_repeat1, 2, 0, 0),
  [495] = {.entry = {.count = 1, .reusable = true}}, SHIFT(238),
  [497] = {.entry = {.count = 1, .reusable = true}}, SHIFT(244),
  [499] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2, 0, 0),
  [501] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2, 0, 0), SHIFT_REPEAT(51),
  [504] = {.entry = {.count = 1, .reusable = true}}, SHIFT(202),
  [506] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [508] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [510] = {.entry = {.count = 1, .reusable = true}}, SHIFT(169),
  [512] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [514] = {.entry = {.count = 1, .reusable = true}}, SHIFT(236),
  [516] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [518] = {.entry = {.count = 1, .reusable = true}}, SHIFT(234),
  [520] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(12),
  [523] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_metadata_block_repeat1, 2, 0, 0), SHIFT_REPEAT(271),
  [526] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_metadata_block_repeat1, 2, 0, 0),
  [528] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [530] = {.entry = {.count = 1, .reusable = true}}, SHIFT(222),
  [532] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [534] = {.entry = {.count = 1, .reusable = true}}, SHIFT(209),
  [536] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [538] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [540] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 1, 0, 0),
  [542] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [544] = {.entry = {.count = 1, .reusable = true}}, SHIFT(179),
  [546] = {.entry = {.count = 1, .reusable = true}}, SHIFT(178),
  [548] = {.entry = {.count = 1, .reusable = true}}, SHIFT(181),
  [550] = {.entry = {.count = 1, .reusable = true}}, SHIFT(180),
  [552] = {.entry = {.count = 1, .reusable = true}}, SHIFT(270),
  [554] = {.entry = {.count = 1, .reusable = true}}, SHIFT(177),
  [556] = {.entry = {.count = 1, .reusable = true}}, SHIFT(174),
  [558] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [560] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_assignment, 1, 0, 0),
  [562] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enums_block, 4, 0, 0),
  [564] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_definition, 4, 0, 0),
  [566] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_structs_block, 6, 0, 0),
  [568] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [570] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_role, 11, 0, 0),
  [572] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [574] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enums_block, 6, 0, 0),
  [576] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_definition, 4, 0, 0),
  [578] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hubs_block, 6, 0, 0),
  [580] = {.entry = {.count = 1, .reusable = true}}, SHIFT(189),
  [582] = {.entry = {.count = 1, .reusable = true}}, SHIFT(267),
  [584] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_block, 6, 0, 1),
  [586] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fields_block, 6, 0, 0),
  [588] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_import_statement, 6, 0, 0),
  [590] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_role, 10, 0, 0),
  [592] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_role, 9, 0, 0),
  [594] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_block, 5, 0, 1),
  [596] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_import_statement, 7, 0, 0),
  [598] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_definition, 5, 0, 0),
  [600] = {.entry = {.count = 1, .reusable = true}}, SHIFT(204),
  [602] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_definition, 5, 0, 0),
  [604] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [606] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata_block, 8, 0, 0),
  [608] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_field, 3, 0, 0),
  [610] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hubs_block, 5, 0, 0),
  [612] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_definition, 5, 0, 0),
  [614] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_definition, 3, 0, 0),
  [616] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata_block, 3, 0, 0),
  [618] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_imports_section, 3, 0, 0),
  [620] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_structs_block, 5, 0, 0),
  [622] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_definition, 3, 0, 0),
  [624] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enums_block, 5, 0, 0),
  [626] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_definition, 3, 0, 0),
  [628] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_block, 7, 0, 1),
  [630] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fields_block, 5, 0, 0),
  [632] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_definition, 3, 0, 0),
  [634] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_definitions_section, 3, 0, 0),
  [636] = {.entry = {.count = 1, .reusable = true}}, SHIFT(182),
  [638] = {.entry = {.count = 1, .reusable = true}}, SHIFT(262),
  [640] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_import_statement, 5, 0, 0),
  [642] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_definition, 6, 0, 0),
  [644] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_definition, 6, 0, 0),
  [646] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata_block, 7, 0, 0),
  [648] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_multiplicity, 1, 0, 0),
  [650] = {.entry = {.count = 1, .reusable = true}}, SHIFT(232),
  [652] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instances_section, 3, 0, 0),
  [654] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hub_definition, 6, 0, 0),
  [656] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instances_section, 6, 0, 0),
  [658] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_definitions_section, 6, 0, 0),
  [660] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hubs_block, 4, 0, 0),
  [662] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_structs_block, 4, 0, 0),
  [664] = {.entry = {.count = 1, .reusable = true}}, SHIFT(192),
  [666] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [668] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_definition, 4, 0, 0),
  [670] = {.entry = {.count = 1, .reusable = true}}, SHIFT(195),
  [672] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [674] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fields_block, 4, 0, 0),
  [676] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_imports_section, 6, 0, 0),
  [678] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_imports_section, 4, 0, 0),
  [680] = {.entry = {.count = 1, .reusable = true}}, SHIFT(263),
  [682] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata_block, 6, 0, 0),
  [684] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instance_block, 8, 0, 1),
  [686] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instances_section, 5, 0, 0),
  [688] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_definitions_section, 5, 0, 0),
  [690] = {.entry = {.count = 1, .reusable = true}}, SHIFT(250),
  [692] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_decorator, 4, 0, 0),
  [694] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_definitions_section, 4, 0, 0),
  [696] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hubs_block, 3, 0, 0),
  [698] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_instances_section, 4, 0, 0),
  [700] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_structs_block, 3, 0, 0),
  [702] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enums_block, 3, 0, 0),
  [704] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_imports_section, 5, 0, 0),
  [706] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fields_block, 3, 0, 0),
  [708] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [710] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [712] = {.entry = {.count = 1, .reusable = true}}, SHIFT(217),
  [714] = {.entry = {.count = 1, .reusable = true}}, SHIFT(215),
  [716] = {.entry = {.count = 1, .reusable = true}}, SHIFT(252),
  [718] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [720] = {.entry = {.count = 1, .reusable = true}}, SHIFT(251),
  [722] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_multiplicity, 3, 0, 0),
  [724] = {.entry = {.count = 1, .reusable = true}}, SHIFT(260),
  [726] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [728] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [730] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [732] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [734] = {.entry = {.count = 1, .reusable = true}}, SHIFT(247),
  [736] = {.entry = {.count = 1, .reusable = true}}, SHIFT(249),
  [738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [740] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [742] = {.entry = {.count = 1, .reusable = true}}, SHIFT(151),
  [744] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [748] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [750] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [752] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [754] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [756] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [758] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [760] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [762] = {.entry = {.count = 1, .reusable = true}}, SHIFT(166),
  [764] = {.entry = {.count = 1, .reusable = true}}, SHIFT(143),
  [766] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [768] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [770] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [772] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_role_direction, 1, 0, 0),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_hubgs(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
