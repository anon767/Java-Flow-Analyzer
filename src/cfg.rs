use std::collections::HashMap;

use crate::edges::Edges;
use crate::Merge;
use crate::program::Program;
use crate::syntax_tree::{ASTIdentifier, ASTNode};

fn create_links(parent: &ASTNode, until: usize, before_statement: Vec<usize>, start_id: usize) -> Edges {
    let mut edges: Edges = Edges::new();
    let mut done = false;
    let mut id: usize = start_id;
    let mut before_statement = before_statement;
    while !done {
        let node = parent.get_node_by_id(id);
        if node.is_none() || id > until {
            done = true;
        } else {
            match node.unwrap().identifier.clone() {
                ASTIdentifier::Block | ASTIdentifier::SwitchBlock => {
                    let tmp_edges = create_links(&node.unwrap(), node.unwrap().children_until, vec![], id + 1);
                    let mut block_nodes: Vec<usize> = vec![0];
                    for (key, value) in tmp_edges {
                        edges.insert(key, value.clone());
                        block_nodes.push(key);
                    }
                    id = node.unwrap().children_until - 1;
                    continue;
                }
                ASTIdentifier::IfStatement | ASTIdentifier::WhileStatement
                | ASTIdentifier::AssertStatement | ASTIdentifier::ExpressionStatement
                | ASTIdentifier::LocalVariableDeclaration | ASTIdentifier::TryWithRessourceStatement
                | ASTIdentifier::TryStatement | ASTIdentifier::SynchronizedStatement
                | ASTIdentifier::ForStatement | ASTIdentifier::DoStatement
                | ASTIdentifier::YieldStatement | ASTIdentifier::SwitchStatement => {
                    add_link(&mut edges, id, &before_statement);
                    before_statement = vec![id];
                }
                ASTIdentifier::BreakStatement => {
                    add_link(&mut edges, id, &before_statement);
                    before_statement = vec![];
                }
                ASTIdentifier::ReturnStatement => {
                    add_link(&mut edges, id, &before_statement);
                }
                _default => (),
            }
        }
        id += 1;
    }
    return edges;
}


fn create_branches(parent: &ASTNode, mut edges: Edges, start_id: usize) -> Edges {
    let mut done = false;
    let mut id: usize = start_id;
    while !done {
        let node = parent.get_node_by_id(id);
        if node.is_none() {
            done = true;
        } else {
            match node.unwrap().identifier.clone() {
                ASTIdentifier::IfStatement | ASTIdentifier::ForStatement
                | ASTIdentifier::WhileStatement | ASTIdentifier::DoStatement => {
                    let blocks = &node.unwrap().get_blocks();
                    for block in blocks {
                        let statements = parent.get_node_by_id(block.clone()).unwrap().get_statements();
                        add_link(&mut edges, statements[0], &vec![id]);
                        let tmp = edges[&id][0].clone();
                        add_link(&mut edges, tmp, &vec![statements[statements.len() - 1]]);
                    }
                }
                ASTIdentifier::SwitchStatement => {
                    let blocks = &node.unwrap().get_blocks();
                    let block = node.unwrap().get_node_by_id(blocks[0]);
                    let statements = get_first_statements_after_switch_label(&block.unwrap());
                    for statement in statements.clone() {
                        add_link(&mut edges, statement.clone(), &vec![id]);
                    }
                    //let tmp = edges[&id][0].clone();
                    //add_link(&mut edges, tmp, statements.borrow());
                }
                ASTIdentifier::TryStatement | ASTIdentifier::TryWithRessourceStatement => {
                    let catch_blocks = get_catch_blocks(&node.unwrap());
                    for catch_block in catch_blocks {
                        let mut blocks = node.unwrap().get_node_by_id(catch_block).unwrap().get_blocks();
                        let new_blocks = &node.unwrap().get_blocks();
                        blocks.append(&mut new_blocks.clone());

                        for block in blocks {
                            let statements = &node.unwrap().get_node_by_id(block).unwrap().get_statements();
                            if statements.len() > 0 {
                                add_link(&mut edges, statements[0], &vec![id]);
                                let tmp = edges[&id][0].clone();
                                add_link(&mut edges, tmp, &vec![statements[statements.len() - 1]]);
                            }
                        }
                    }
                }

                _default => (),
            }
        }
        id += 1;
    }
    return edges;
}

fn add_link(edges: &mut Edges, id: usize, before_statement: &Vec<usize>) {
    for v in before_statement {
        if v > &0 {
            edges.entry(v.clone()).or_insert(vec![]).push(id);
        }
    }
}

fn get_catch_blocks(node: &ASTNode) -> Vec<usize> {
    let mut blocks = vec![];
    for child in &node.children {
        match child.identifier.clone() {
            ASTIdentifier::CatchClause => {
                blocks.push(child.id);
            }
            _default => { continue; }
        }
    }
    return blocks;
}


fn get_first_statements_after_switch_label(node: &ASTNode) -> Vec<usize> {
    let mut blocks = vec![];
    let statements: Vec<usize> = node.get_statements();
    for child in &node.children {
        match child.identifier.clone() {
            ASTIdentifier::SwitchLabel => {
                for statement in statements.clone() {
                    if statement > child.id {
                        blocks.push(statement);
                        break;
                    }
                }
            }
            _default => {}
        }
    }
    return blocks;
}

