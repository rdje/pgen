# JSON Schema Specifications

This document defines the standardized JSON formats for cross-language communication in the EBNF parser generator ecosystem.

## Raw AST JSON Format

The Raw AST JSON is produced by the Perl EBNF parser (`ebnf_to_json.pl`) and consumed by all language-specific pipeline implementations.

### Schema Definition

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Raw AST JSON Schema",
  "type": "object",
  "required": ["grammar_name", "raw_ast", "metadata"],
  "properties": {
    "grammar_name": {
      "type": "string",
      "description": "Name of the grammar being parsed"
    },
    "raw_ast": {
      "type": "array",
      "description": "Array of rule definitions",
      "items": {
        "type": "array",
        "description": "Single rule definition as array of tokens",
        "items": {
          "type": "array",
          "minItems": 2,
          "maxItems": 2,
          "items": [
            {
              "type": "string",
              "enum": [
                "rule", "rule_reference", "quoted_string", "regex",
                "operator", "group_open", "group_close", "quantifier",
                "semantic_annotation", "return_scalar", "return_array", "return_object",
                "logging_annotation"
              ],
              "description": "Token type"
            },
            {
              "type": "string", 
              "description": "Token value"
            }
          ]
        }
      }
    },
    "metadata": {
      "type": "object",
      "required": ["format"],
      "properties": {
        "source_file": {
          "type": "string",
          "description": "Original EBNF file path"
        },
        "format": {
          "type": "string",
          "const": "raw_ast"
        },
        "generated_at": {
          "type": "string",
          "description": "Generation timestamp"
        },
        "parser": {
          "type": "string",
          "description": "Parser identification"
        },
        "includes": {
          "type": "array",
          "items": {"type": "string"},
          "description": "List of included files"
        },
        "rule_count": {
          "type": "integer",
          "minimum": 0,
          "description": "Total number of rules"
        }
      }
    }
  }
}
```

### Example Raw AST JSON

```json
{
  "grammar_name": "simple_arithmetic",
  "raw_ast": [
    [
      ["rule", "expression"],
      ["rule_reference", "term"],
      ["group_open", "("],
      ["quoted_string", "+"],
      ["rule_reference", "term"],
      ["group_close", ")"],
      ["quantifier", "*"]
    ],
    [
      ["rule", "term"],
      ["regex", "(\\d+)"]
    ]
  ],
  "metadata": {
    "source_file": "arithmetic.ebnf",
    "format": "raw_ast",
    "generated_at": "2025-09-02T01:00:00Z",
    "parser": "ebnf_to_json.pl v2.0",
    "rule_count": 2
  }
}
```

## Transformed AST JSON Format

The Transformed AST JSON is produced by language-specific pipeline implementations and consumed by code/data generators.

### Schema Definition

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#", 
  "title": "Transformed AST JSON Schema",
  "type": "object",
  "required": ["grammar_name", "grammar_tree", "rule_order", "metadata"],
  "properties": {
    "grammar_name": {
      "type": "string",
      "description": "Name of the grammar"
    },
    "grammar_tree": {
      "type": "object",
      "description": "Transformed AST tree with rule definitions",
      "patternProperties": {
        "^[a-zA-Z_][a-zA-Z0-9_]*$": {
          "$ref": "#/definitions/ASTNode"
        }
      }
    },
    "rule_order": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Ordered list of rule names"
    },
    "metadata": {
      "type": "object",
      "required": ["format", "source_format"],
      "properties": {
        "format": {
          "type": "string",
          "const": "transformed_ast"
        },
        "source_format": {
          "type": "string", 
          "const": "raw_ast"
        },
        "transformed_at": {
          "type": "string",
          "description": "Transformation timestamp"
        },
        "transformer": {
          "type": "string",
          "description": "Pipeline implementation identifier"
        },
        "pipeline_stage": {
          "type": "string",
          "const": "transformation"
        },
        "annotations": {
          "type": "object",
          "description": "Preserved annotations from raw AST",
          "properties": {
            "semantic_annotations": {"type": "object"},
            "logging_annotations": {"type": "object"},
            "return_annotations": {"type": "object"}
          }
        }
      }
    }
  },
  "definitions": {
    "ASTNode": {
      "type": "object",
      "required": ["type"],
      "properties": {
        "type": {
          "type": "string",
          "enum": ["atom", "sequence", "or", "quantified"]
        }
      },
      "allOf": [
        {
          "if": {"properties": {"type": {"const": "atom"}}},
          "then": {
            "required": ["value"],
            "properties": {
              "value": {
                "oneOf": [
                  {
                    "type": "array",
                    "minItems": 2,
                    "maxItems": 2,
                    "description": "Token format [type, value]"
                  },
                  {"$ref": "#/definitions/ASTNode"}
                ]
              }
            }
          }
        },
        {
          "if": {"properties": {"type": {"const": "sequence"}}},
          "then": {
            "required": ["elements"],
            "properties": {
              "elements": {
                "type": "array",
                "items": {
                  "oneOf": [
                    {
                      "type": "array",
                      "minItems": 2,
                      "maxItems": 2,
                      "description": "Token format [type, value]"
                    },
                    {"$ref": "#/definitions/ASTNode"}
                  ]
                }
              }
            }
          }
        },
        {
          "if": {"properties": {"type": {"const": "or"}}},
          "then": {
            "required": ["alternatives"],
            "properties": {
              "alternatives": {
                "type": "array",
                "items": {"$ref": "#/definitions/ASTNode"}
              }
            }
          }
        },
        {
          "if": {"properties": {"type": {"const": "quantified"}}},
          "then": {
            "required": ["element", "quantifier"],
            "properties": {
              "element": {"$ref": "#/definitions/ASTNode"},
              "quantifier": {
                "type": "string",
                "enum": ["*", "+", "?"]
              }
            }
          }
        }
      ]
    }
  }
}
```

