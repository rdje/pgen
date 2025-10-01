use pgen::ast_pipeline::{RustASTPipeline, PipelineConfig};

#[test]
fn test_branch_return_annotations() {
    // Create pipeline with debug enabled
    let config = PipelineConfig {
        debug: true,
        trace: true,
        preserve_annotations: true,
        ..Default::default()
    };
    
    let mut pipeline = RustASTPipeline::new(config);
    
    // Transform the test file
    let json_file = "../test/grammars/branch_return_test.json";
    let result = pipeline.transform_from_file(json_file, None);
    
    match result {
        Ok((grammar_tree, rule_order)) => {
            println!("✅ Transformation successful!");
            println!("Rules: {:?}", rule_order);
            
            // Check that we have branch return annotations
            let branch_annotations = pipeline.get_all_branch_return_annotations();
            println!("\n📌 Branch return annotations found:");
            
            for (rule_name, branches) in branch_annotations {
                println!("\nRule '{}' has {} branches:", rule_name, branches.len());
                for (idx, branch_ann) in branches.iter().enumerate() {
                    if let Some(ann) = branch_ann {
                        println!("  Branch {}: {} - {}", idx, ann.annotation_type, ann.annotation_content);
                    } else {
                        println!("  Branch {}: No annotation", idx);
                    }
                }
            }
            
            // Verify specific rules have the expected number of branches with annotations
            assert!(branch_annotations.contains_key("value"), "Should have 'value' rule");
            if let Some(value_branches) = branch_annotations.get("value") {
                assert_eq!(value_branches.len(), 4, "value rule should have 4 branches");
                for branch in value_branches {
                    assert!(branch.is_some(), "Each branch of 'value' should have an annotation");
                }
            }
            
            assert!(branch_annotations.contains_key("identifier"), "Should have 'identifier' rule");
            if let Some(id_branches) = branch_annotations.get("identifier") {
                assert_eq!(id_branches.len(), 3, "identifier rule should have 3 branches");
                for branch in id_branches {
                    assert!(branch.is_some(), "Each branch of 'identifier' should have an annotation");
                }
            }
            
            assert!(branch_annotations.contains_key("list"), "Should have 'list' rule");
            if let Some(list_branches) = branch_annotations.get("list") {
                assert_eq!(list_branches.len(), 3, "list rule should have 3 branches");
                for branch in list_branches {
                    assert!(branch.is_some(), "Each branch of 'list' should have an annotation");
                }
            }
            
            assert!(branch_annotations.contains_key("result"), "Should have 'result' rule");
            if let Some(result_branches) = branch_annotations.get("result") {
                assert_eq!(result_branches.len(), 3, "result rule should have 3 branches");
                for branch in result_branches {
                    assert!(branch.is_some(), "Each branch of 'result' should have an annotation");
                }
            }
            
            println!("\n✅ All assertions passed!");
        }
        Err(e) => {
            panic!("Failed to transform: {}", e);
        }
    }
}