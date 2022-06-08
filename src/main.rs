use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use docopt::Docopt;
use regex::Regex;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::cfg::calculate_cfg_per_programs;
use crate::cg::calculate_cg;
use crate::config::Config;
use crate::edges::{Merge, show_edges_multiple_programs};
use crate::flow_solver::Reachable;
use crate::program::Program;
use crate::project::{Project, ProjectExt};
use crate::syntax_tree::{ASTIdentifier, ASTNode};

mod syntax_tree;
mod cfg;
mod edges;
mod cg;
mod flow_solver;
mod project;
mod program;
mod config;


const USAGE: &'static str = "
Analyze Java Project

Usage:
  rustparse --path <path>

Options:
  --path=<path>     Sets the path to the project configuration file.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_path: String,
}

fn find_files(path: String) -> Vec<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.path().to_string_lossy();
        if f_name.ends_with(".java") {
            files.push(f_name.to_string());
        }
    }
    files
}


fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let config = Config::parse(&args.flag_path);

    let config_file_path = &args.flag_path.split("/").collect::<Vec<&str>>()[..args.flag_path.split("/").count() - 1].join("/");
    let files = find_files(format!("{}/{}", config_file_path, config.project));
    println!("creating");
    let programs = Program::new_list_from_files(files);
    println!("creating done");
    let program_refs: Vec<&Program> = programs.iter().collect();
    println!("CG");
    let mut edges = calculate_cg(&program_refs);
    println!("CG Done");
    println!("CFG");
    let cfg = calculate_cfg_per_programs(&program_refs);
    println!("CFG Done");
    edges.merge(&cfg);
    let reachable = flow_solver::solve(&edges);
    let mut analysis_nodes: HashMap<String, Vec<usize>> = HashMap::new();

    for cnode in config.nodes {
        let predicate = |node: &ASTNode| {
            if cnode.identifier.is_some() {
                if node.identifier != ASTIdentifier::from_str(cnode.identifier.clone().unwrap().as_str()).unwrap() {
                    return false;
                }
            }
            if cnode.code.is_some() {
                let re = Regex::new(cnode.code.clone().unwrap().as_str()).unwrap();
                if !re.is_match(node.code.clone().as_str()) {
                    return false;
                }
            }
            return true;
        };
        let found: Vec<usize> = Project::find_node(&&*&program_refs, &predicate).iter().map(|n| n.id).collect();
        analysis_nodes.insert(cnode.name, found);
    }

    for cflow in config.flows {
        for node in analysis_nodes.get(cflow.from.as_str()).unwrap() {
            for target in analysis_nodes.get(cflow.to.as_str()).unwrap_or(&vec![]) {
                if reachable.is_reachable(*node, *target) {
                    let s = &Program::get_node_by_id_multiple_programs(&program_refs, *node).unwrap();
                    let t = &Program::get_node_by_id_multiple_programs(&program_refs, *target).unwrap();
                    println!("{:#?} reaches {:#?}", cflow.from, cflow.to);
                    println!("Source {} {}:{}", s.1, s.0.line_start, s.0.line_end);
                    println!("Target {} {}:{}", t.1, t.0.line_start, t.0.line_end);
                    println!("____________________________________")
                }
            }
        }
    }
}
