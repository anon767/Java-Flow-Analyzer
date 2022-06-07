use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Read;
use std::str::FromStr;

use docopt::Docopt;
use regex::Regex;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::cfg::calculate_cfg_per_programs;
use crate::cg::calculate_cg;
use crate::edges::{Merge, show_edges_multiple_programs};
use crate::flow_solver::Reachable;
use crate::config::Config;
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
    let file_contents: Vec<String> = files.iter().map(|f| {
        let mut file = std::fs::File::open(f).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    }).collect();

    let programs = Program::new_list(file_contents.iter().map(|f| f.as_str()).collect());
    let program_refs: Vec<&Program> = programs.iter().collect();
    let mut edges = calculate_cg(&program_refs);
    let cfg = calculate_cfg_per_programs(&program_refs);
    edges.merge(&cfg);

    let reachable = flow_solver::solve(&edges);

    let mut analysis_nodes: HashMap<String, Vec<usize>> = HashMap::new();

    for cnode in config.nodes {
        let predicate = |node: &ASTNode| {
            let mut res = true;
            if cnode.identifier.is_some() {
                res &= node.identifier == ASTIdentifier::from_str(cnode.identifier.clone().unwrap().as_str()).unwrap();
            }
            if cnode.code.is_some() {
                let re = Regex::new(cnode.code.clone().unwrap().as_str()).unwrap();
                res &= re.is_match(node.code.clone().as_str())
            }
            return res;
        };
        let found: Vec<usize> = Project::find_node(&&*&program_refs, &predicate).iter().map(|n| n.id).collect();
        analysis_nodes.insert(cnode.name, found);
    }

    for cflow in config.flows {
        for node in analysis_nodes.get(cflow.from.as_str()).unwrap() {
            for target in analysis_nodes.get(cflow.to.as_str()).unwrap() {
                if reachable.is_reachable(*node, *target) {
                    println!("{:#?} reaches {:#?}", cflow.from, cflow.to);
                    println!("Source {:#?}", Program::get_node_by_id_multiple_programs(&program_refs, *node).unwrap().code);
                    println!("Target {:#?}", Program::get_node_by_id_multiple_programs(&program_refs, *target).unwrap().code);
                    println!("____________________________________")
                }
            }
        }
    }
}
