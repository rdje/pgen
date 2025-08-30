#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../perl";

use AST::UniversalReturnAnnotation;
use AST::UniversalComposer;
use AST::PerlReturnCodeGenerator;
use Data::Dumper;

print "=== Testing Universal Return Annotation System ===\n\n";

# Test 1: Simple scalar reference $1
print "Test 1: Simple scalar reference \$1\n";
my $scalar_ast = AST::UniversalReturnAnnotation::new_scalar_ref(1);
print "AST: " . AST::UniversalReturnAnnotation::ast_to_string($scalar_ast) . "\n";

my $generator = AST::PerlReturnCodeGenerator->new();
my $perl_code = AST::UniversalComposer::compose_return_expression(
    $scalar_ast, $generator, '\\@results'
);
print "Generated Perl: $perl_code\n";
print "Expected: \$\\\@results[0]\n\n";

# Test 2: Quantified array [$1*] - This is for dot_path := accessor+ -> [$1*]
print "Test 2: Quantified array [\$1*] (dot_path case)\n";
my $quantified_ast = AST::UniversalReturnAnnotation::new_quantified_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1)
);
print "AST: " . AST::UniversalReturnAnnotation::ast_to_string($quantified_ast) . "\n";

my $quantified_code = AST::UniversalComposer::compose_return_expression(
    $quantified_ast, $generator, '\\@results'
);
print "Generated Perl: $quantified_code\n";
print "Expected: collect_quantified_results(1, \\\\\@results)\n\n";

# Test 3: Simple object {type: "property", name: $2}
print "Test 3: Simple object {type: \"property\", name: \$2}\n";
my $object_ast = AST::UniversalReturnAnnotation::new_object(
    {key => 'type', value => AST::UniversalReturnAnnotation::new_literal('property')},
    {key => 'name', value => AST::UniversalReturnAnnotation::new_scalar_ref(2)}
);
print "AST: " . AST::UniversalReturnAnnotation::ast_to_string($object_ast) . "\n";

my $object_code = AST::UniversalComposer::compose_return_expression(
    $object_ast, $generator, '\\@results'
);
print "Generated Perl: $object_code\n";
print "Expected: {\"type\" => \"property\", \"name\" => \$\\\@results[1]}\n\n";

# Test 4: Simple array [$1, $2]
print "Test 4: Simple array [\$1, \$2]\n";
my $array_ast = AST::UniversalReturnAnnotation::new_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1),
    AST::UniversalReturnAnnotation::new_scalar_ref(2)
);
print "AST: " . AST::UniversalReturnAnnotation::ast_to_string($array_ast) . "\n";

my $array_code = AST::UniversalComposer::compose_return_expression(
    $array_ast, $generator, '\\@results'
);
print "Generated Perl: $array_code\n";
print "Expected: [\$\\\@results[0], \$\\\@results[1]]\n\n";

# Test 5: Mixed array [$1, $3*] - like index_list := index (',' /\s*/ index)* -> [$1, $3*]
print "Test 5: Mixed array [\$1, \$3*]\n";
my $mixed_ast = AST::UniversalReturnAnnotation::new_mixed_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1),
    AST::UniversalReturnAnnotation::new_quantified_collection(
        AST::UniversalReturnAnnotation::new_scalar_ref(3)
    )
);
print "AST: " . AST::UniversalReturnAnnotation::ast_to_string($mixed_ast) . "\n";

my $mixed_code = AST::UniversalComposer::compose_return_expression(
    $mixed_ast, $generator, '\\@results'
);
print "Generated Perl: $mixed_code\n";
print "Expected: [\$\\\@results[0], \@{collect_quantified_results(3, \\\\\@results)}]\n\n";

# Test 6: Branch annotation for accessor := property_accessor -> {type: "property", name: $1}
print "Test 6: Branch annotation example\n";
my $property_return_ast = AST::UniversalReturnAnnotation::new_object(
    {key => 'type', value => AST::UniversalReturnAnnotation::new_literal('property')},
    {key => 'name', value => AST::UniversalReturnAnnotation::new_scalar_ref(1)}
);

my $branch_annotation = AST::UniversalReturnAnnotation::new_branch_annotation(
    'accessor', 0, $property_return_ast
);

print "Branch: " . Dumper($branch_annotation) . "\n";

my $branch_return_code = AST::UniversalComposer::compose_branch_return(
    $branch_annotation, $generator, '\\@results'
);
print "Generated branch return: $branch_return_code\n";
print "Expected: return {\"type\" => \"property\", \"name\" => \$\\\@results[0]};\n\n";

# Test 7: Validation
print "Test 7: Validation\n";
print "Valid scalar AST: " . (AST::UniversalReturnAnnotation::validate_ast($scalar_ast) ? "PASS" : "FAIL") . "\n";
print "Valid quantified AST: " . (AST::UniversalReturnAnnotation::validate_ast($quantified_ast) ? "PASS" : "FAIL") . "\n";
print "Valid object AST: " . (AST::UniversalReturnAnnotation::validate_ast($object_ast) ? "PASS" : "FAIL") . "\n";

# Test invalid AST
my $invalid_ast = {type => 'unknown', data => 'invalid'};
print "Invalid AST: " . (AST::UniversalReturnAnnotation::validate_ast($invalid_ast) ? "FAIL" : "PASS") . "\n\n";

print "=== Tests Complete ===\n";

# Show how this would be used in the actual parser generator
print "\n=== Integration Example ===\n";
print "For the rule: dot_path := accessor+ -> [\$1*]\n";
print "The parser generator would:\n";
print "1. Parse the return annotation '[\$1*]' into AST\n";
print "2. Call compose_return_expression(ast, generator, '\\\\\@results')\n";
print "3. Get: $quantified_code\n";
print "4. Generate the complete function:\n\n";

my $function_template = <<'EOF';
sub parse_dot_path {
    my ($input) = @_;
    my @results = ();
    
    # Use quantified_rule to parse one or more accessors
    my $result_1 = quantified_rule($input, \&parse_accessor, 1, 999);
    unless (defined $result_1) {
        return undef;
    }
    push @results, $result_1;
    
    # Generated return code using universal composition:
    return RETURN_CODE_HERE;
}
EOF

$function_template =~ s/RETURN_CODE_HERE/$quantified_code/;
print $function_template;
