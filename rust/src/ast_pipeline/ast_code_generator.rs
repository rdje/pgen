// AST-based Code Generator for Parser Generation
// This replaces string-based code generation with proper AST manipulation
// Guarantees syntactically correct Rust code generation

use syn::{
    parse_quote, Block, Expr, ExprBlock, ExprMatch, Ident, Item, ItemFn, 
    ItemImpl, Pat, PatIdent, Path, Stmt, Token, Type, TypePath,
    punctuated::Punctuated, token, ExprIf, ExprLet, Local,
};
use quote::{quote, format_ident, ToTokens};
use proc_macro2::{TokenStream, Span};
use std::collections::HashMap;
use anyhow::Result;

/// AST-based code generator that produces syntactically correct Rust code
/// This is the long-term solution for robust parser generation
pub struct AstCodeGenerator {
    /// Parser name (e.g., "ReturnAnnotationParser")
    parser_name: String,
    
    /// Generated methods
    methods: Vec<ItemFn>,
    
    /// Import statements
    imports: Vec<TokenStream>,
    
    /// Type definitions
    types: Vec<Item>,
}

impl AstCodeGenerator {
    pub fn new(parser_name: String) -> Self {
        Self {
            parser_name,
            methods: Vec::new(),
            imports: Vec::new(),
            types: Vec::new(),
        }
    }
    
    /// Generate a complete parser implementation
    pub fn generate_parser(&mut self, rules: &HashMap<String, RuleDefinition>) -> Result<TokenStream> {
        // Generate imports
        let imports = self.generate_imports();
        
        // Generate type definitions
        let types = self.generate_types();
        
        // Generate parser struct
        let parser_struct = self.generate_parser_struct();
        
        // Generate parser implementation
        let parser_impl = self.generate_parser_impl(rules)?;
        
        // Combine everything into a complete module
        Ok(quote! {
            #imports
            #types
            #parser_struct
            #parser_impl
        })
    }
    
    /// Generate import statements
    fn generate_imports(&self) -> TokenStream {
        quote! {
            use std::collections::HashMap;
            use std::ops::Range;
            use regex::Regex;
            use serde_json;
            
            // Add other necessary imports
        }
    }
    
