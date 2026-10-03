mod error;
mod lexer;
mod parser;

mod hir;

use lexer::*;
use parser::*;

fn main() {
    let mut arena = ast::NodeArena::new();

    let src = "print(1 + 2)\nif x:\n    print(3)\nelse:\n    print(4)\n";
    let lexer = Lexer::new(src.to_string());
    let parser = Parser::new(lexer, &mut arena);

    let tree = parser.program();
    match tree {
        Ok(node) => {
            println!("{}", src);
            println!("Parsed successfully!\n");
            for root in &node.roots {
                if let Some(ast_node) = node.get_node(*root) {
                    println!("Root: {:}", ast_node);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing: {}", e);
        }
    }
}