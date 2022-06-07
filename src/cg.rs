use std::collections::HashMap;

use crate::edges::Edges;
use crate::Merge;
use crate::program::Program;
use crate::syntax_tree::{ASTIdentifier, ASTNode};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub first_statement_node: usize,
    pub last_statement_node: usize,
    pub node: usize,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub node: usize,
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Clone)]
pub struct Caller {
    pub name: String,
    pub node: usize,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub node: usize,
}

fn get_function_name(code: String) -> String {
    let fully_qualified_name = code.split("(")
        .collect::<Vec<&str>>()[0]
        .split(" ")
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .to_string();
    return fully_qualified_name.split(".").last().unwrap_or(&fully_qualified_name).to_string();
}

fn get_class_name(code: String) -> String {
    let class_name_with_inheritance_qualifier = code
        .split("class")
        .collect::<Vec<&str>>()[1]
        .split("{")
        .collect::<Vec<&str>>()[0]
        .trim()
        .to_string();
    return class_name_with_inheritance_qualifier.split(" ").next().unwrap().to_string();
}

fn get_import_name(import_code: String) -> String {
    return import_code
        .split(".")
        .collect::<Vec<&str>>()
        .last()
        .unwrap_or(&"")
        .replace(";", "")
        .replace("import ", "")
        .trim()
        .to_string();
}

fn get_enclosing_statement(parent: &ASTNode, id: usize) -> Option<usize> {
    let mut done = false;
    let mut i = id;
    while !done {
        let node = parent.get_node_by_id(i);
        if node.is_none() {
            done = true;
        } else {
            let child = node.unwrap();
            match &child.identifier {
                ASTIdentifier::ExpressionStatement |
                ASTIdentifier::LocalVariableDeclaration |
                ASTIdentifier::ReturnStatement | ASTIdentifier::AssertStatement
                | ASTIdentifier::YieldStatement | ASTIdentifier::IfStatement
                | ASTIdentifier::WhileStatement | ASTIdentifier::TryWithRessourceStatement
                | ASTIdentifier::TryStatement | ASTIdentifier::SynchronizedStatement
                | ASTIdentifier::ForStatement | ASTIdentifier::DoStatement
                | ASTIdentifier::SwitchStatement => {
                    if child.id < id && id <= child.children_until {
                        return Some(child.id.clone());
                    }
                }
                _default => (),
            }
        }
        i -= 1;
    }
    return None;
}

fn get_class_body(node: &ASTNode) -> Option<&ASTNode> {
    for child in node.children.iter() {
        match &child.identifier {
            ASTIdentifier::ClassBody => return Some(child),
            _default => (),
        }
    }
    return None;
}

fn get_functions(parent: &ASTNode) -> Vec<Function> {
    let mut functions = Vec::new();
    if let Some(class_body) = get_class_body(parent) {
        for child in class_body.children.iter() {
            match &child.identifier {
                ASTIdentifier::MethodDeclaration => {
                    let first_statement = get_function_statements(parent, child.id);
                    functions.push(Function {
                        name: get_function_name(child.code.parse().unwrap()),
                        node: child.id,
                        first_statement_node: *first_statement.first().unwrap(),
                        last_statement_node: *first_statement.last().unwrap(),
                    });
                }
                _default => (),
            }
        }
    }
    return functions;
}

fn create_func_table(parent: &ASTNode) -> HashMap<String, Class> {
    let mut classes: HashMap<String, Class> = HashMap::new();
    let mut id: usize = parent.id;
    let mut done = false;
    while !done {
        let node = parent.get_node_by_id(id);
        if node.is_none() {
            done = true;
        } else {
            match node.unwrap().identifier.clone() {
                ASTIdentifier::ClassDeclaration => {
                    let mut functions: HashMap<String, Function> = HashMap::new();
                    for func in get_functions(&node.unwrap()) {
                        functions.insert(func.name.clone(), func);
                    }
                    let class = Class {
                        name: get_class_name(node.unwrap().code.clone()),
                        node: node.unwrap().id,
                        functions,
                    };
                    id = node.unwrap().children_until - 1;
                    classes.insert(class.name.clone(), class);
                }
                _default => {
                    id += 1;
                }
            }
        }
    }
    return classes;
}


fn get_method_calls(parent: &ASTNode) -> Vec<Caller> {
    let mut calls: Vec<Caller> = Vec::new();
    let mut done = false;
    let mut id: usize = parent.id;

    while !done {
        let node = parent.get_node_by_id(id);
        if node.is_none() {
            done = true;
        } else {
            match &node.unwrap().identifier {
                ASTIdentifier::MethodInvocation => {
                    let statement_id = get_enclosing_statement(parent, node.unwrap().id);
                    if statement_id.is_none() {
                        id += 1;
                        continue;
                    }
                    let caller = Caller {
                        name: get_function_name(node.unwrap().code.clone()),
                        node: statement_id.unwrap(),
                    };
                    calls.push(caller);
                }

                _default => {}
            }
        }
        id += 1;
    }
    return calls;
}

fn get_imports(parent: &ASTNode) -> Vec<Import> {
    let mut imports: Vec<Import> = Vec::new();
    let mut done = false;
    let mut id: usize = parent.id;

    while !done {
        let node = parent.get_node_by_id(id);
        if node.is_none() {
            done = true;
        } else {
            match node.unwrap().identifier.clone() {
                ASTIdentifier::ImportDeclaration => {
                    let import = Import {
                        name: get_import_name(node.unwrap().code.clone()),
                        node: node.unwrap().id,
                    };
                    imports.push(import);
                }
                _default => {}
            }
        }
        id += 1;
    }
    return imports;
}


