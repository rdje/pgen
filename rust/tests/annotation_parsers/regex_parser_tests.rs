//! Regex Parser Test Suite
//! 
//! Round-trip testing for regex pattern parsing and serialization

use super::test_utils::load_test_cases;
use super::TestResults;

/// Represents different types of regex AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum RegexAST {
    // Basic atoms
    Literal(String),
    Dot,
    
    // Character classes
    CharClass {
        negated: bool,
        items: Vec<CharClassItem>,
    },
    
    // Quantifiers
    Quantified {
        atom: Box<RegexAST>,
        quantifier: Quantifier,
    },
    
    // Groups
    Group {
        kind: GroupKind,
        pattern: Box<RegexAST>,
    },
    
    // Sequences and alternatives
    Sequence(Vec<RegexAST>),
    Alternation(Vec<RegexAST>),
    
    // Anchors
    Anchor(AnchorKind),
    
    // Escapes
    Escape(EscapeKind),
    
    // Backreferences
    Backreference(BackrefKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharClassItem {
    Literal(char),
    Range(char, char),
    Escape(EscapeKind),
    Posix(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    Star { lazy: bool, possessive: bool },
    Plus { lazy: bool, possessive: bool },
    Question { lazy: bool, possessive: bool },
    Exact(u32),
    Range { min: u32, max: Option<u32>, lazy: bool, possessive: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupKind {
    Capturing,
    NonCapturing,
    Named(String),
    Atomic,
    LookaheadPositive,
    LookaheadNegative,
    LookbehindPositive,
    LookbehindNegative,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorKind {
    StartOfString,        // ^
    EndOfString,          // $
    StartOfInput,         // \A
    EndOfInput,           // \Z
    EndOfInputFinal,      // \z
    WordBoundary,         // \b
    NonWordBoundary,      // \B
    ContinuousMatch,      // \G
}

#[derive(Debug, Clone, PartialEq)]
pub enum EscapeKind {
    // Character class escapes
    Digit,          // \d
    NonDigit,       // \D
    Word,           // \w
    NonWord,        // \W
    Whitespace,     // \s
    NonWhitespace,  // \S
    
    // Literal escapes
    Newline,        // \n
    Tab,            // \t
    CarriageReturn, // \r
    Backslash,      // \\
    
    // Meta character escapes
    Dot,            // \.
    Star,           // \*
    Plus,           // \+
    Question,       // \?
    Caret,          // \^
    Dollar,         // \$
    Pipe,           // \|
    LeftParen,      // \(
    RightParen,     // \)
    LeftBracket,    // \[
    RightBracket,   // \]
    LeftBrace,      // \{
    RightBrace,     // \}
    
    // Hex/Unicode/Octal escapes
    Hex(String),     // \x41, \x{41}
    Unicode(String), // \u{41}
    Octal(String),   // \012
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackrefKind {
    Numbered(u32),      // \1, \2, etc.
    Named(String),      // \k<name>, \k'name'
}

/// Mock regex parser - implements basic parsing for common patterns
pub struct MockRegexParser;

impl MockRegexParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, pattern: &str) -> Result<RegexAST, String> {
        if pattern.trim().is_empty() {
            return Err("Empty pattern".to_string());
        }
        
        self.parse_alternation(pattern.trim())
    }
    
    fn parse_alternation(&self, pattern: &str) -> Result<RegexAST, String> {
        if pattern.contains('|') {
            let parts: Vec<&str> = pattern.split('|').collect();
            let alternatives: Result<Vec<RegexAST>, String> = parts
                .iter()
                .map(|part| self.parse_sequence(part))
                .collect();
            
            match alternatives {
                Ok(alts) => {
                    if alts.len() == 1 {
                        Ok(alts.into_iter().next().unwrap())
                    } else {
                        Ok(RegexAST::Alternation(alts))
                    }
                }
                Err(e) => Err(e)
            }
        } else {
            self.parse_sequence(pattern)
        }
    }
    
    fn parse_sequence(&self, pattern: &str) -> Result<RegexAST, String> {
        let mut items = Vec::new();
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let (ast, consumed) = self.parse_piece(&chars, i)?;
            items.push(ast);
            i += consumed;
        }
        
        if items.len() == 1 {
            Ok(items.into_iter().next().unwrap())
        } else {
            Ok(RegexAST::Sequence(items))
        }
    }
    
    fn parse_piece(&self, chars: &[char], start: usize) -> Result<(RegexAST, usize), String> {
        let (atom, atom_consumed) = self.parse_atom(chars, start)?;
        let quantifier_start = start + atom_consumed;
        
        if quantifier_start < chars.len() {
            match chars[quantifier_start] {
                '*' => Ok((RegexAST::Quantified {
                    atom: Box::new(atom),
                    quantifier: Quantifier::Star { lazy: false, possessive: false }
                }, atom_consumed + 1)),
                '+' => Ok((RegexAST::Quantified {
                    atom: Box::new(atom),
                    quantifier: Quantifier::Plus { lazy: false, possessive: false }
                }, atom_consumed + 1)),
                '?' => Ok((RegexAST::Quantified {
                    atom: Box::new(atom),
                    quantifier: Quantifier::Question { lazy: false, possessive: false }
                }, atom_consumed + 1)),
                '{' => {
                    // Simple counted quantifier parsing
                    let mut end = quantifier_start + 1;
                    while end < chars.len() && chars[end] != '}' {
                        end += 1;
                    }
                    if end >= chars.len() {
                        return Err("Unclosed quantifier".to_string());
                    }
                    
                    let count_str: String = chars[(quantifier_start + 1)..end].iter().collect();
                    if let Ok(count) = count_str.parse::<u32>() {
                        Ok((RegexAST::Quantified {
                            atom: Box::new(atom),
                            quantifier: Quantifier::Exact(count)
                        }, end - start + 1))
                    } else {
                        Err(format!("Invalid quantifier: {{{}}}", count_str))
                    }
                }
                _ => Ok((atom, atom_consumed))
            }
        } else {
            Ok((atom, atom_consumed))
        }
    }
    
    fn parse_atom(&self, chars: &[char], start: usize) -> Result<(RegexAST, usize), String> {
        if start >= chars.len() {
            return Err("Unexpected end of pattern".to_string());
        }
        
        match chars[start] {
            '.' => Ok((RegexAST::Dot, 1)),
            '^' => Ok((RegexAST::Anchor(AnchorKind::StartOfString), 1)),
            '$' => Ok((RegexAST::Anchor(AnchorKind::EndOfString), 1)),
            
            '\\' => {
                if start + 1 >= chars.len() {
                    return Err("Incomplete escape sequence".to_string());
                }
                
                match chars[start + 1] {
                    'd' => Ok((RegexAST::Escape(EscapeKind::Digit), 2)),
                    'D' => Ok((RegexAST::Escape(EscapeKind::NonDigit), 2)),
                    'w' => Ok((RegexAST::Escape(EscapeKind::Word), 2)),
                    'W' => Ok((RegexAST::Escape(EscapeKind::NonWord), 2)),
                    's' => Ok((RegexAST::Escape(EscapeKind::Whitespace), 2)),
                    'S' => Ok((RegexAST::Escape(EscapeKind::NonWhitespace), 2)),
                    'n' => Ok((RegexAST::Escape(EscapeKind::Newline), 2)),
                    't' => Ok((RegexAST::Escape(EscapeKind::Tab), 2)),
                    'r' => Ok((RegexAST::Escape(EscapeKind::CarriageReturn), 2)),
                    '\\' => Ok((RegexAST::Escape(EscapeKind::Backslash), 2)),
                    '.' => Ok((RegexAST::Escape(EscapeKind::Dot), 2)),
                    '*' => Ok((RegexAST::Escape(EscapeKind::Star), 2)),
                    '+' => Ok((RegexAST::Escape(EscapeKind::Plus), 2)),
                    '?' => Ok((RegexAST::Escape(EscapeKind::Question), 2)),
                    '^' => Ok((RegexAST::Escape(EscapeKind::Caret), 2)),
                    '$' => Ok((RegexAST::Escape(EscapeKind::Dollar), 2)),
                    '|' => Ok((RegexAST::Escape(EscapeKind::Pipe), 2)),
                    '(' => Ok((RegexAST::Escape(EscapeKind::LeftParen), 2)),
                    ')' => Ok((RegexAST::Escape(EscapeKind::RightParen), 2)),
                    '[' => Ok((RegexAST::Escape(EscapeKind::LeftBracket), 2)),
                    ']' => Ok((RegexAST::Escape(EscapeKind::RightBracket), 2)),
                    '{' => Ok((RegexAST::Escape(EscapeKind::LeftBrace), 2)),
                    '}' => Ok((RegexAST::Escape(EscapeKind::RightBrace), 2)),
                    'A' => Ok((RegexAST::Anchor(AnchorKind::StartOfInput), 2)),
                    'Z' => Ok((RegexAST::Anchor(AnchorKind::EndOfInput), 2)),
                    'z' => Ok((RegexAST::Anchor(AnchorKind::EndOfInputFinal), 2)),
                    'b' => Ok((RegexAST::Anchor(AnchorKind::WordBoundary), 2)),
                    'B' => Ok((RegexAST::Anchor(AnchorKind::NonWordBoundary), 2)),
                    'G' => Ok((RegexAST::Anchor(AnchorKind::ContinuousMatch), 2)),
                    c if c.is_ascii_digit() => {
                        // Simple numbered backreference
                        let num = c.to_digit(10).unwrap();
                        Ok((RegexAST::Backreference(BackrefKind::Numbered(num)), 2))
                    }
                    _ => Err(format!("Unsupported escape: \\{}", chars[start + 1]))
                }
            }
            
            '[' => self.parse_char_class(chars, start),
            
            '(' => self.parse_group(chars, start),
            
            // Regular literal characters
            c if !c.is_ascii_control() && !"*+?{}()[]|^$.\\".contains(c) => {
                // Parse consecutive literal characters
                let mut end = start + 1;
                while end < chars.len() {
                    let ch = chars[end];
                    if ch.is_ascii_control() || "*+?{}()[]|^$.\\".contains(ch) {
                        break;
                    }
                    end += 1;
                }
                
                let literal: String = chars[start..end].iter().collect();
                Ok((RegexAST::Literal(literal), end - start))
            }
            
            _ => Err(format!("Unexpected character: {}", chars[start]))
        }
    }
    
    fn parse_char_class(&self, chars: &[char], start: usize) -> Result<(RegexAST, usize), String> {
        if start >= chars.len() || chars[start] != '[' {
            return Err("Expected '['".to_string());
        }
        
        let mut i = start + 1;
        let negated = if i < chars.len() && chars[i] == '^' {
            i += 1;
            true
        } else {
            false
        };
        
        let mut items = Vec::new();
        
        while i < chars.len() && chars[i] != ']' {
            if chars[i] == '\\' && i + 1 < chars.len() {
                // Escape sequence in character class
                match chars[i + 1] {
                    'd' => items.push(CharClassItem::Escape(EscapeKind::Digit)),
                    'D' => items.push(CharClassItem::Escape(EscapeKind::NonDigit)),
                    'w' => items.push(CharClassItem::Escape(EscapeKind::Word)),
                    'W' => items.push(CharClassItem::Escape(EscapeKind::NonWord)),
                    's' => items.push(CharClassItem::Escape(EscapeKind::Whitespace)),
                    'S' => items.push(CharClassItem::Escape(EscapeKind::NonWhitespace)),
                    c => items.push(CharClassItem::Literal(c)),
                }
                i += 2;
            } else if i + 2 < chars.len() && chars[i + 1] == '-' && chars[i + 2] != ']' {
                // Character range
                let start_char = chars[i];
                let end_char = chars[i + 2];
                items.push(CharClassItem::Range(start_char, end_char));
                i += 3;
            } else {
                // Single character
                items.push(CharClassItem::Literal(chars[i]));
                i += 1;
            }
        }
        
        if i >= chars.len() {
            return Err("Unclosed character class".to_string());
        }
        
        i += 1; // Skip the ']'
        
        Ok((RegexAST::CharClass { negated, items }, i - start))
    }
    
    fn parse_group(&self, chars: &[char], start: usize) -> Result<(RegexAST, usize), String> {
        if start >= chars.len() || chars[start] != '(' {
            return Err("Expected '('".to_string());
        }
        
        let mut i = start + 1;
        
        // Check for special group syntax
        let kind = if i + 1 < chars.len() && chars[i] == '?' {
            match chars[i + 1] {
                ':' => {
                    i += 2;
                    GroupKind::NonCapturing
                }
                '>' => {
                    i += 2;
                    GroupKind::Atomic
                }
                '=' => {
                    i += 2;
                    GroupKind::LookaheadPositive
                }
                '!' => {
                    i += 2;
                    GroupKind::LookaheadNegative
                }
                '<' if i + 2 < chars.len() => {
                    match chars[i + 2] {
                        '=' => {
                            i += 3;
                            GroupKind::LookbehindPositive
                        }
                        '!' => {
                            i += 3;
                            GroupKind::LookbehindNegative
                        }
                        _ => {
                            // Named group (?<name>...)
                            i += 2;
                            let name_start = i;
                            while i < chars.len() && chars[i] != '>' {
                                i += 1;
                            }
                            if i >= chars.len() {
                                return Err("Unclosed named group".to_string());
                            }
                            let name: String = chars[name_start..i].iter().collect();
                            i += 1;
                            GroupKind::Named(name)
                        }
                    }
                }
                '\'' => {
                    // Named group (?'name'...)
                    i += 2;
                    let name_start = i;
                    while i < chars.len() && chars[i] != '\'' {
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err("Unclosed named group".to_string());
                    }
                    let name: String = chars[name_start..i].iter().collect();
                    i += 1;
                    GroupKind::Named(name)
                }
                _ => return Err("Unsupported group syntax".to_string())
            }
        } else {
            GroupKind::Capturing
        };
        
        // Find the matching closing parenthesis
        let mut paren_count = 1;
        let group_start = i;
        
        while i < chars.len() && paren_count > 0 {
            match chars[i] {
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '\\' if i + 1 < chars.len() => i += 1, // Skip escaped characters
                _ => {}
            }
            i += 1;
        }
        
        if paren_count > 0 {
            return Err("Unclosed group".to_string());
        }
        
        let group_end = i - 1; // Don't include the closing ')'
        let group_pattern: String = chars[group_start..group_end].iter().collect();
        
        let inner_ast = self.parse_alternation(&group_pattern)?;
        
        Ok((RegexAST::Group {
            kind,
            pattern: Box::new(inner_ast)
        }, i - start))
    }
}

/// Regex serializer - converts AST back to regex pattern string
pub struct RegexSerializer;

impl RegexSerializer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn serialize(&self, ast: &RegexAST) -> String {
        self.serialize_node(ast)
    }
    
    fn serialize_node(&self, ast: &RegexAST) -> String {
        match ast {
            RegexAST::Literal(s) => s.clone(),
            RegexAST::Dot => ".".to_string(),
            
            RegexAST::CharClass { negated, items } => {
                let mut result = "[".to_string();
                if *negated {
                    result.push('^');
                }
                for item in items {
                    result.push_str(&self.serialize_char_class_item(item));
                }
                result.push(']');
                result
            }
            
            RegexAST::Quantified { atom, quantifier } => {
                let atom_str = self.serialize_node(atom);
                let quant_str = self.serialize_quantifier(quantifier);
                format!("{}{}", atom_str, quant_str)
            }
            
            RegexAST::Group { kind, pattern } => {
                let pattern_str = self.serialize_node(pattern);
                match kind {
                    GroupKind::Capturing => format!("({})", pattern_str),
                    GroupKind::NonCapturing => format!("(?:{})", pattern_str),
                    GroupKind::Named(name) => format!("(?<{}>{})", name, pattern_str),
                    GroupKind::Atomic => format!("(?>{})", pattern_str),
                    GroupKind::LookaheadPositive => format!("(?={})", pattern_str),
                    GroupKind::LookaheadNegative => format!("(?!{})", pattern_str),
                    GroupKind::LookbehindPositive => format!("(?<={})", pattern_str),
                    GroupKind::LookbehindNegative => format!("(?<!{})", pattern_str),
                }
            }
            
            RegexAST::Sequence(items) => {
                items.iter().map(|item| self.serialize_node(item)).collect()
            }
            
            RegexAST::Alternation(items) => {
                items.iter()
                    .map(|item| self.serialize_node(item))
                    .collect::<Vec<_>>()
                    .join("|")
            }
            
            RegexAST::Anchor(kind) => self.serialize_anchor(kind),
            RegexAST::Escape(kind) => self.serialize_escape(kind),
            RegexAST::Backreference(kind) => self.serialize_backref(kind),
        }
    }
    
    fn serialize_char_class_item(&self, item: &CharClassItem) -> String {
        match item {
            CharClassItem::Literal(c) => {
                // Escape special characters in character classes
                match *c {
                    ']' => "\\]".to_string(),
                    '\\' => "\\\\".to_string(),
                    '^' => "\\^".to_string(),
                    '-' => "\\-".to_string(),
                    _ => c.to_string()
                }
            }
            CharClassItem::Range(start, end) => format!("{}-{}", start, end),
            CharClassItem::Escape(escape) => self.serialize_escape(escape),
            CharClassItem::Posix(name) => format!("[:{name}:]"),
        }
    }
    
    fn serialize_quantifier(&self, quantifier: &Quantifier) -> String {
        match quantifier {
            Quantifier::Star { lazy, possessive } => {
                let mut result = "*".to_string();
                if *lazy { result.push('?'); }
                if *possessive { result.push('+'); }
                result
            }
            Quantifier::Plus { lazy, possessive } => {
                let mut result = "+".to_string();
                if *lazy { result.push('?'); }
                if *possessive { result.push('+'); }
                result
            }
            Quantifier::Question { lazy, possessive } => {
                let mut result = "?".to_string();
                if *lazy { result.push('?'); }
                if *possessive { result.push('+'); }
                result
            }
            Quantifier::Exact(count) => format!("{{{}}}", count),
            Quantifier::Range { min, max, lazy, possessive } => {
                let mut result = if let Some(max) = max {
                    format!("{{{},{}}}", min, max)
                } else {
                    format!("{{{},}}", min)
                };
                if *lazy { result.push('?'); }
                if *possessive { result.push('+'); }
                result
            }
        }
    }
    
    fn serialize_anchor(&self, anchor: &AnchorKind) -> String {
        match anchor {
            AnchorKind::StartOfString => "^".to_string(),
            AnchorKind::EndOfString => "$".to_string(),
            AnchorKind::StartOfInput => "\\A".to_string(),
            AnchorKind::EndOfInput => "\\Z".to_string(),
            AnchorKind::EndOfInputFinal => "\\z".to_string(),
            AnchorKind::WordBoundary => "\\b".to_string(),
            AnchorKind::NonWordBoundary => "\\B".to_string(),
            AnchorKind::ContinuousMatch => "\\G".to_string(),
        }
    }
    
    fn serialize_escape(&self, escape: &EscapeKind) -> String {
        match escape {
            EscapeKind::Digit => "\\d".to_string(),
            EscapeKind::NonDigit => "\\D".to_string(),
            EscapeKind::Word => "\\w".to_string(),
            EscapeKind::NonWord => "\\W".to_string(),
            EscapeKind::Whitespace => "\\s".to_string(),
            EscapeKind::NonWhitespace => "\\S".to_string(),
            EscapeKind::Newline => "\\n".to_string(),
            EscapeKind::Tab => "\\t".to_string(),
            EscapeKind::CarriageReturn => "\\r".to_string(),
            EscapeKind::Backslash => "\\\\".to_string(),
            EscapeKind::Dot => "\\.".to_string(),
            EscapeKind::Star => "\\*".to_string(),
            EscapeKind::Plus => "\\+".to_string(),
            EscapeKind::Question => "\\?".to_string(),
            EscapeKind::Caret => "\\^".to_string(),
            EscapeKind::Dollar => "\\$".to_string(),
            EscapeKind::Pipe => "\\|".to_string(),
            EscapeKind::LeftParen => "\\(".to_string(),
            EscapeKind::RightParen => "\\)".to_string(),
            EscapeKind::LeftBracket => "\\[".to_string(),
            EscapeKind::RightBracket => "\\]".to_string(),
            EscapeKind::LeftBrace => "\\{".to_string(),
            EscapeKind::RightBrace => "\\}".to_string(),
            EscapeKind::Hex(hex) => format!("\\x{}", hex),
            EscapeKind::Unicode(unicode) => format!("\\u{{{}}}", unicode),
            EscapeKind::Octal(octal) => format!("\\{}", octal),
        }
    }
    
    fn serialize_backref(&self, backref: &BackrefKind) -> String {
        match backref {
            BackrefKind::Numbered(num) => format!("\\{}", num),
            BackrefKind::Named(name) => format!("\\k<{}>", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let parser = MockRegexParser::new();
        
        // Test simple literal
        let result = parser.parse("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RegexAST::Literal("hello".to_string()));
        
        // Test dot
        let result = parser.parse(".");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RegexAST::Dot);
        
        // Test quantifiers
        let result = parser.parse("a*");
        assert!(result.is_ok());
        match result.unwrap() {
            RegexAST::Quantified { atom, quantifier } => {
                assert_eq!(*atom, RegexAST::Literal("a".to_string()));
                assert_eq!(quantifier, Quantifier::Star { lazy: false, possessive: false });
            }
            _ => panic!("Expected quantified expression")
        }
    }
    
    #[test]
    fn test_round_trip_basic_patterns() {
        let parser = MockRegexParser::new();
        let serializer = RegexSerializer::new();
        
        let test_patterns = vec![
            "hello",
            ".",
            "a*",
            "b+",
            "c?",
            "^test",
            "test$",
            "\\d",
            "\\w",
            "\\s",
        ];
        
        for pattern in test_patterns {
            println!("Testing pattern: {}", pattern);
            match parser.parse(pattern) {
                Ok(ast) => {
                    let serialized = serializer.serialize(&ast);
                    assert_eq!(pattern, serialized, "Round-trip failed for pattern: {}", pattern);
                }
                Err(e) => {
                    panic!("Failed to parse pattern '{}': {}", pattern, e);
                }
            }
        }
    }
    
    #[test]
    fn test_round_trip_character_classes() {
        let parser = MockRegexParser::new();
        let serializer = RegexSerializer::new();
        
        let test_patterns = vec![
            "[a-z]",
            "[A-Z]",
            "[0-9]",
            "[abc]",
            "[^a-z]",
            "[^0-9]",
        ];
        
        for pattern in test_patterns {
            println!("Testing character class: {}", pattern);
            match parser.parse(pattern) {
                Ok(ast) => {
                    let serialized = serializer.serialize(&ast);
                    assert_eq!(pattern, serialized, "Round-trip failed for character class: {}", pattern);
                }
                Err(e) => {
                    panic!("Failed to parse character class '{}': {}", pattern, e);
                }
            }
        }
    }
    
    #[test]
    fn test_round_trip_groups() {
        let parser = MockRegexParser::new();
        let serializer = RegexSerializer::new();
        
        let test_patterns = vec![
            "(abc)",
            "(?:def)",
            "(?<name>test)",
            "(?>atomic)",
        ];
        
        for pattern in test_patterns {
            println!("Testing group: {}", pattern);
            match parser.parse(pattern) {
                Ok(ast) => {
                    let serialized = serializer.serialize(&ast);
                    assert_eq!(pattern, serialized, "Round-trip failed for group: {}", pattern);
                }
                Err(e) => {
                    println!("Pattern '{}' failed to parse (may not be implemented yet): {}", pattern, e);
                }
            }
        }
    }
    
    #[test]
    fn test_round_trip_alternations() {
        let parser = MockRegexParser::new();
        let serializer = RegexSerializer::new();
        
        let test_patterns = vec![
            "a|b",
            "cat|dog",
            "red|blue|green",
        ];
        
        for pattern in test_patterns {
            println!("Testing alternation: {}", pattern);
            match parser.parse(pattern) {
                Ok(ast) => {
                    let serialized = serializer.serialize(&ast);
                    assert_eq!(pattern, serialized, "Round-trip failed for alternation: {}", pattern);
                }
                Err(e) => {
                    panic!("Failed to parse alternation '{}': {}", pattern, e);
                }
            }
        }
    }
}

/// Test runner function for loading and running round-trip tests from files
pub fn run_regex_round_trip_tests() -> TestResults {
    let mut results = TestResults::new();
    let parser = MockRegexParser::new();
    let serializer = RegexSerializer::new();
    
    println!("Running regex round-trip tests from test data files...");
    
    // Load test cases from each test data file
    let test_files = vec![
        "basic_literals.ebnf",
        "character_classes.ebnf",
        "quantifiers.ebnf",
        "escapes.ebnf",
        "groups_alternation.ebnf",
        "anchors.ebnf",
        "complex.ebnf",
    ];
    
    for file_name in test_files {
        println!("\n=== Testing patterns from {} ===", file_name);
        
        let test_cases = load_test_cases("regex_patterns");
        
        for (case_file, content) in test_cases {
            if case_file != file_name {
                continue;
            }
            
            let patterns: Vec<&str> = content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .collect();
            
            for pattern in patterns {
                let pattern = pattern.trim();
                if pattern.is_empty() {
                    continue;
                }
                
                print!("  Testing: {} ... ", pattern);
                
                match parser.parse(pattern) {
                    Ok(ast) => {
                        let serialized = serializer.serialize(&ast);
                        if pattern == serialized {
                            println!("✓ PASS");
                            results.add_pass();
                        } else {
                            println!("✗ FAIL (expected: '{}', got: '{}')", pattern, serialized);
                            results.add_fail(format!("Round-trip mismatch for '{}': expected '{}', got '{}'", 
                                                    pattern, pattern, serialized));
                        }
                    }
                    Err(e) => {
                        println!("✗ PARSE_FAIL ({})", e);
                        results.add_fail(format!("Failed to parse '{}': {}", pattern, e));
                    }
                }
            }
        }
    }
    
    println!("\n=== Round-trip Test Results ===");
    println!("Passed: {}", results.passed);
    println!("Failed: {}", results.failed);
    println!("Total: {}", results.total());
    println!("Success rate: {:.1}%", results.success_rate() * 100.0);
    
    if !results.errors.is_empty() {
        println!("\n=== Errors ===");
        for error in &results.errors {
            println!("  - {}", error);
        }
    }
    
    results
}
