use clap::Parser;
use serde::{de::IntoDeserializer, Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
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

fn parse_input_graph(input_file: &File) -> Option<Graph> {
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

#[derive(Debug, Clone, Deserialize)]
#[serde(remote = "Operation")]
enum Operation {
    #[serde(rename = "exp")]
    Exp,
    #[serde(rename = "+")]
    Plus,
    #[serde(rename = "*")]
    Mult,
    Const(f64),
}

impl<'de> Deserialize<'de> for Operation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Possible {
            A(String),
            B(f64),
        }
        match Possible::deserialize(deserializer).unwrap() {
            Possible::A(val) => Operation::deserialize(val.into_deserializer()),
            Possible::B(val) => Ok(Operation::Const(val)),
        }
    }
}

#[derive(Debug)]
struct Node {
    operation: Operation,
    children: Vec<String>,
    parents: Vec<String>,
}

impl Node {
    fn new(operation: Operation) -> Node {
        Node {
            operation,
            children: Vec::new(),
            parents: Vec::new(),
        }
    }
}

fn evaluate_node(node: &Node, nodes: &HashMap<String, Node>) -> Option<f64> {
    let parents = node
        .parents
        .iter()
        .map(|c| evaluate_node(&nodes[c], nodes))
        .collect::<Vec<Option<f64>>>();
    match node.operation {
        Operation::Const(num) => Some(num),
        Operation::Exp => {
            if parents.len() != 1 {
                None
            } else {
                Some(parents[0]?.exp())
            }
        }
        Operation::Plus => {
            if parents.len() < 2 {
                return None;
            }
            let mut result = 0.0;
            for i in &parents {
                match i {
                    None => return None,
                    Some(val) => result += val,
                }
            }
            Some(result)
        }
        Operation::Mult => {
            if parents.len() < 2 {
                return None;
            }
            let mut result = 1.0;
            for i in &parents {
                match i {
                    None => return None,
                    Some(val) => result *= val,
                }
            }
            Some(result)
        }
    }
}

fn find_root(nodes: &HashMap<String, Node>) -> Option<String> {
    let mut root_name = String::new();
    let mut found = false;
    for (name, node) in nodes.iter() {
        if node.children.is_empty() {
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
    for node in &nodes[name].parents {
        if node == root {
            return true;
        }
        if has_cycle(root, node, visited, nodes) {
            return true;
        }
    }
    false
}

fn evaluate_expr(
    g: &Graph,
    operations: &HashMap<String, Operation>,
) -> Option<f64> {
    let mut nodes = HashMap::new();
    for vert in &g.vertices {
        nodes.insert(vert.clone(), Node::new(operations[vert].clone()));
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

    evaluate_node(root_node, &nodes)
}

#[derive(Parser)]
struct Config {
    #[arg(long, value_name = "FILE")]
    input1: String,
    #[arg(long, value_name = "FILE")]
    input2: String,
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
    let g = match parse_input_graph(&input) {
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

    let operations = match File::open(config.input2) {
        Ok(mut ops_f) => {
            let mut ser = String::new();
            ops_f
                .read_to_string(&mut ser)
                .expect("Не удалось прочитать из файла с операциями");
            serde_json::from_str(&ser)
                .expect("Неверный формат файла с операциями")
        }
        Err(_) => {
            println!("Не удалось прочитать операции из указанного файла");
            return;
        }
    };

    let call_string = match evaluate_expr(&g, &operations) {
        Some(s) => s,
        None => {
            println!("Некорректный ввод - в графе есть циклы");
            return;
        }
    };
    output
        .write_all(call_string.to_string().as_bytes())
        .expect("Не удалось записать вывод в файл");
}
