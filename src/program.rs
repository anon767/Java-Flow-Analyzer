use std::str::{from_utf8, FromStr};
use tree_sitter::{Node, Parser};
use crate::syntax_tree::{ASTIdentifier, ASTNode};

#[derive(Debug)]
pub struct Program {
    code: String,
    pub tree: ASTNode,
    pub count: usize,
}

impl Program {
    fn get_id(&mut self) -> usize {
        self.count += 1;
        self.count
    }

    pub fn get_node_by_id_multiple_programs<'a>(programs: &'a Vec<&Program>, id: usize) -> Option<&'a ASTNode> {
        for program in programs {
            let node = &program.tree.get_node_by_id( id);
            if node.is_some() {
                return node.clone();
            }
        }
        return None;
    }

    pub fn traverse(&mut self, tree: Node) -> ASTNode {
        let prog_code = self.code.clone();
        let code = from_utf8(&prog_code.as_bytes()[tree.start_byte() as usize..tree.end_byte() as usize]).unwrap();
        let mut current = ASTNode {
            id: self.get_id(),
            identifier: ASTIdentifier::from_str(tree.kind())
                .unwrap_or(ASTIdentifier::UNKNOWN),
            code: code.to_string(),
            children: vec![],
            children_until: 0,
            cache: Default::default()
        };
        for child in 0..tree.child_count() {
            let child_node = self.traverse(tree.child(child).unwrap());
            current.children.push(child_node);
        }

        current.children_until = self.count;
        return current;
    }

    fn create_graph(&mut self, tree: tree_sitter::Tree) {
        self.tree = ASTNode {
            id: self.get_id(),
            identifier: ASTIdentifier::Root,
            code: self.code.clone(),
            children: vec![],
            children_until: self.count,
            cache: Default::default()
        };


        for child in 0..tree.root_node().child_count() {
            let child_node = self.traverse(tree.root_node().child(child).unwrap());
            self.tree.children.push(child_node);
        }
        self.tree.children_until = self.count;
    }
    pub fn new(code: &str) -> Program {
        return Program {
            code: code.to_string(),
            tree: ASTNode {
                id: 0,
                children_until: 1,
                identifier: ASTIdentifier::Root,
                code: code.to_string(),
                children: vec![],
                cache: Default::default()
            },
            count: 0,
        };
    }
    pub fn new_list(codes: Vec<&str>) -> Vec<Program> {
        let mut programs = vec![];
        let mut p1 = Program::new(codes.first().unwrap());
        p1.get_tree();
        programs.push(p1);
        for code in 1..codes.len() {
            let mut p = Program::subsequent(codes[code], (&programs)[(&programs).len() - 1].count);
            p.get_tree();
            programs.push(p);
        }
        return programs;
    }
    fn subsequent(code: &str, count: usize) -> Program {
        return Program {
            code: code.to_string(),
            tree: ASTNode {
                id: count,
                children_until: count + 1,
                identifier: ASTIdentifier::Root,
                code: code.to_string(),
                children: vec![],
                cache: Default::default()
            },
            count,
        };
    }

    pub fn get_tree(&mut self) {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_java::language()).expect("Error loading Java grammar");


        let parsed = parser.parse(&self.code, None);
        if parsed.is_some() {
            self.create_graph(parsed.unwrap());
        }
    }
}

