#!/usr/bin/perl
# Proposed refactored architecture for FSMGen supporting multiple target languages

package FSMGen::Core;
use strict;
use warnings;

# ===================================================================
# ABSTRACT SYNTAX TREE (Language-Independent)
# ===================================================================
package FSM::AST::Node {
    sub new { bless {}, shift }
    sub accept { die "Abstract method" }
}

package FSM::AST::Module {
    use parent 'FSM::AST::Node';
    sub new {
        my ($class, $name) = @_;
        bless {
            name => $name,
            ports => [],
            signals => {},
            states => [],
            transitions => [],
            processes => []
        }, $class;
    }
}

package FSM::AST::State {
    use parent 'FSM::AST::Node';
    sub new {
        my ($class, $name, $assignments) = @_;
        bless { name => $name, assignments => $assignments }, $class;
    }
}

# ===================================================================
# INTERMEDIATE REPRESENTATION (Optimized, Target-Independent)
# ===================================================================
package FSM::IR::Module {
    sub new {
        my ($class) = @_;
        bless {
            name => '',
            interface => FSM::IR::Interface->new(),
            signals => {},
            wen_hierarchy => {},  # Write-enable optimization
            state_machine => undef,
            datapath => undef
        }, $class;
    }
}

# ===================================================================
# BACKEND CODE GENERATORS (Language-Specific)
# ===================================================================
package FSM::Backend::CodeGenerator {
    sub new { bless { templates => {} }, shift }
    
    # Template-based generation
    sub render_module {
        my ($self, $ir_module) = @_;
        return $self->process_template('module', $ir_module);
    }
    
    sub process_template {
        my ($self, $template_name, $data) = @_;
        # Template processing logic
        die "Abstract method - must be implemented by subclasses";
    }
}

package FSM::Backend::VHDL {
    use parent 'FSM::Backend::CodeGenerator';
    
    sub new {
        my $class = shift;
        my $self = $class->SUPER::new();
        
        # VHDL-specific templates
        $self->{templates} = {
            module_header => sub {
                my $module = shift;
                return "ENTITY $module->{name} IS\n  PORT (\n";
            },
            
            port_declaration => sub {
                my ($name, $direction, $type, $width) = @_;
                my $vhdl_type = $width > 1 ? 
                    "STD_LOGIC_VECTOR(" . ($width-1) . " DOWNTO 0)" : 
                    "STD_LOGIC";
                return "    $name : $direction $vhdl_type";
            },
            
            state_machine => sub {
                my ($states, $transitions) = @_;
                my $vhdl = "-- State machine process\n";
                $vhdl .= "process(clk, reset)\nbegin\n";
                # Generate VHDL state machine
                return $vhdl;
            }
        };
        
        return $self;
    }
    
    sub process_template {
        my ($self, $template_name, $data) = @_;
        return $self->{templates}{$template_name}->($data);
    }
}

package FSM::Backend::SystemVerilog {
    use parent 'FSM::Backend::CodeGenerator';
    
    sub new {
        my $class = shift;
        my $self = $class->SUPER::new();
        
        # SystemVerilog-specific templates  
        $self->{templates} = {
            module_header => sub {
                my $module = shift;
                return "module $module->{name} (\n";
            },
            
            port_declaration => sub {
                my ($name, $direction, $type, $width) = @_;
                my $sv_direction = lc($direction);
                my $sv_type = $width > 1 ? "logic [" . ($width-1) . ":0]" : "logic";
                return "    $sv_direction $sv_type $name";
            },
            
            state_machine => sub {
                my ($states, $transitions) = @_;
                my $sv = "// State machine\n";
                $sv .= "always_ff \@(posedge clk or negedge reset) begin\n";
                # Generate SystemVerilog state machine
                return $sv;
            },
            
            enum_declaration => sub {
                my ($states) = @_;
                return "typedef enum logic [" . 
                       (int(log(scalar(@$states))/log(2))) . ":0] {\n" .
                       join(",\n", map { "    $_" } @$states) . "\n} state_t;\n";
            }
        };
        
        return $self;
    }
}

# ===================================================================
# MAIN COMPILER PIPELINE  
# ===================================================================
package FSMGen::Compiler {
    sub new {
        my ($class, $target_language) = @_;
        
        my $backend;
        if ($target_language eq 'vhdl') {
            $backend = FSM::Backend::VHDL->new();
        } elsif ($target_language eq 'systemverilog') {
            $backend = FSM::Backend::SystemVerilog->new();
        } else {
            die "Unsupported target: $target_language";
        }
        
        bless {
            parser => FSM::Parser->new(),
            optimizer => FSM::Optimizer->new(),
            backend => $backend
        }, $class;
    }
    
    sub compile {
        my ($self, $fsm_file) = @_;
        
        # 1. Parse FSM file to AST (language-independent)
        my $ast = $self->{parser}->parse_file($fsm_file);
        
        # 2. Convert AST to optimized IR (language-independent) 
        my $ir = $self->{optimizer}->optimize($ast);
        
        # 3. Generate target language code (language-specific)
        return $self->{backend}->generate_code($ir);
    }
}

# ===================================================================
# USAGE EXAMPLES
# ===================================================================

# Generate VHDL
my $vhdl_compiler = FSMGen::Compiler->new('vhdl');
my $vhdl_code = $vhdl_compiler->compile('example.fsm');

# Generate SystemVerilog  
my $sv_compiler = FSMGen::Compiler->new('systemverilog');
my $sv_code = $sv_compiler->compile('example.fsm');

# Future: Generate Chisel, SpinalHDL, etc.
# my $chisel_compiler = FSMGen::Compiler->new('chisel');

1;
