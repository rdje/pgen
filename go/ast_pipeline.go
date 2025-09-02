// Package ast_pipeline provides Go AST Pipeline Implementation
//
// Provides complete EBNF AST transformation pipeline with dual-mode API:
// - Same-language optimization: In-memory data structures
// - Cross-language interface: JSON input/output
//
// Implements the 5-stage transformation pipeline equivalent to Perl AST::Transform.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"
)

// PipelineConfig holds configuration for AST transformation pipeline
type PipelineConfig struct {
	Debug               bool `json:"debug"`
	PreserveAnnotations bool `json:"preserve_annotations"`
	ValidateInput       bool `json:"validate_input"`
	ValidateOutput      bool `json:"validate_output"`
	MaxRecursionDepth   int  `json:"max_recursion_depth"`
}

// DefaultConfig returns default pipeline configuration
func DefaultConfig() PipelineConfig {
	return PipelineConfig{
		Debug:               false,
		PreserveAnnotations: true,
		ValidateInput:       true,
		ValidateOutput:      true,
		MaxRecursionDepth:   100,
	}
}

// Token represents a raw AST token
type Token []string

// TokenSequence represents a sequence of tokens  
type TokenSequence []Token

// RawAST represents the complete raw AST
type RawAST []TokenSequence

// RawASTJson represents raw AST JSON structure from ebnf_to_json.pl
type RawASTJson struct {
	GrammarName string                 `json:"grammar_name"`
	RawAST      RawAST                 `json:"raw_ast"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// ASTNode represents a node in the transformed AST
type ASTNode struct {
	Type         string      `json:"type"`
	Value        interface{} `json:"value,omitempty"`
	Elements     []ASTNode   `json:"elements,omitempty"`
	Alternatives []ASTNode   `json:"alternatives,omitempty"`
	Element      *ASTNode    `json:"element,omitempty"`
	Quantifier   string      `json:"quantifier,omitempty"`
}

// NewAtomNode creates a new atom node
func NewAtomNode(value interface{}) ASTNode {
	return ASTNode{
		Type:  "atom",
		Value: value,
	}
}

// NewSequenceNode creates a new sequence node
func NewSequenceNode(elements []ASTNode) ASTNode {
	return ASTNode{
		Type:     "sequence",
		Elements: elements,
	}
}

// NewOrNode creates a new OR node
func NewOrNode(alternatives []ASTNode) ASTNode {
	return ASTNode{
		Type:         "or",
		Alternatives: alternatives,
	}
}

// NewQuantifiedNode creates a new quantified node
func NewQuantifiedNode(element ASTNode, quantifier string) ASTNode {
	return ASTNode{
		Type:       "quantified",
		Element:    &element,
		Quantifier: quantifier,
	}
}

// Annotations holds preserved annotations from raw AST
type Annotations struct {
	SemanticAnnotations map[string][]string `json:"semantic_annotations"`
	LoggingAnnotations  map[string][]string `json:"logging_annotations"`
	ReturnAnnotations   map[string]string   `json:"return_annotations"`
}

// NewAnnotations creates new empty annotations
func NewAnnotations() Annotations {
	return Annotations{
		SemanticAnnotations: make(map[string][]string),
		LoggingAnnotations:  make(map[string][]string),
		ReturnAnnotations:   make(map[string]string),
	}
}

// TransformStats holds transformation statistics
type TransformStats struct {
	RulesProcessed        int `json:"rules_processed"`
	AnnotationsPreserved  int `json:"annotations_preserved"`
	TransformationsApplied int `json:"transformations_applied"`
}

// TransformMetadata holds metadata for transformed AST JSON
type TransformMetadata struct {
	Format         string        `json:"format"`
	SourceFormat   string        `json:"source_format"`
	TransformedAt  string        `json:"transformed_at"`
	Transformer    string        `json:"transformer"`
	PipelineStage  string        `json:"pipeline_stage"`
	Annotations    Annotations   `json:"annotations"`
	Stats          TransformStats `json:"stats"`
}

// TransformedASTJson represents transformed AST JSON structure
type TransformedASTJson struct {
	GrammarName  string             `json:"grammar_name"`
	GrammarTree  map[string]ASTNode `json:"grammar_tree"`
	RuleOrder    []string           `json:"rule_order"`
	Metadata     TransformMetadata  `json:"metadata"`
}

// GoASTPipeline implements the AST transformation pipeline in Go
type GoASTPipeline struct {
	config      PipelineConfig
	stats       TransformStats
	annotations Annotations
}

// NewGoASTPipeline creates a new Go AST pipeline with configuration
func NewGoASTPipeline(config PipelineConfig) *GoASTPipeline {
	return &GoASTPipeline{
		config:      config,
		stats:       TransformStats{},
		annotations: NewAnnotations(),
	}
}

// LoadRawAST loads raw AST JSON from file
func (p *GoASTPipeline) LoadRawAST(filePath string) (*RawASTJson, error) {
	if p.config.Debug {
		fmt.Printf("Loading raw AST from: %s\n", filePath)
	}

	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read file %s: %w", filePath, err)
	}

	var rawData RawASTJson
	if err := json.Unmarshal(data, &rawData); err != nil {
		return nil, fmt.Errorf("failed to parse JSON from %s: %w", filePath, err)
	}

	if p.config.ValidateInput {
		if err := p.validateRawAST(&rawData); err != nil {
			return nil, err
		}
	}

	return &rawData, nil
}

// validateRawAST validates raw AST JSON format
func (p *GoASTPipeline) validateRawAST(data *RawASTJson) error {
	if data.GrammarName == "" {
		return fmt.Errorf("raw AST JSON missing grammar_name")
	}

	if len(data.RawAST) == 0 {
		return fmt.Errorf("raw AST JSON has empty raw_ast array")
	}

	if format, ok := data.Metadata["format"]; ok {
		if fmt.Sprintf("%v", format) != "raw_ast" {
			return fmt.Errorf("metadata.format must be 'raw_ast'")
		}
	}

	return nil
}

// TransformRawAST transforms raw AST to semantic AST using the 5-stage pipeline
func (p *GoASTPipeline) TransformRawAST(rawAST RawAST) (map[string]ASTNode, []string, error) {
	if p.config.Debug {
		fmt.Println("=== Go AST Transformation Pipeline ===")
	}

	// Stage 1: Extract annotations
	cleanedAST, err := p.extractAnnotations(rawAST)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 1 failed: %w", err)
	}

	// Stage 2: Group by OR operators
	groupedRules, err := p.groupByOrOperators(cleanedAST)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 2 failed: %w", err)
	}

	// Stage 2.5: Handle parentheses
	processedRules, err := p.handleParentheses(groupedRules)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 2.5 failed: %w", err)
	}

	// Stage 3: Parse sequences
	sequencedRules, err := p.parseSequences(processedRules)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 3 failed: %w", err)
	}

	// Stage 4: Handle quantifiers
	quantifiedRules, err := p.handleQuantifiers(sequencedRules)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 4 failed: %w", err)
	}

	// Stage 5: Build tree structure
	grammarTree, ruleOrder, err := p.buildTreeStructure(quantifiedRules)
	if err != nil {
		return nil, nil, fmt.Errorf("stage 5 failed: %w", err)
	}

	p.stats.RulesProcessed = len(grammarTree)
	p.stats.TransformationsApplied = 5

	return grammarTree, ruleOrder, nil
}

// extractAnnotations extracts and preserves annotations from raw AST (Stage 1)
func (p *GoASTPipeline) extractAnnotations(rawAST RawAST) (RawAST, error) {
	if p.config.Debug {
		fmt.Println("Stage 1: Extracting annotations...")
	}

	var cleanedAST RawAST

	for _, ruleDef := range rawAST {
		if len(ruleDef) == 0 {
			continue
		}

		var ruleName string
		var cleanedRule TokenSequence

		for _, token := range ruleDef {
			if len(token) != 2 {
				continue
			}

			tokenType, tokenValue := token[0], token[1]

			switch tokenType {
			case "rule":
				ruleName = tokenValue
				cleanedRule = append(cleanedRule, token)
		case "semantic_annotation", "logging_annotation":
			if ruleName != "" && p.config.PreserveAnnotations {
				// Parse annotation format: ["annotation_type", [name, value]] for semantic
				// or ["annotation_type", [name, [args...]]] for logging
				var parsedValue []interface{}
				if err := json.Unmarshal([]byte(tokenValue), &parsedValue); err == nil {
					if len(parsedValue) >= 2 {
						annotationName := fmt.Sprintf("%v", parsedValue[0])
						
						if tokenType == "semantic_annotation" {
							if p.annotations.SemanticAnnotations[ruleName] == nil {
								p.annotations.SemanticAnnotations[ruleName] = []string{}
							}
							annotationValue := fmt.Sprintf("%v", parsedValue[1])
							formattedAnnotation := fmt.Sprintf("%s:%s", annotationName, annotationValue)
							p.annotations.SemanticAnnotations[ruleName] = append(
								p.annotations.SemanticAnnotations[ruleName], formattedAnnotation)
							
						} else if tokenType == "logging_annotation" {
							if p.annotations.LoggingAnnotations[ruleName] == nil {
								p.annotations.LoggingAnnotations[ruleName] = []string{}
							}
							var args string
							if argsArray, ok := parsedValue[1].([]interface{}); ok {
								var argStrs []string
								for _, arg := range argsArray {
									argStrs = append(argStrs, fmt.Sprintf("%v", arg))
								}
								args = strings.Join(argStrs, ",")
							} else {
								args = fmt.Sprintf("%v", parsedValue[1])
							}
							formattedAnnotation := fmt.Sprintf("%s(%s)", annotationName, args)
							p.annotations.LoggingAnnotations[ruleName] = append(
								p.annotations.LoggingAnnotations[ruleName], formattedAnnotation)
						}
					}
				} else {
					// Fallback for malformed annotation data
					if tokenType == "semantic_annotation" {
						if p.annotations.SemanticAnnotations[ruleName] == nil {
							p.annotations.SemanticAnnotations[ruleName] = []string{}
						}
						p.annotations.SemanticAnnotations[ruleName] = append(
							p.annotations.SemanticAnnotations[ruleName], fmt.Sprintf("raw:%s", tokenValue))
					} else if tokenType == "logging_annotation" {
						if p.annotations.LoggingAnnotations[ruleName] == nil {
							p.annotations.LoggingAnnotations[ruleName] = []string{}
						}
						p.annotations.LoggingAnnotations[ruleName] = append(
							p.annotations.LoggingAnnotations[ruleName], fmt.Sprintf("raw:%s", tokenValue))
					}
				}
				p.stats.AnnotationsPreserved++
			}
			// Don't add to cleaned rule
			case "return_scalar", "return_array", "return_object":
				if ruleName != "" && p.config.PreserveAnnotations {
					p.annotations.ReturnAnnotations[ruleName] = tokenType
				}
				// Don't add to cleaned rule
			default:
				cleanedRule = append(cleanedRule, token)
			}
		}

		if len(cleanedRule) > 0 {
			cleanedAST = append(cleanedAST, cleanedRule)
		}
	}

	if p.config.Debug {
		fmt.Printf("Preserved %d annotations\n", p.stats.AnnotationsPreserved)
	}

	return cleanedAST, nil
}

// groupByOrOperators groups rule definitions by OR operators (Stage 2)
func (p *GoASTPipeline) groupByOrOperators(ast RawAST) (map[string][]TokenSequence, error) {
	if p.config.Debug {
		fmt.Println("Stage 2: Grouping by OR operators...")
	}

	grouped := make(map[string][]TokenSequence)

	for _, ruleDef := range ast {
		if len(ruleDef) == 0 {
			continue
		}

		var ruleName string
		for _, token := range ruleDef {
			if len(token) == 2 && token[0] == "rule" {
				ruleName = token[1]
				break
			}
		}

		if ruleName != "" {
			var alternatives []TokenSequence
			var currentAlt TokenSequence

			// Skip rule definition token
			for _, token := range ruleDef[1:] {
				if len(token) == 2 && token[0] == "operator" && token[1] == "|" {
					if len(currentAlt) > 0 {
						alternatives = append(alternatives, currentAlt)
						currentAlt = TokenSequence{}
					}
				} else {
					currentAlt = append(currentAlt, token)
				}
			}

			if len(currentAlt) > 0 {
				alternatives = append(alternatives, currentAlt)
			}

			if grouped[ruleName] == nil {
				grouped[ruleName] = []TokenSequence{}
			}
			grouped[ruleName] = append(grouped[ruleName], alternatives...)
		}
	}

	return grouped, nil
}

// handleParentheses handles parentheses and grouping (Stage 2.5)
func (p *GoASTPipeline) handleParentheses(groupedRules map[string][]TokenSequence) (map[string][]TokenSequence, error) {
	if p.config.Debug {
		fmt.Println("Stage 2.5: Handling parentheses...")
	}

	processed := make(map[string][]TokenSequence)

	for ruleName, alternatives := range groupedRules {
		var processedAlts []TokenSequence

		for _, alt := range alternatives {
			processedAlt, err := p.processParenthesesInSequence(alt)
			if err != nil {
				return nil, err
			}
			processedAlts = append(processedAlts, processedAlt)
		}

		processed[ruleName] = processedAlts
	}

	return processed, nil
}

// processParenthesesInSequence processes parentheses within a token sequence
func (p *GoASTPipeline) processParenthesesInSequence(sequence TokenSequence) (TokenSequence, error) {
	var result TokenSequence
	i := 0

	for i < len(sequence) {
		token := sequence[i]

		if len(token) == 2 && token[0] == "group_open" {
			// Find matching close
			parenCount := 1
			j := i + 1
			var groupContent TokenSequence

			for j < len(sequence) && parenCount > 0 {
				if len(sequence[j]) == 2 {
					switch sequence[j][0] {
					case "group_open":
						parenCount++
					case "group_close":
						parenCount--
					}
				}

				if parenCount > 0 {
					groupContent = append(groupContent, sequence[j])
				}
				j++
			}

			if len(groupContent) > 0 {
				// Create group token - serialize content as JSON
				contentJSON, err := json.Marshal(groupContent)
				if err != nil {
					return nil, fmt.Errorf("failed to serialize group content: %w", err)
				}
				result = append(result, Token{"group", string(contentJSON)})
			}

			i = j
		} else {
			result = append(result, token)
			i++
		}
	}

	return result, nil
}

// parseSequences parses sequences of grammar elements (Stage 3)
func (p *GoASTPipeline) parseSequences(processedRules map[string][]TokenSequence) (map[string][]ASTNode, error) {
	if p.config.Debug {
		fmt.Println("Stage 3: Parsing sequences...")
	}

	sequenced := make(map[string][]ASTNode)

	for ruleName, alternatives := range processedRules {
		var parsedAlts []ASTNode

		for _, alt := range alternatives {
			var parsedAlt ASTNode
			if len(alt) == 1 {
				var err error
				parsedAlt, err = p.parseSingleElement(alt[0])
				if err != nil {
					return nil, err
				}
			} else {
				var elements []ASTNode
				for _, elem := range alt {
					element, err := p.parseSingleElement(elem)
					if err != nil {
						return nil, err
					}
					elements = append(elements, element)
				}
				parsedAlt = NewSequenceNode(elements)
			}
			parsedAlts = append(parsedAlts, parsedAlt)
		}

		sequenced[ruleName] = parsedAlts
	}

	return sequenced, nil
}

// parseSingleElement parses a single grammar element
func (p *GoASTPipeline) parseSingleElement(element Token) (ASTNode, error) {
	if len(element) != 2 {
		return NewAtomNode(element), nil
	}

	tokenType, tokenValue := element[0], element[1]

	if tokenType == "group" {
		// Deserialize group content
		var groupContent TokenSequence
		if err := json.Unmarshal([]byte(tokenValue), &groupContent); err != nil {
			return ASTNode{}, fmt.Errorf("failed to deserialize group content: %w", err)
		}

		if len(groupContent) == 1 {
			return p.parseSingleElement(groupContent[0])
		} else {
			var elements []ASTNode
			for _, elem := range groupContent {
				element, err := p.parseSingleElement(elem)
				if err != nil {
					return ASTNode{}, err
				}
				elements = append(elements, element)
			}
			return NewSequenceNode(elements), nil
		}
	} else {
		return NewAtomNode(element), nil
	}
}

// handleQuantifiers handles quantifiers (*, +, ?) (Stage 4)
func (p *GoASTPipeline) handleQuantifiers(sequencedRules map[string][]ASTNode) (map[string][]ASTNode, error) {
	if p.config.Debug {
		fmt.Println("Stage 4: Handling quantifiers...")
	}

	quantified := make(map[string][]ASTNode)

	for ruleName, alternatives := range sequencedRules {
		var processedAlts []ASTNode

		for _, alt := range alternatives {
			processedAlt, err := p.applyQuantifiersToNode(alt)
			if err != nil {
				return nil, err
			}
			processedAlts = append(processedAlts, processedAlt)
		}

		quantified[ruleName] = processedAlts
	}

	return quantified, nil
}

// applyQuantifiersToNode applies quantifiers to AST node
func (p *GoASTPipeline) applyQuantifiersToNode(node ASTNode) (ASTNode, error) {
	if node.Type == "sequence" {
		var newElements []ASTNode
		i := 0

		for i < len(node.Elements) {
			element := node.Elements[i]

			// Check if next element is a quantifier
			if i+1 < len(node.Elements) {
				nextElem := node.Elements[i+1]
				if nextElem.Type == "atom" {
					if token, ok := nextElem.Value.([]interface{}); ok && len(token) == 2 {
						if tokenType, ok := token[0].(string); ok && tokenType == "operator" {
							if quantifier, ok := token[1].(string); ok {
								if quantifier == "*" || quantifier == "+" || quantifier == "?" {
									quantifiedNode := NewQuantifiedNode(element, quantifier)
									newElements = append(newElements, quantifiedNode)
									i += 2 // Skip quantifier token
									continue
								}
							}
						}
					}
				}
			}

			newElements = append(newElements, element)
			i++
		}

		return NewSequenceNode(newElements), nil
	}

	return node, nil
}

// buildTreeStructure builds final tree structure (Stage 5)
func (p *GoASTPipeline) buildTreeStructure(quantifiedRules map[string][]ASTNode) (map[string]ASTNode, []string, error) {
	if p.config.Debug {
		fmt.Println("Stage 5: Building tree structure...")
	}

	grammarTree := make(map[string]ASTNode)
	var ruleOrder []string

	for ruleName := range quantifiedRules {
		ruleOrder = append(ruleOrder, ruleName)
	}

	for ruleName, alternatives := range quantifiedRules {
		var finalNode ASTNode
		if len(alternatives) == 1 {
			finalNode = alternatives[0]
		} else {
			finalNode = NewOrNode(alternatives)
		}

		grammarTree[ruleName] = finalNode
	}

	return grammarTree, ruleOrder, nil
}

// SaveTransformedAST saves transformed AST to JSON file
func (p *GoASTPipeline) SaveTransformedAST(grammarTree map[string]ASTNode, ruleOrder []string, grammarName, outputFile string) error {
	if p.config.Debug {
		fmt.Printf("Saving transformed AST to: %s\n", outputFile)
	}

	metadata := TransformMetadata{
		Format:         "transformed_ast",
		SourceFormat:   "raw_ast",
		TransformedAt:  time.Now().UTC().Format(time.RFC3339),
		Transformer:    "Go AST Pipeline v1.0",
		PipelineStage:  "transformation",
		Annotations:    p.annotations,
		Stats:          p.stats,
	}

	transformedData := TransformedASTJson{
		GrammarName: grammarName,
		GrammarTree: grammarTree,
		RuleOrder:   ruleOrder,
		Metadata:    metadata,
	}

	jsonData, err := json.MarshalIndent(transformedData, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to serialize transformed AST: %w", err)
	}

	if err := os.WriteFile(outputFile, jsonData, 0644); err != nil {
		return fmt.Errorf("failed to write file %s: %w", outputFile, err)
	}

	if p.config.Debug {
		fmt.Println("Transformed AST saved successfully")
	}

	return nil
}

// TransformFromFile transforms raw AST JSON file to in-memory AST (same-language API)
func (p *GoASTPipeline) TransformFromFile(rawASTJSONFile string, outputJSONFile *string) (map[string]ASTNode, []string, error) {
	rawData, err := p.LoadRawAST(rawASTJSONFile)
	if err != nil {
		return nil, nil, err
	}

	grammarTree, ruleOrder, err := p.TransformRawAST(rawData.RawAST)
	if err != nil {
		return nil, nil, err
	}

	if outputJSONFile != nil {
		if err := p.SaveTransformedAST(grammarTree, ruleOrder, rawData.GrammarName, *outputJSONFile); err != nil {
			return nil, nil, err
		}
	}

	return grammarTree, ruleOrder, nil
}

// TransformToJSON transforms raw AST JSON file to transformed AST JSON file (cross-language API)
func (p *GoASTPipeline) TransformToJSON(rawASTJSONFile, outputJSONFile string) error {
	grammarTree, ruleOrder, err := p.TransformFromFile(rawASTJSONFile, nil)
	if err != nil {
		return err
	}

	rawData, err := p.LoadRawAST(rawASTJSONFile)
	if err != nil {
		return err
	}

	return p.SaveTransformedAST(grammarTree, ruleOrder, rawData.GrammarName, outputJSONFile)
}

// main function for CLI usage
func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: %s input_raw.json [output_transformed.json] [--debug] [--stats]\n", os.Args[0])
		os.Exit(1)
	}

	inputFile := os.Args[1]
	var outputFile *string
	debug := false
	stats := false

	// Parse command line arguments
	for i, arg := range os.Args[2:] {
		switch arg {
		case "--debug", "-d":
			debug = true
		case "--stats", "-s":
			stats = true
		default:
			if i == 0 && !strings.HasPrefix(arg, "--") {
				outputFile = &arg
			}
		}
	}

	config := DefaultConfig()
	config.Debug = debug

	pipeline := NewGoASTPipeline(config)

	if outputFile != nil {
		// Cross-language mode: JSON → JSON
		if err := pipeline.TransformToJSON(inputFile, *outputFile); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("Transformed AST saved to: %s\n", *outputFile)
	} else {
		// Same-language mode: JSON → In-memory
		grammarTree, ruleOrder, err := pipeline.TransformFromFile(inputFile, nil)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("Transformed AST loaded in-memory: %d rules\n", len(grammarTree))
		fmt.Printf("Rule order: %s\n", strings.Join(ruleOrder, ", "))
	}

	if stats {
		fmt.Printf("\nTransformation Statistics:\n")
		fmt.Printf("  Rules processed: %d\n", pipeline.stats.RulesProcessed)
		fmt.Printf("  Annotations preserved: %d\n", pipeline.stats.AnnotationsPreserved)
		fmt.Printf("  Transformations applied: %d\n", pipeline.stats.TransformationsApplied)
		fmt.Printf("  Pipeline: Go AST Pipeline v1.0\n")
	}
}
