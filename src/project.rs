use crate::program::Program;
use crate::syntax_tree::{ASTIdentifier, ASTNode};

pub type Project<'a> = &'a Vec<&'a Program>;

pub trait ProjectExt {
    fn find_node(&self, predicate: &dyn Fn(&ASTNode) -> bool) -> Vec<&ASTNode>;

    fn find_statement(&self, predicate: &dyn Fn(&ASTNode) -> bool) -> Vec<&ASTNode>;
}

impl ProjectExt for Project<'_> {
    fn find_node(&self, predicate: &dyn Fn(&ASTNode) -> bool) -> Vec<&ASTNode> {
        let mut result: Vec<&ASTNode> = vec![];
        for program in *self {
            let mut done = false;
            let parent = &program.tree;
            let mut i = parent.id;
            while !done {
                let node = parent.get_node_by_id(i);
                if node.is_none() {
                    done = true;
                } else {
                    let child = node.unwrap();
                    if predicate(child) {
                        result.push(child);
                    }
                }
                i += 1;
            }
        }
        return result;
    }

    fn find_statement(&self, predicate: &dyn Fn(&ASTNode) -> bool) -> Vec<&ASTNode> {
        let new_predicate = |node: &ASTNode| {
            match node.identifier.clone() {
                ASTIdentifier::ExpressionStatement |
                ASTIdentifier::LocalVariableDeclaration |
                ASTIdentifier::ReturnStatement | ASTIdentifier::AssertStatement
                | ASTIdentifier::YieldStatement | ASTIdentifier::IfStatement
                | ASTIdentifier::WhileStatement | ASTIdentifier::TryWithRessourceStatement
                | ASTIdentifier::TryStatement | ASTIdentifier::SynchronizedStatement
                | ASTIdentifier::ForStatement | ASTIdentifier::DoStatement
                | ASTIdentifier::SwitchStatement => {
                    predicate(node)
                }
                _default => false
            }
        };
        return self.find_node(&new_predicate);
    }
}