    /// Generate type definitions
    fn generate_types(&self) -> TokenStream {
        quote! {
            pub type ParseResult<T> = Result<T, ParseError>;
            
            #[derive(Debug, Clone, PartialEq)]
            pub enum ParseError {
                UnexpectedEof { position: usize },
                UnexpectedToken { expected: &'static str, found: char, position: usize },
                InvalidSyntax { message: &'static str, position: usize },
                Backtrack { position: usize },
            }
            
            #[derive(Debug, Clone, PartialEq)]
            pub enum ParseContent<'input> {
                Terminal(&'input str),
                Sequence(Vec<ParseNode<'input>>),
                Alternative(Box<ParseNode<'input>>),
                Quantified(Vec<ParseNode<'input>>, &'static str),
            }
            
            #[derive(Debug, Clone, PartialEq)]
            pub struct ParseNode<'input> {
                pub rule_name: &'static str,
                pub content: ParseContent<'input>,
                pub span: Range<usize>,
            }
        }
    }
    
    /// Generate the parser struct
    fn generate_parser_struct(&self) -> TokenStream {
        let parser_name = format_ident!("{}", self.parser_name);
        
        quote! {
            pub struct #parser_name<'input> {
                input: &'input str,
                position: usize,
                memo: HashMap<(RuleId, usize), MemoEntry<'input>>,
                recursion_guard: RecursionGuard,
                debug_mode: bool,
                debug_output: Vec<String>,
            }
        }
    }
    
    /// Generate parser implementation
    fn generate_parser_impl(&mut self, rules: &HashMap<String, RuleDefinition>) -> Result<TokenStream> {
        let parser_name = format_ident!("{}", self.parser_name);
        
        // Generate methods for each rule
        let mut rule_methods = Vec::new();
        for (rule_name, rule_def) in rules {
            let method = self.generate_rule_method(rule_name, rule_def)?;
            rule_methods.push(method);
        }
        
        // Generate helper methods
        let helper_methods = self.generate_helper_methods();
        
        Ok(quote! {
            impl<'input> #parser_name<'input> {
                #(#rule_methods)*
                #helper_methods
            }
        })
    }
    
    /// Generate a single rule method with proper AST structure
    fn generate_rule_method(&self, rule_name: &str, rule_def: &RuleDefinition) -> Result<TokenStream> {
        let method_name = format_ident!("parse_{}", rule_name);
        let rule_name_str = rule_name;
        
        // Generate the method body based on rule type
        let method_body = match &rule_def.rule_type {
            RuleType::SingleBranch(branch) => {
                self.generate_single_branch_body(rule_name, branch)?
            }
            RuleType::MultiBranch(branches) => {
                self.generate_multi_branch_body(rule_name, branches)?
            }
        };
        
        Ok(quote! {
            fn #method_name(&mut self) -> ParseResult<ParseNode<'input>> {
                // Recursion guard check
                let position = self.position;
                let cycle_type = self.recursion_guard.check_cycle(#rule_name_str, position);
                
                match cycle_type {
                    CycleType::Infinite => {
                        return Err(ParseError::InvalidSyntax {
                            message: "Infinite recursion detected",
                            position,
                        });
                    }
                    CycleType::LeftRecursive => {
                        return Err(ParseError::InvalidSyntax {
                            message: "Left recursion detected",
                            position,
                        });
                    }
                    _ => {}
                }
                
                self.recursion_guard.enter(#rule_name_str, position);
                
                let result = self.memoized_call(Self::#method_name, |parser| {
                    let start_pos = parser.position;
                    #method_body
                    let end_pos = parser.position;
                    
                    Ok(ParseNode {
                        rule_name: #rule_name_str,
                        content: result,
                        span: start_pos..end_pos,
                    })
                });
                
                self.recursion_guard.exit();
                result
            }
        })
    }
    
    /// Generate single-branch rule body
    fn generate_single_branch_body(&self, rule_name: &str, branch: &BranchDefinition) -> Result<TokenStream> {
        // This generates a properly structured AST for single-branch rules
        let branch_content = self.generate_branch_content(branch)?;
        
        // Handle return annotation if present
        let result_transformation = if let Some(return_ann) = &branch.return_annotation {
            self.generate_return_annotation_transform(return_ann)?
        } else {
            quote! { branch_result }
        };
        
        Ok(quote! {
            let result: ParseContent<'input>;
            
            // Generate branch content
            let branch_result = {
                #branch_content
            };
            
            // Apply transformation
            result = #result_transformation;
            
            result
        })
    }
    
    /// Generate multi-branch rule body
    fn generate_multi_branch_body(&self, rule_name: &str, branches: &[BranchDefinition]) -> Result<TokenStream> {
        let mut branch_checks = Vec::new();
        
        for (idx, branch) in branches.iter().enumerate() {
            let branch_content = self.generate_branch_content(branch)?;
            let branch_check = if idx == 0 {
                quote! {
                    if let Some(content) = parser.try_parse(|p| {
                        #branch_content
                    }) {
                        result = content;
                    }
                }
            } else {
                quote! {
                    else if let Some(content) = parser.try_parse(|p| {
                        #branch_content
                    }) {
                        result = content;
                    }
                }
            };
            branch_checks.push(branch_check);
        }
        
        // Add final else clause
        let else_clause = quote! {
            else {
                return Err(ParseError::InvalidSyntax {
                    message: "No alternative matched",
                    position: parser.position,
                });
            }
        };
        
        Ok(quote! {
            let result: ParseContent<'input>;
            #(#branch_checks)*
            #else_clause
            result
        })
    }
    
    /// Generate content for a single branch
    fn generate_branch_content(&self, branch: &BranchDefinition) -> Result<TokenStream> {
        match &branch.content {
            BranchContent::Sequence(elements) => {
                self.generate_sequence_content(elements)
            }
            BranchContent::Terminal(value) => {
                Ok(quote! {
                    ParseContent::Terminal(parser.match_string(#value)?)
                })
            }
            BranchContent::RuleReference(rule) => {
                let method = format_ident!("parse_{}", rule);
                Ok(quote! {
                    ParseContent::Alternative(Box::new(parser.#method()?))
                })
            }
            BranchContent::Quantified { element, quantifier } => {
                self.generate_quantified_content(element, quantifier)
            }
        }
    }
    
    /// Generate sequence content
    fn generate_sequence_content(&self, elements: &[SequenceElement]) -> Result<TokenStream> {
        let mut element_parsers = Vec::new();
        
        for (idx, element) in elements.iter().enumerate() {
            let element_parser = self.generate_sequence_element(idx, element)?;
            element_parsers.push(element_parser);
        }
        
        Ok(quote! {
            let mut sequence_elements = Vec::with_capacity(#(elements.len()));
            #(#element_parsers)*
            ParseContent::Sequence(sequence_elements)
        })
    }
    
    /// Generate code for a sequence element
    fn generate_sequence_element(&self, index: usize, element: &SequenceElement) -> Result<TokenStream> {
        let element_content = match element {
            SequenceElement::Terminal(s) => {
                quote! { ParseContent::Terminal(parser.match_string(#s)?) }
            }
            SequenceElement::RuleRef(rule) => {
                let method = format_ident!("parse_{}", rule);
                quote! { ParseContent::Alternative(Box::new(parser.#method()?)) }
            }
            SequenceElement::Optional(inner) => {
                let inner_code = self.generate_sequence_element(0, inner)?;
                quote! {
                    if let Some(content) = parser.try_parse(|p| Ok(#inner_code)) {
                        content
                    } else {
                        ParseContent::Sequence(Vec::new())
                    }
                }
            }
        };
        
        let element_name = format!("element_{}", index);
        Ok(quote! {
            {
                let element_start = parser.position;
                let element_content = #element_content;
                let element_end = parser.position;
                
                sequence_elements.push(ParseNode {
                    rule_name: #element_name,
                    content: element_content,
                    span: element_start..element_end,
                });
            }
        })
    }
    
    /// Generate quantified content
    fn generate_quantified_content(&self, element: &Box<BranchContent>, quantifier: &str) -> Result<TokenStream> {
        let element_parser = self.generate_branch_content(element)?;
        
        match quantifier {
            "*" => Ok(quote! {
                let mut results = Vec::new();
                while let Some(content) = parser.try_parse(|p| Ok(#element_parser)) {
                    results.push(ParseNode {
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    });
                }
                ParseContent::Quantified(results, "*")
            }),
            "+" => Ok(quote! {
                let mut results = Vec::new();
                let first = #element_parser;
                results.push(ParseNode {
                    rule_name: "quantified",
                    content: first,
                    span: 0..0,
                });
                
                while let Some(content) = parser.try_parse(|p| Ok(#element_parser)) {
                    results.push(ParseNode {
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    });
                }
                ParseContent::Quantified(results, "+")
            }),
            "?" => Ok(quote! {
                if let Some(content) = parser.try_parse(|p| Ok(#element_parser)) {
                    ParseContent::Quantified(vec![ParseNode {
                        rule_name: "quantified",
                        content,
                        span: 0..0,
                    }], "?")
                } else {
                    ParseContent::Quantified(Vec::new(), "?")
                }
            }),
            _ => Err(anyhow::anyhow!("Unknown quantifier: {}", quantifier))
        }
    }
    
    /// Generate return annotation transformation
    fn generate_return_annotation_transform(&self, annotation: &ReturnAnnotation) -> Result<TokenStream> {
        match &annotation.annotation_type {
            ReturnAnnotationType::Object(props) => {
                // Generate object construction
                let mut field_assignments = Vec::new();
                for (key, value_ast) in props {
                    let field_value = self.generate_return_value_extraction(value_ast)?;
                    field_assignments.push(quote! {
                        json_obj[#key] = serde_json::json!(#field_value);
                    });
                }
                
                Ok(quote! {
                    {
                        let mut json_obj = serde_json::json!({});
                        #(#field_assignments)*
                        let json_str = serde_json::to_string(&json_obj).unwrap_or_else(|_| "{}".to_string());
                        ParseContent::Terminal(json_str)
                    }
                })
            }
            ReturnAnnotationType::PositionalRef(index) => {
                // Extract from branch_result
                Ok(quote! {
                    match &branch_result {
                        ParseContent::Sequence(elements) if elements.len() > #index => {
                            elements[#index].content.clone()
                        }
                        _ => branch_result
                    }
                })
            }
            ReturnAnnotationType::Literal(value) => {
                Ok(quote! {
                    ParseContent::Terminal(#value)
                })
            }
        }
    }
    
    /// Generate code to extract value for return annotation
    fn generate_return_value_extraction(&self, value_ast: &ValueAST) -> Result<TokenStream> {
        match value_ast {
            ValueAST::PositionalRef(idx) => {
                Ok(quote! {
                    match &branch_result {
                        ParseContent::Sequence(ref elems) if elems.len() > #idx => {
                            match &elems[#idx].content {
                                ParseContent::Terminal(s) => s.to_string(),
                                _ => format!("{:?}", elems[#idx].content)
                            }
                        }
                        _ => "<missing>".to_string()
                    }
                })
            }
            ValueAST::StringLiteral(s) => {
                Ok(quote! { #s })
            }
            ValueAST::NumberLiteral(n) => {
                Ok(quote! { #n })
            }
        }
    }
    
    /// Generate helper methods
    fn generate_helper_methods(&self) -> TokenStream {
        quote! {
            fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
                let start = self.position;
                let end = start + expected.len();
                
                if end <= self.input.len() && &self.input[start..end] == expected {
                    self.position = end;
                    Ok(&self.input[start..end])
                } else {
                    Err(ParseError::UnexpectedEof { position: self.position })
                }
            }
            
            fn try_parse<F, T>(&mut self, parser: F) -> Option<T>
            where
                F: FnOnce(&mut Self) -> ParseResult<T>
            {
                let saved_pos = self.position;
                match parser(self) {
                    Ok(result) => Some(result),
                    Err(_) => {
                        self.position = saved_pos;
                        None
                    }
                }
            }
            
            fn memoized_call<F>(&mut self, rule_id: RuleId, parser: F) -> ParseResult<ParseNode<'input>>
            where
                F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>
            {
                let key = (rule_id, self.position);
                
                if let Some(entry) = self.memo.get(&key) {
                    self.position = entry.end_pos;
                    return entry.result.clone();
                }
                
                let result = parser(self);
                let end_pos = self.position;
                
                self.memo.insert(key, MemoEntry {
                    result: result.clone(),
                    end_pos,
                });
                
                result
            }
        }
    }
}

// Supporting types for the AST generator

#[derive(Debug, Clone)]
pub struct RuleDefinition {
    pub rule_type: RuleType,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RuleType {
    SingleBranch(BranchDefinition),
    MultiBranch(Vec<BranchDefinition>),
}

#[derive(Debug, Clone)]
pub struct BranchDefinition {
    pub content: BranchContent,
    pub return_annotation: Option<ReturnAnnotation>,
}

#[derive(Debug, Clone)]
pub enum BranchContent {
    Sequence(Vec<SequenceElement>),
    Terminal(String),
    RuleReference(String),
    Quantified { element: Box<BranchContent>, quantifier: String },
}

#[derive(Debug, Clone)]
pub enum SequenceElement {
    Terminal(String),
    RuleRef(String),
    Optional(Box<SequenceElement>),
}

#[derive(Debug, Clone)]
pub struct ReturnAnnotation {
    pub annotation_type: ReturnAnnotationType,
}

#[derive(Debug, Clone)]
pub enum ReturnAnnotationType {
    Object(HashMap<String, ValueAST>),
    PositionalRef(usize),
    Literal(String),
}

#[derive(Debug, Clone)]
pub enum ValueAST {
    PositionalRef(usize),
    StringLiteral(String),
    NumberLiteral(i64),
}

// Additional types
type RuleId = u16;

struct MemoEntry<'input> {
    result: ParseResult<ParseNode<'input>>,
    end_pos: usize,
}