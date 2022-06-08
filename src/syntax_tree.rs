use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum ASTIdentifier {
    Identifier,
    MethodInvocation,
    ExpressionStatement,
    DecimalIntegerLiteral,
    BinaryExpression,
    StringLiteral,
    BooleanLiteral,
    ReturnStatement,
    IfStatement,
    AssignmentExpression,
    VariableDeclarator,
    IntegralType,
    LocalVariableDeclaration,
    Block,
    ParenthesizedExpression,
    FormalParameter,
    FormalParameters,
    MethodDeclaration,
    ClassBody,
    Class,
    Root,
    WhileStatement,
    ForStatement,
    AssertStatement,
    DoStatement,
    BreakStatement,
    ContinueStatement,
    YieldStatement,
    SynchronizedStatement,
    ThrowStatement,
    TryStatement,
    TryWithRessourceStatement,
    SwitchStatement,
    SwitchLabel,
    Case,
    Catch,
    SwitchBlock,
    CatchClause,
    ClassDeclaration,
    PackageDeclaration,
    ImportDeclaration,
    UNKNOWN,
}

impl FromStr for ASTIdentifier {
    type Err = ();
    fn from_str(input: &str) -> Result<ASTIdentifier, Self::Err> {
        match input {
            "identifier" => Ok(ASTIdentifier::Identifier),
            "method_invocation" => Ok(ASTIdentifier::MethodInvocation),
            "expression_statement" => Ok(ASTIdentifier::ExpressionStatement),
            "decimal_integer_literal" => Ok(ASTIdentifier::DecimalIntegerLiteral),
            "binary_expression" => Ok(ASTIdentifier::BinaryExpression),
            "string_literal" => Ok(ASTIdentifier::StringLiteral),
            "boolean_literal" => Ok(ASTIdentifier::BooleanLiteral),
            "return_statement" => Ok(ASTIdentifier::ReturnStatement),
            "if_statement" => Ok(ASTIdentifier::IfStatement),
            "assignment_expression" => Ok(ASTIdentifier::AssignmentExpression),
            "variable_declarator" => Ok(ASTIdentifier::VariableDeclarator),
            "integral_type" => Ok(ASTIdentifier::IntegralType),
            "local_variable_declaration" => Ok(ASTIdentifier::LocalVariableDeclaration),
            "block" => Ok(ASTIdentifier::Block),
            "parenthesized_expression" => Ok(ASTIdentifier::ParenthesizedExpression),
            "formal_parameter" => Ok(ASTIdentifier::FormalParameter),
            "formal_parameters" => Ok(ASTIdentifier::FormalParameters),
            "method_declaration" => Ok(ASTIdentifier::MethodDeclaration),
            "class_body" => Ok(ASTIdentifier::ClassBody),
            "class" => Ok(ASTIdentifier::Class),
            "while_statement" => Ok(ASTIdentifier::WhileStatement),
            "for_statement" => Ok(ASTIdentifier::ForStatement),
            "enhanced_for_statement" => Ok(ASTIdentifier::ForStatement),
            "assert_statement" => Ok(ASTIdentifier::AssertStatement),
            "do_statement" => Ok(ASTIdentifier::DoStatement),
            "break_statement" => Ok(ASTIdentifier::BreakStatement),
            "continue_statement" => Ok(ASTIdentifier::ContinueStatement),
            "yield_statement" => Ok(ASTIdentifier::YieldStatement),
            "synchronized_statement" => Ok(ASTIdentifier::SynchronizedStatement),
            "throw_statement" => Ok(ASTIdentifier::ThrowStatement),
            "try_statement" => Ok(ASTIdentifier::TryStatement),
            "try_with_ressource_statement" => Ok(ASTIdentifier::TryWithRessourceStatement),
            "switch_statement" => Ok(ASTIdentifier::SwitchStatement),
            "case" => Ok(ASTIdentifier::Case),
            "switch_label" => Ok(ASTIdentifier::SwitchLabel),
            "switch_block" => Ok(ASTIdentifier::SwitchBlock),
            "catch" => Ok(ASTIdentifier::Catch),
            "catch_clause" => Ok(ASTIdentifier::CatchClause),
            "class_declaration" => Ok(ASTIdentifier::ClassDeclaration),
            "package_declaration" => Ok(ASTIdentifier::PackageDeclaration),
            "import_declaration" => Ok(ASTIdentifier::ImportDeclaration),
            "root" => Ok(ASTIdentifier::Root),
            _ => {
                Ok(ASTIdentifier::UNKNOWN)
            }
        }
    }
}