fn create_links(func_table: &HashMap<String, Class>, method_calls: &Vec<Caller>, imports: Option<&Vec<Import>>) -> Edges {
    let mut edges = Edges::new();
    for caller in method_calls {
        for (_, class) in func_table.iter() {
            if imports.is_some() {
                let mut found = false;
                for import in imports.unwrap().iter() {
                    if class.name == import.name {
                        found = true;
                    }
                }
                if !found {
                    continue;
                }
            }
            let function = class.functions.get(&caller.name);
            if function.is_some() {
                edges.entry(caller.node.clone()).or_insert(vec![]).push(function.unwrap().first_statement_node.clone());
                edges.entry(function.unwrap().first_statement_node.clone()).or_insert(vec![]).push(caller.node.clone());
            }
        }
    }
    if imports.is_none() {
        for (_, class) in func_table.iter() {
            for (_, func) in class.functions.iter() {
                edges.entry(func.node.clone()).or_insert(vec![]).push(func.first_statement_node.clone());
            }
        }
    }
    return edges;
}

fn get_function_statements(program: &ASTNode, function: usize) -> Vec<usize> {
    let fun_node = program.get_node_by_id(function).unwrap();
    let mut statements: Vec<usize> = vec![];
    let block = &fun_node.get_blocks();
    if block.first().is_some() {
        let block_node = program.get_node_by_id(block.first().unwrap().clone()).unwrap();
        let first_statement = block_node.get_statements();
        if first_statement.first().is_some() {
            statements.push(first_statement.first().unwrap().clone());
        }
    }
    return statements;
}

pub fn calculate_cg(programs: &Vec<&Program>) -> Edges {
    let mut edges = HashMap::new();
    let mut func_table: HashMap<String, Class> = HashMap::new();
    let mut method_calls: Vec<Caller> = Vec::new();
    for program in programs {
        let local_func_table = create_func_table(&program.tree);
        let local_method_calls = get_method_calls(&program.tree);
        edges.extend(create_links(&local_func_table, &local_method_calls, None));
        func_table.extend(local_func_table);
        method_calls.extend(local_method_calls);
    }
    for program in programs {
        let local_imports = get_imports(&program.tree);
        let local_method_calls = get_method_calls(&program.tree);
        edges.merge(&create_links(&func_table, &local_method_calls, Some(&local_imports)));
    }

    return edges;
}

#[cfg(test)]
mod tests {
    use crate::{calculate_cg, syntax_tree};
    use crate::edges::{show_edges, show_edges_multiple_programs};
    use crate::program::Program;

    const INNER_CLASS_CALL: &str = r#"public class Math {

    static int multiplyBytwo(int number) {
        return number * 2;
    }

    public static void main(String[] args) {
        int result = multiplyBytwo(2);
        System.out.println("The output is: " + result);
    }
}"#;
    const OUTER_CLASS_CALL_SINGLE_FILE: &str = r#"class Second {
  public static void main(String[] args) {
    Main myCar = new Main();     // Create a myCar object
    myCar.fullThrottle();      // Call the fullThrottle() method
    myCar.speed(200);          // Call the speed() method
  }
}
public class Main {
  public void fullThrottle() {
    System.out.println("The car is going as fast as it can!");
  }

  public void speed(int maxSpeed) {
    System.out.println("Max speed is: " + maxSpeed);
  }
}"#;

    const OUTER_CLASS_CALL_MULTIPLE_FILES_DEFINITIONS: &str = r#"class Animal {
  public void animalSound() {
    System.out.println("The animal makes a sound");
  }
}

class Pig extends Animal {
  public void animalSound() {
    System.out.println("The pig says: wee wee");
  }
}

class Dog extends Animal {
  public void animalSound() {
    System.out.println("The dog says: bow wow");
  }
}"#;
    const OUTER_CLASS_CALL_MULTIPLE_FILES: &str = r#"
import Dog;
class Main {
  public static void main(String[] args) {
    Animal myAnimal = new Animal();  // Create a Animal object
    Animal myDog = new Dog();  // Create a Dog object
    myDog.animalSound();
  }
}"#;


    #[test]
    fn test_local_func_call() {
        let mut program = Program::new(INNER_CLASS_CALL);
        program.get_tree();
        let edges = calculate_cg(&vec![&program]);
        assert_eq!(edges.len(), 6);
        assert_eq!(edges[&50], vec![24], "The method invocation should point to the first statement of the function definition");
        show_edges(&program.tree, &edges);
    }

    #[test]
    fn test_external_func_call_single_file() {
        let mut program = Program::new(OUTER_CLASS_CALL_SINGLE_FILE);
        program.get_tree();
        let edges = calculate_cg(&vec![&program]);
        assert_eq!(edges.len(), 7);
        assert_eq!(edges[&38], vec![78], "myCar.fullThrottle() should point to the fullThrottle() method");
        assert_eq!(edges[&48], vec![106], "myCar.speed(200) should point to the speed(int maxSpeed) method");
        show_edges(&program.tree, &edges);
    }

    #[test]
    fn test_external_func_call_multiple_file() {
        let programs = Program::new_list(vec![OUTER_CLASS_CALL_MULTIPLE_FILES_DEFINITIONS, OUTER_CLASS_CALL_MULTIPLE_FILES]);
        let program_refs: Vec<&Program> = programs.iter().collect();
        let edges = calculate_cg(&program_refs);
        assert_eq!(edges.len(), 4);
        assert_eq!(edges[&152], vec![83], "The method invocation myDog.anomalSound should point to the animalSound() method of the Dog Class");
    }
}