pub fn get_functions(parent: &ASTNode) -> Vec<&ASTNode> {
    let mut functions = vec![];
    let mut done = false;
    let mut i = parent.id;
    while !done {
        let node = parent.get_node_by_id(i);
        if node.is_none() {
            done = true;
        } else {
            let child = node.unwrap();
            match child.identifier.clone() {
                ASTIdentifier::MethodDeclaration => {
                    functions.push(child);
                }
                _default => (),
            }
        }
        i += 1;
    }
    return functions;
}

pub fn calculate_cfg(program: &ASTNode) -> Edges {
    let links = create_links(&program, program.children_until, vec![0], program.id);
    let branched_links = create_branches(&program, links, program.id);
    return branched_links;
}

pub fn calculate_cfg_per_programs(programs: &Vec<&Program>) -> Edges {
    let mut cfgs: Edges = HashMap::new();
    for program in programs {
        for function in get_functions(&program.tree) {
            cfgs.merge(&calculate_cfg(function));
        }
    }
    return cfgs;
}

#[cfg(test)]
mod tests {
    use crate::syntax_tree;
    use crate::program::Program;

    use super::*;

    const IF_CODE: &str = r#"
    class Test {
        int double(int x) {
            if (x > 5) {
                blubb = 1;
                bla = 5+blubb;
            } else {
                int bla = 5;
                bla = 5+3;
            }
            System.out.println("Hello, world!" + bla);
        }
    }
"#;
    const IF_IF_CODE: &str = r#"
    class Test {
        int double(int x) {
            if (x > 5) {
                if (x < 10) {
                    blubb = 1;
                    bla = 5+blubb;
                }
            } else {
                int bla = 5;
                bla = 5+3;
            }
            System.out.println("Hello, world!" + bla);
        }
    }
"#;
    const SWITCH_CODE: &str = r#"
   public class SwitchCaseClass {

    public static void main(String[] args){
        int i=2;

        switch(i){
        case 0:
            System.out.println("i ist null");
            break;
        case 1:
            System.out.println("i ist eins");
            break;
        case 2:
            System.out.println("i ist zwei");
            break;
        case 3:
            System.out.println("i ist drei");
            break;
        default:
            System.out.println("i liegt nicht zwischen null und drei");
            break;
        }
        System.out.println("Ende");

    }
}
"#;
    const TRY_CATCH_CODE: &str = r#"
	public static void main(String[] args) {
			String input = JOptionPane.showInputDialog(null, "Wie alt bist du?");
			try{
				int alter = Integer.parseInt(input);
				JOptionPane.showMessageDialog(null, "Du bist " + alter + " Jahre alt.");
			}catch(Exception e){
				JOptionPane.showMessageDialog(null, "Du Schlingel hast keine Zahl eingegeben.");
			}
            System.out.println("Ende");
	}

"#;
    const FOR_STATEMENT: &str = r#"
	public static void main(String[] args) {
			for (int i = 0; i < 5; i++) {
              System.out.println(i);
            }
           Systen.out.println("Ende");
	}
"#;
    const WHILE_STATEMENT: &str = r#"
	public static void main(String[] args) {
			while (i < 5) {
              System.out.println(i);
              i++;
            }
           Systen.out.println("Ende");
	}
"#;

    #[test]
    fn test_for_statement() {
        let mut program = Program::new(FOR_STATEMENT);
        program.get_tree();
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 3);
        assert_eq!(edges[&26], vec![59], "int i = 0; --> Sysout Ende");
        assert_eq!(edges[&23], vec![26, 45], "for --> int i = 0; AND Sysout i");
        assert_eq!(edges[&45], vec![26], "Sysout i --> int i = 0;");
    }

    #[test]
    fn test_try_catch_statement() {
        let mut program = Program::new(TRY_CATCH_CODE);
        program.get_tree();
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 5);
        assert_eq!(edges[&87], vec![100], "showMessageDialog --> Sysout Ende");
        assert_eq!(edges[&23], vec![39], "showInputDialog --> TryCatch");
        assert_eq!(edges[&43], vec![58], "parseInt --> showMessageDialog");
        assert_eq!(edges[&58], vec![100], "showMessageDialog --> Sysout Ende");
        assert_eq!(edges[&39], vec![100, 87, 43], "TryCatch --> Sysout Ende AND ShowMessageDialog AND parseInt");
    }

    #[test]
    fn test_switch_statement() {
        let mut program = Program::new(SWITCH_CODE);
        program.get_tree();
        //println!("{:#?}", program);
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 7);
        assert_eq!(edges[&35], vec![143, 47, 67, 87, 107, 126], "Switch --> all Sysouts");
    }

    #[test]
    fn test_if_if_statement() {
        let mut program = Program::new(IF_IF_CODE);
        program.get_tree();
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 6);
        assert_eq!(edges[&20], vec![80, 31, 62], "Outer IF --> inner IF AND Sysout Ende AND Else");
    }

    #[test]
    fn test_if_statement() {
        let mut program = Program::new(IF_CODE);
        program.get_tree();
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 5);
        assert_eq!(edges[&20], vec![68, 31, 50], "IF --> Sysout AND THEN Branch AND Else Branch");
    }

    #[test]
    fn test_while_statement() {
        let mut program = Program::new(WHILE_STATEMENT);
        program.get_tree();
        let edges = calculate_cfg(&program.tree);
        assert_eq!(edges.len(), 3);
        assert_eq!(edges[&23], vec![53, 34], "WHILE --> Sysout AND Sysout Ende");
    }
}

