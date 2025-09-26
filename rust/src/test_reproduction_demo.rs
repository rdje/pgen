//! Demonstration of Test Target Mapper functionality
//! Shows how stress tests can provide reproduction commands

use crate::test_target_mapper::TestTargetMapper;

/// Demonstrates the test target mapping functionality
pub fn demo_test_target_mapping() {
    let mapper = TestTargetMapper::new();
    
    println!("🎯 TEST TARGET MAPPER DEMONSTRATION");
    println!("==================================");
    
    // Demo return annotation test mapping
    println!("\n📋 Return Annotation Test Cases:");
    let return_cases = mapper.get_return_test_cases();
    for case in &return_cases {
        let cmd = mapper.get_reproduction_command("return", case);
        println!("  Input: {:20} → {}", case, cmd);
    }
    
    // Demo semantic annotation test mapping  
    println!("\n📋 Semantic Annotation Test Cases:");
    let semantic_cases = mapper.get_semantic_test_cases();
    for case in &semantic_cases {
        let cmd = mapper.get_reproduction_command("semantic", case);
        println!("  Input: {:30} → {}", case, cmd);
    }
    
    // Demo regex test mapping
    println!("\n📋 Regex Test Cases:");
    let regex_cases = mapper.get_regex_test_cases();
    for case in &regex_cases {
        let cmd = mapper.get_reproduction_command("regex", case);
        println!("  Input: {:15} → {}", case, cmd);
    }
    
    // Demo error scenario
    println!("\n🚨 Example Error Scenario with Reproduction:");
    let test_input = "$1";
    let reproduction_cmd = mapper.get_reproduction_command("return", test_input);
    println!("  ❌ PARSE FAILED: Return parser failed on '{}': SomeError", test_input);
    println!("  🔧 REPRODUCE THIS FAILURE: {}", reproduction_cmd);
    
    println!("\n💡 When stress tests fail, they now show exactly which 'make' command");
    println!("   to run to reproduce that specific failure individually!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn demo_reproduction_commands() {
        demo_test_target_mapping();
    }
}