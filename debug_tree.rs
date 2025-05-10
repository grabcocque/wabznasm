use tree_sitter::{Node, Parser as Tsparser};

include!(concat!(env!("OUT_DIR"), "/calc.rs"));

fn print_tree_recursive(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");

    println!(
        "{}Node: {} '{}' [{}-{}]",
        indent,
        node.kind(),
        node_text,
        node.start_byte(),
        node.end_byte()
    );

    // Print field names if any
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(field_name) = node.field_name_for_child(i as u32) {
                println!("{}  Field '{}' ->", indent, field_name);
                print_tree_recursive(child, source, depth + 2);
            } else {
                println!("{}  Child {} ->", indent, i);
                print_tree_recursive(child, source, depth + 2);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "1+2";

    let mut parser = Tsparser::new();
    parser.set_language(&language())?;

    let tree = parser.parse(input, None).ok_or("Failed to parse")?;
    let root = tree.root_node();

    println!("=== Parse Tree for '{}' ===", input);
    print_tree_recursive(root, input, 0);

    println!("\n=== Node Info Summary ===");
    println!("Root node: {}", root.kind());
    println!("Has error: {}", root.has_error());
    println!("Child count: {}", root.child_count());

    // Walk through the tree to find the additive operation
    fn find_additive(node: Node, source: &str, path: &str) {
        if node.kind() == "additive" {
            println!("\nFound additive at path: {}", path);
            println!(
                "  Text: '{}'",
                node.utf8_text(source.as_bytes()).unwrap_or("<error>")
            );

            // Check fields
            if let Some(left) = node.child_by_field_name("left") {
                println!(
                    "  Left field: {} '{}'",
                    left.kind(),
                    left.utf8_text(source.as_bytes()).unwrap_or("<error>")
                );
            }
            if let Some(op) = node.child_by_field_name("operator") {
                println!(
                    "  Operator field: {} '{}'",
                    op.kind(),
                    op.utf8_text(source.as_bytes()).unwrap_or("<error>")
                );
            }
            if let Some(right) = node.child_by_field_name("right") {
                println!(
                    "  Right field: {} '{}'",
                    right.kind(),
                    right.utf8_text(source.as_bytes()).unwrap_or("<error>")
                );
            }
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let child_path = if let Some(field_name) = node.field_name_for_child(i as u32) {
                    format!("{}.{}", path, field_name)
                } else {
                    format!("{}[{}]", path, i)
                };
                find_additive(child, source, &child_path);
            }
        }
    }

    find_additive(root, input, "root");

    Ok(())
}
