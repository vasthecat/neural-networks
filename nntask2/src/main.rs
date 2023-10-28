use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Clone)]
struct Arc {
    from: String,
    to: String,
    order: i32,
}

#[derive(Debug)]
struct Graph {
    vertices: Vec<String>,
    arcs: Vec<Arc>,
}

#[derive(PartialEq, Eq)]
enum ParserState {
    Waiting,
    Graph,
    Vertex,
    Arc,
    From,
    To,
    Order,
}

fn parse_input(input_file: &File) -> Option<Graph> {
    let mut vertices = Vec::new();
    let mut arcs = Vec::new();
    let mut arc = Arc {
        from: String::new(),
        to: String::new(),
        order: 0,
    };
    let mut state = ParserState::Waiting;
    let parser = EventReader::new(input_file);

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                state = match (name.local_name.as_str(), state) {
                    ("graph", ParserState::Waiting) => ParserState::Graph,
                    ("vertex", ParserState::Graph) => ParserState::Vertex,
                    ("arc", ParserState::Graph) => ParserState::Arc,
                    ("from", ParserState::Arc) => ParserState::From,
                    ("to", ParserState::Arc) => ParserState::To,
                    ("order", ParserState::Arc) => ParserState::Order,
                    _ => return None,
                };
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                state = match (name.local_name.as_str(), state) {
                    ("graph", ParserState::Graph) => ParserState::Waiting,
                    ("vertex", ParserState::Vertex) => ParserState::Graph,
                    ("arc", ParserState::Arc) => {
                        arcs.push(arc.clone());
                        ParserState::Graph
                    }
                    ("from", ParserState::From) => ParserState::Arc,
                    ("to", ParserState::To) => ParserState::Arc,
                    ("order", ParserState::Order) => ParserState::Arc,
                    _ => return None,
                };
            }
            Ok(XmlEvent::Characters(text)) => match state {
                ParserState::Vertex => vertices.push(text),
                ParserState::From => arc.from = text,
                ParserState::To => arc.to = text,
                ParserState::Order => arc.order = text.parse::<i32>().unwrap(),
                _ => return None,
            },
            Err(err) => println!("{err}"),
            _ => {}
        }
    }

    Some(Graph { vertices, arcs })
}

#[derive(Debug)]
struct Node {
    name: String,
    children: Vec<String>,
    parents: Vec<String>,
}

impl Node {
    fn new(name: String) -> Node {
        Node {
            name,
            children: Vec::new(),
            parents: Vec::new(),
        }
    }
}

fn call_string(node: &Node, nodes: &HashMap<String, Node>) -> String {
    let children = node
        .children
        .iter()
        .map(|c| call_string(&nodes[c], nodes))
        .collect::<Vec<String>>();
    format!("{}({})", node.name, children.join(", "))
}

fn find_root(nodes: &HashMap<String, Node>) -> Option<String> {
    let mut root_name = String::new();
    let mut found = false;
    for (name, node) in nodes.iter() {
        if node.parents.len() == 0 {
            if found {
                return None;
            }
            root_name = name.clone();
            found = true;
        }
    }
    Some(root_name)
}

fn has_cycle(
    root: &String,
    name: &String,
    visited: &mut HashSet<String>,
    nodes: &HashMap<String, Node>,
) -> bool {
    if visited.contains(name) {
        return false;
    }
    visited.insert(name.clone());
    for node in &nodes[name].children {
        if node == root {
            return true;
        }
        if has_cycle(root, node, visited, nodes) {
            return true;
        }
    }
    false
}

fn get_call_string(g: &Graph) -> Option<String> {
    let mut nodes = HashMap::new();
    for vert in &g.vertices {
        nodes.insert(vert.clone(), Node::new(vert.clone()));
    }
    for arc in &g.arcs {
        nodes
            .get_mut(&arc.from)
            .unwrap()
            .children
            .push(arc.to.clone());
        nodes
            .get_mut(&arc.to)
            .unwrap()
            .parents
            .push(arc.from.clone());
    }

    let mut visited = HashSet::new();
    for vert in &g.vertices {
        if has_cycle(vert, vert, &mut visited, &nodes) {
            return None;
        }
        visited.clear();
    }

    let root_name = find_root(&nodes)?;
    let root_node = &nodes[&root_name];

    Some(call_string(root_node, &nodes))
}

#[derive(Parser)]
struct Config {
    #[arg(long, value_name = "FILE")]
    input1: String,
    #[arg(long, value_name = "FILE")]
    output1: String,
}

fn main() {
    let config = Config::parse();
    let input = match File::open(config.input1) {
        Ok(s) => s,
        Err(_) => {
            println!("Не удалось прочитать граф из указанного файла");
            return;
        }
    };
    let g = match parse_input(&input) {
        Some(g) => g,
        None => {
            println!("Некорректный ввод");
            return;
        }
    };
    let mut output = match File::create(config.output1) {
        Ok(out) => out,
        Err(_) => {
            println!("Не удалось создать файл для вывода");
            return;
        }
    };
    let call_string = match get_call_string(&g) {
        Some(s) => s,
        None => {
            println!("Некорректный ввод - в графе есть циклы");
            return;
        }
    };
    output
        .write(call_string.as_bytes())
        .expect("Не удалось записать вывод в файл");
}