#[derive(Clone)]
pub struct ASTNode {
    pub id: usize,
    pub children_until: usize,
    pub identifier: ASTIdentifier,
    pub code: String,
    pub children: Vec<ASTNode>,
    pub cache: HashMap<usize, ASTNode>,
    pub line_start: usize,
    pub line_end: usize,
}

impl Default for ASTNode {
    fn default() -> Self {
        ASTNode {
            id: 0,
            children_until: 0,
            identifier: ASTIdentifier::UNKNOWN,
            code: String::new(),
            children: Vec::new(),
            cache: Default::default(),
            line_start: 0,
            line_end: 0,
        }
    }
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:#?} ", self.identifier)
    }
}

impl fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ASTNode")
            .field("id", &self.id)
            .field("children_until", &self.children_until)
            .field("identifier", &self.identifier)
            .field("code", &self.code)
            .field("cache_size", &self.cache.keys().count())
            .field("children", &self.children)
            .finish()
    }
}

impl Hash for ASTNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq<Self> for ASTNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl ASTNode {
    pub fn get_node_by_id(self: &Self, id: usize, dfs: bool) -> Option<&ASTNode> {
        if self.cache.contains_key(&id) {
            return self.cache.get(&id);
        }
        if dfs {
            return self.get_node_by_id_dfs(id);
        } else {
            return self.get_node_by_id_bfs(id);
        }
    }

    fn get_node_by_id_bfs(self: &Self, id: usize) -> Option<&ASTNode> {
        let mut deque: VecDeque<&ASTNode> = VecDeque::new();
        deque.push_back(self);
        while !deque.is_empty() {
            let node = deque.pop_front().unwrap();

            if node.id == id {
                return Some(node);
            }
            if node.id > id {
                continue;
            }

            for child in &node.children {
                deque.push_back(child);
            }
        }
        return None;
    }


    fn get_node_by_id_dfs(self: &Self, id: usize) -> Option<&ASTNode> {
        let mut stack: Vec<&ASTNode> = vec![];
        stack.push(self);
        while !stack.is_empty() {
            let node = stack.pop().unwrap();

            if node.id == id {
                return Some(node);
            }
            if node.id > id {
                continue;
            }

            for child in &node.children {
                stack.push(child);
            }
        }
        return None;
    }

    pub fn build_cache(&mut self) {
        for i in self.id..self.children_until {
            let node = self.get_node_by_id(i, true).unwrap();
            self.cache.insert(i, node.clone());
        }
    }
    pub fn get_statements(self: &Self) -> Vec<usize> {
        let mut nodes: Vec<usize> = vec![];
        for child in &self.children {
            match &child.identifier {
                ASTIdentifier::ExpressionStatement |
                ASTIdentifier::LocalVariableDeclaration |
                ASTIdentifier::ReturnStatement | ASTIdentifier::AssertStatement
                | ASTIdentifier::YieldStatement | ASTIdentifier::IfStatement
                | ASTIdentifier::WhileStatement | ASTIdentifier::TryWithRessourceStatement
                | ASTIdentifier::TryStatement | ASTIdentifier::SynchronizedStatement
                | ASTIdentifier::ForStatement | ASTIdentifier::DoStatement
                | ASTIdentifier::SwitchStatement => {
                    nodes.push(child.id);
                }
                _default => { continue; }
            }
        }
        return nodes;
    }

    pub fn get_blocks(self: &Self) -> Vec<usize> {
        let mut blocks = vec![];
        for child in &self.children {
            match &child.identifier {
                ASTIdentifier::Block
                | ASTIdentifier::SwitchBlock => {
                    blocks.push(child.id);
                }
                _default => { continue; }
            }
        }
        return blocks;
    }
}


