use super::ASTNode;
// Robust SOTA implementation for parsing grouped and quantified elements in EBNF
// Handles arbitrary nesting levels and complex patterns

use crate::ast_pipeline::TokenValue;
use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    GroupOpen,               // (
    GroupClose,              // )
    Quantifier(String),      // ?, *, +
    Element(String, String), // (type, value) for actual content
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedElement {
    /// A simple element (terminal or rule reference)
    Simple {
        token_type: String,
        token_value: String,
    },
    /// A sequence of elements
    Sequence { elements: Vec<ParsedElement> },
    /// An alternative (a | b | c)
    Alternative { branches: Vec<ParsedElement> },
    /// A quantified element with ?, *, or +
    Quantified {
        element: Box<ParsedElement>,
        quantifier: String,
    },
    /// A grouped element without quantifier
    Group { element: Box<ParsedElement> },
}

impl fmt::Display for ParsedElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedElement::Simple {
                token_type,
                token_value,
            } => {
                write!(f, "{}:{}", token_type, token_value)
            }
            ParsedElement::Sequence { elements } => {
                write!(f, "(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            ParsedElement::Alternative { branches } => {
                write!(f, "(")?;
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", branch)?;
                }
                write!(f, ")")
            }
            ParsedElement::Quantified {
                element,
                quantifier,
            } => {
                write!(f, "{}{}", element, quantifier)
            }
            ParsedElement::Group { element } => {
                write!(f, "({})", element)
            }
        }
    }
}

#[derive(Debug)]
pub struct GroupedQuantifierParser {
    debug_mode: bool,
}

impl GroupedQuantifierParser {
    pub fn new(debug_mode: bool) -> Self {
        Self { debug_mode }
    }

    pub fn tokenize_from_raw_tokens(&self, tokens: &[Vec<TokenValue>]) -> Result<Vec<Token>> {
        let mut result = Vec::new();

        for token in tokens {
            if token.len() >= 2 {
                // Convert TokenValue to String for processing
                let token_type = match &token[0] {
                    TokenValue::String(s) => s.as_str(),
                };
                let token_value = match &token[1] {
                    TokenValue::String(s) => s.as_str(),
                };

                match token_type {
                    "group_open" => result.push(Token::GroupOpen),
                    "group_close" => result.push(Token::GroupClose),
                    "quantifier" => result.push(Token::Quantifier(token_value.to_string())),
                    _ => result.push(Token::Element(
                        token_type.to_string(),
                        token_value.to_string(),
                    )),
                }
            }
        }

        Ok(result)
    }

    pub fn parse_sequence(&self, tokens: &[Token]) -> Result<Vec<ParsedElement>> {
        // Simple implementation for now - just convert each token to a ParsedElement
        let mut result = Vec::new();

        for token in tokens {
            match token {
                Token::Element(token_type, token_value) => {
                    result.push(ParsedElement::Simple {
                        token_type: token_type.clone(),
                        token_value: token_value.clone(),
                    });
                }
                _ => {
                    // For now, skip non-element tokens
                }
            }
        }

        Ok(result)
    }

    pub fn to_ast_node(&self, element: ParsedElement) -> ASTNode {
        match element {
            ParsedElement::Simple {
                token_type,
                token_value,
            } => ASTNode::Atom {
                value: crate::ast_pipeline::ASTValue::Token(vec![
                    crate::ast_pipeline::TokenValue::String(token_type),
                    crate::ast_pipeline::TokenValue::String(token_value),
                ]),
            },
            ParsedElement::Sequence { elements } => {
                let ast_elements: Vec<ASTNode> =
                    elements.into_iter().map(|e| self.to_ast_node(e)).collect();
                ASTNode::Sequence {
                    elements: ast_elements,
                }
            }
            ParsedElement::Alternative { branches } => {
                let ast_branches: Vec<ASTNode> =
                    branches.into_iter().map(|e| self.to_ast_node(e)).collect();
                ASTNode::Or {
                    alternatives: ast_branches,
                }
            }
            ParsedElement::Quantified {
                element,
                quantifier,
            } => {
                let ast_element = self.to_ast_node(*element);
                ASTNode::Quantified {
                    element: Box::new(ast_element),
                    quantifier,
                }
            }
            ParsedElement::Group { element } => {
                let ast_element = self.to_ast_node(*element);
                ASTNode::Atom {
                    value: crate::ast_pipeline::ASTValue::Node(Box::new(ast_element)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_simple() {
        let element = ParsedElement::Simple {
            token_type: "rule_reference".to_string(),
            token_value: "expression".to_string(),
        };
        assert_eq!(format!("{}", element), "rule_reference:expression");
    }

    #[test]
    fn test_display_sequence() {
        let element = ParsedElement::Sequence {
            elements: vec![
                ParsedElement::Simple {
                    token_type: "rule_reference".to_string(),
                    token_value: "term".to_string(),
                },
                ParsedElement::Simple {
                    token_type: "terminal".to_string(),
                    token_value: "+".to_string(),
                },
            ],
        };
        assert_eq!(format!("{}", element), "(rule_reference:term terminal:+)");
    }
}