### Example Transformed AST JSON

```json
{
  "grammar_name": "simple_arithmetic",
  "grammar_tree": {
    "expression": {
      "type": "sequence",
      "elements": [
        {
          "type": "atom",
          "value": {
            "type": "atom", 
            "value": ["rule_reference", "term"]
          }
        },
        {
          "type": "quantified",
          "quantifier": "*",
          "element": {
            "type": "atom",
            "value": {
              "type": "sequence",
              "elements": [
                ["quoted_string", "+"],
                ["rule_reference", "term"]
              ]
            }
          }
        }
      ]
    },
    "term": {
      "type": "atom",
      "value": {
        "type": "atom",
        "value": ["regex", "(\\d+)"]
      }
    }
  },
  "rule_order": ["expression", "term"],
  "metadata": {
    "format": "transformed_ast",
    "source_format": "raw_ast", 
    "transformed_at": "2025-09-02T01:05:00Z",
    "transformer": "Python AST Pipeline v1.0",
    "pipeline_stage": "transformation"
  }
}
```

## Token Types Reference

### Raw AST Token Types

| Token Type | Description | Example Value |
|------------|-------------|---------------|
| `rule` | Rule definition | `"expression"` |
| `rule_reference` | Reference to another rule | `"term"` |
| `quoted_string` | String literal | `"+"` |
| `regex` | Regular expression | `"(\\d+)"` |
| `operator` | Grammar operators | `"|"`, `"*"`, `"+"`, `"?"` |
| `group_open` | Opening grouping | `"("` |
| `group_close` | Closing grouping | `")"` |
| `quantifier` | Quantification operators | `"*"`, `"+"`, `"?"` |
| `semantic_annotation` | Semantic metadata | `"@type:integer"` |
| `return_scalar` | Return value annotation | `"return_scalar"` |
| `return_array` | Return array annotation | `"return_array"` |
| `return_object` | Return object annotation | `"return_object"` |
| `logging_annotation` | Debug/logging metadata | `"@log:debug"` |

### Transformed AST Node Types

| Node Type | Description | Required Fields |
|-----------|-------------|-----------------|
| `atom` | Atomic element (terminal or rule reference) | `type`, `value` |
| `sequence` | Ordered sequence of elements | `type`, `elements` |
| `or` | Alternative choices (A \| B \| C) | `type`, `alternatives` |
| `quantified` | Element with quantifier (A*, A+, A?) | `type`, `element`, `quantifier` |

## Validation Tools

Language implementations should validate JSON against these schemas to ensure compatibility. Example validation libraries:

- **Python**: `jsonschema` library
- **Rust**: `serde_json` + `jsonschema` crate
- **Go**: `github.com/xeipuuv/gojsonschema`
- **Julia**: `JSON.jl` + `JSONSchema.jl`
- **JavaScript**: `ajv` library

## Schema Evolution

When extending the schemas:

1. **Backwards Compatible**: Add optional fields only
2. **Version Schema**: Update `$schema` URL with version
3. **Document Changes**: Update this documentation
4. **Test Compatibility**: Ensure existing implementations continue to work

This ensures the multi-language ecosystem remains compatible as features are added.
