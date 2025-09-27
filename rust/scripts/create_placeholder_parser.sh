#!/bin/bash
# Script to create placeholder parser files to avoid circular dependencies

if [ $# -ne 2 ]; then
    echo "Usage: $0 <parser_name> <output_file>"
    echo "Example: $0 Semantic_annotationParser ../generated/semantic_annotation_parser.rs"
    exit 1
fi

PARSER_NAME="$1"
OUTPUT_FILE="$2"

cat > "$OUTPUT_FILE" << EOF
// Temporary placeholder for ${PARSER_NAME}
// This will be replaced by the generated parser

use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseNode<'input> {
    pub rule_name: &'static str,
    pub content: ParseContent<'input>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseContent<'input> {
    Terminal(&'input str),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEof { position: usize },
    UnexpectedToken { expected: &'static str, found: char, position: usize },
    InvalidSyntax { message: &'static str, position: usize },
    Backtrack { position: usize },
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct ${PARSER_NAME}<'input> {
    _phantom: std::marker::PhantomData<&'input str>,
}

impl<'input> ${PARSER_NAME}<'input> {
    pub fn new(_input: &'input str) -> Self {
        Self { _phantom: std::marker::PhantomData }
    }
    
    pub fn with_debug(_input: &'input str) -> Self {
        Self::new(_input)
    }
    
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
        // Placeholder - always fails, forces bootstrap mode
        Err(ParseError::UnexpectedEof { position: 0 })
    }
}
EOF

echo "Created placeholder parser: $OUTPUT_FILE"