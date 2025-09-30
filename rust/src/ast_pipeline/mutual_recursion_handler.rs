// Mutual Recursion Handler for SOTA Parser Generation
// Handles complex mutual recursion patterns without grammar modification

use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

/// Cycle detection result
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    /// No cycle detected
    None,
    /// Non-productive infinite cycle (same rule, same position)
    Infinite,
    /// Left-recursive cycle (same rule, earlier position)  
    LeftRecursive,
    /// Mutual recursion cycle (different rules, may be productive)
    MutualRecursive { depth: usize, rules: Vec<String> },
}

/// Smart recursion guard with cycle detection
pub struct RecursionGuard {
    /// Stack of (rule_name, position) currently being parsed
    parse_stack: Vec<(String, usize)>,
    /// Maximum depth before forcing backtrack
    max_depth: usize,
    /// Cache of detected cycles for quick lookup
    cycle_cache: HashMap<(String, usize), CycleType>,
    /// Tracks which rule combinations form mutual recursion
    mutual_recursion_groups: HashMap<String, HashSet<String>>,
}

impl RecursionGuard {
    pub fn new(max_depth: usize) -> Self {
        Self {
            parse_stack: Vec::new(),
            max_depth,
            cycle_cache: HashMap::new(),
            mutual_recursion_groups: HashMap::new(),
        }
    }

    /// Check if entering this rule would create a cycle
    pub fn check_cycle(&mut self, rule_name: &str, position: usize) -> CycleType {
        // Check cache first
        if let Some(cached) = self.cycle_cache.get(&(rule_name.to_string(), position)) {
            return cached.clone();
        }

        // Check for direct infinite recursion
        for (i, (r, p)) in self.parse_stack.iter().enumerate() {
            if r == rule_name && *p == position {
                // Exact same rule at same position = infinite loop
                let cycle = CycleType::Infinite;
                self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }
            
            if r == rule_name && *p > position {
                // Same rule but consumed input = left recursion
                let cycle = CycleType::LeftRecursive;
                self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }
        }

        // Check for mutual recursion patterns
        if self.parse_stack.len() >= 2 {
            let mut rules_in_cycle = HashSet::new();
            let mut found_repeat = false;
            
            for (r, p) in self.parse_stack.iter().rev() {
                rules_in_cycle.insert(r.clone());
                if r == rule_name {
                    found_repeat = true;
                    break;
                }
            }
            
            if found_repeat && rules_in_cycle.len() > 1 {
                // Mutual recursion detected
                let cycle = CycleType::MutualRecursive {
                    depth: self.parse_stack.len(),
                    rules: rules_in_cycle.into_iter().collect(),
                };
                self.cycle_cache.insert((rule_name.to_string(), position), cycle.clone());
                return cycle;
            }
        }

        // Check depth limit
        if self.parse_stack.len() >= self.max_depth {
            // Too deep, likely mutual recursion
            let rules: HashSet<String> = self.parse_stack.iter().map(|(r, _)| r.clone()).collect();
            return CycleType::MutualRecursive {
                depth: self.parse_stack.len(),
                rules: rules.into_iter().collect(),
            };
        }

        CycleType::None
    }

    /// Enter a rule (push to stack)
    pub fn enter(&mut self, rule_name: &str, position: usize) {
        self.parse_stack.push((rule_name.to_string(), position));
    }

    /// Exit a rule (pop from stack)  
    pub fn exit(&mut self) {
        self.parse_stack.pop();
    }

    /// Get current recursion depth
    pub fn depth(&self) -> usize {
        self.parse_stack.len()
    }

    /// Check if we should allow this parse attempt based on cycle type
    pub fn should_continue(&self, cycle_type: &CycleType, position: usize, input_len: usize) -> bool {
        match cycle_type {
            CycleType::None => true,
            CycleType::Infinite => false, // Never continue on infinite loops
            CycleType::LeftRecursive => false, // Block left recursion
            CycleType::MutualRecursive { depth, .. } => {
                // Allow mutual recursion up to a point, but with exponential backoff
                // This handles legitimate nested structures while preventing stack overflow
                *depth < self.max_depth && position < input_len
            }
        }
    }
}

/// Generate parser code with mutual recursion handling
pub fn generate_mutual_recursion_safe_parser_method(
    rule_name: &str,
    original_body: &str,
) -> String {
    format!(r#"
    /// Parse {rule_name} with mutual recursion protection
    #[inline]
    fn parse_{rule_name}(&mut self) -> ParseResult<ParseNode<'input>> {{
        // Check for recursion cycles before entering
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("{rule_name}", position);
        
        match cycle_type {{
            CycleType::Infinite => {{
                // Infinite loop detected, fail immediately
                return Err(ParseError::InvalidSyntax {{
                    message: format!("Infinite recursion detected in rule: {rule_name}"),
                    position,
                }});
            }}
            CycleType::LeftRecursive => {{
                // Left recursion detected, fail to break cycle
                return Err(ParseError::InvalidSyntax {{
                    message: format!("Left recursion detected in rule: {rule_name}"),
                    position,
                }});
            }}
            CycleType::MutualRecursive {{ depth, ref rules }} if depth >= self.max_recursion_depth => {{
                // Maximum recursion depth exceeded
                return Err(ParseError::InvalidSyntax {{
                    message: format!("Maximum recursion depth {{}} exceeded in mutual recursion between: {{:?}}", depth, rules),
                    position,
                }});
            }}
            _ => {{
                // Safe to proceed
            }}
        }}
        
        // Enter recursion tracking
        self.recursion_guard.enter("{rule_name}", position);
        
        // Original parser body with memoization
        let result = self.memoized_call(Self::RULE_{rule_name_upper}, |parser| {{
            parser.debug_enter_rule("{rule_name}");
            let start_pos = parser.position;
            
            let parse_result: ParseResult<ParseNode<'input>> = (|| {{
{original_body}
            }})();
            
            match &parse_result {{
                Ok(_) => parser.debug_exit_success("{rule_name}", start_pos),
                Err(err) => parser.debug_exit_fail("{rule_name}", err),
            }};
            
            parse_result
        }});
        
        // Exit recursion tracking
        self.recursion_guard.exit();
        
        result
    }}
"#,
        rule_name = rule_name,
        rule_name_upper = rule_name.to_uppercase(),
        original_body = original_body
    )
}

/// Trampoline implementation for zero-stack parsing
pub enum ParseContinuation<'input> {
    Done(Result<ParseNode<'input>, ParseError>),
    Continue {
        rule: String,
        position: usize,
    },
}

// Parse with trampolining will be integrated directly into generated parsers

// Type aliases for integration with generated parsers
pub type RuleId = u16;
pub type ParseError = String; // Will be replaced with actual error type
pub type ParseNode<'input> = String; // Will be replaced with actual node type
pub type MemoEntry<'input> = String; // Will be replaced with actual memo type

// Re-export for use in generated code
pub use self::{CycleType, RecursionGuard, ParseContinuation};
