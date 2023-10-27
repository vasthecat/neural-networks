use clap::Parser;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

#[derive(Debug)]
struct Arc {
    from: String,
    to: String,
    order: i32,
}

impl Arc {
    fn write_xml<T>(&self, writer: &mut EventWriter<T>) -> Result<()>
    where
        T: Write,
    {
        writer.write(XmlEvent::start_element("arc"))?;
        writer.write(XmlEvent::start_element("from"))?;
        writer.write(XmlEvent::characters(&self.from))?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::start_element("to"))?;
        writer.write(XmlEvent::characters(&self.to))?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::start_element("order"))?;
        writer.write(XmlEvent::characters(&self.order.to_string()))?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

#[derive(Debug)]
struct Graph {
    vertices: Vec<String>,
    arcs: Vec<Arc>,
}

impl Graph {
    fn write_xml<T>(&self, writer: &mut EventWriter<T>) -> Result<()>
    where
        T: Write,
    {
        writer.write(XmlEvent::start_element("graph"))?;
        for vertex in &self.vertices {
            writer.write(XmlEvent::start_element("vertex"))?;
            writer.write(XmlEvent::characters(vertex))?;
            writer.write(XmlEvent::end_element())?;
        }
        for arc in &self.arcs {
            arc.write_xml(writer)?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
enum ParserState {
    Lparen,
    From,
    To,
    Order,
    Comma,
}

fn parse_input(input: &str) -> Option<Graph> {
    let mut expecting = ParserState::Lparen;
    let mut vertices = HashSet::new();
    let mut arcs = Vec::new();
    let mut from = String::new();
    let mut to = String::new();
    let mut order = String::new();
    for char in input.chars() {
        if char.is_ascii_whitespace() {
            continue;
        }
        match expecting {
            ParserState::Lparen => {
                if char != '(' {
                    return None;
                } else {
                    expecting = ParserState::From;
                }
            }
            ParserState::From => {
                if char == ',' {
                    expecting = ParserState::To;
                } else if char.is_ascii_alphanumeric() {
                    from.push(char);
                } else {
                    return None;
                }
            }
            ParserState::To => {
                if char == ',' {
                    expecting = ParserState::Order;
                } else if char.is_ascii_alphanumeric() {
                    to.push(char);
                } else {
                    return None;
                }
            }
            ParserState::Order => {
                if char == ')' {
                    expecting = ParserState::Comma;
                    match order.parse::<i32>() {
                        Ok(order) => arcs.push(Arc {
                            from: from.clone(),
                            to: to.clone(),
                            order,
                        }),
                        Err(_) => return None,
                    }
                    vertices.insert(from.clone());
                    vertices.insert(to.clone());
                    from.clear();
                    to.clear();
                    order.clear();
                } else if char.is_ascii_digit() {
                    order.push(char);
                } else {
                    return None;
                }
            }
            ParserState::Comma => {
                if char == ',' {
                    expecting = ParserState::Lparen;
                }
            }
        }
    }
    if expecting != ParserState::Comma {
        None
    } else {
        let mut vertices: Vec<_> = vertices.into_iter().collect();
        vertices.sort();
        arcs.sort_by(|a, b| a.order.partial_cmp(&b.order).unwrap());
        Some(Graph { vertices, arcs })
    }
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
    let input = match fs::read_to_string(config.input1) {
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
    let output = match File::create(config.output1) {
        Ok(out) => out,
        Err(_) => {
            println!("Не удалось создать файл для вывода");
            return;
        }
    };
    let mut writer = EmitterConfig::new()
        .write_document_declaration(false)
        .perform_indent(true)
        .create_writer(output);
    if let Err(err) = g.write_xml(&mut writer) {
        println!("{err}");
    }
}
