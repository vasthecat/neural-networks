use burn::backend::Wgpu;
use burn::tensor::Tensor;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

type Backend = Wgpu;

fn sigmoid(x: Tensor<Backend, 1>) -> Tensor<Backend, 1> {
    (x.ones_like() + x.neg().exp()).recip()
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    weights: Vec<Vec<Vec<f32>>>,
}

fn parse_xs(s: &str) -> Option<Tensor<Backend, 1>> {
    let mut nums = Vec::new();
    let mut q = s;
    loop {
        match q.find(',') {
            Some(idx) => {
                let num = q[..idx].trim().parse::<f32>().ok()?;
                nums.push(num);
                q = &q[idx + 1..];
            }
            None => {
                let num = q.trim().parse::<f32>().ok()?;
                nums.push(num);
                break;
            }
        }
    }
    Some(Tensor::<Backend, 1>::from_data(nums.as_slice()))
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

    let data: Data = match File::open(config.input1) {
        Ok(mut file) => {
            let mut ser = String::new();
            file.read_to_string(&mut ser)
                .expect("Не удалось прочитать файл с весами");
            serde_json::from_str(&ser).expect("Неверный формат файла с весами")
        }
        Err(_) => {
            println!("Не удалось прочитать граф из указанного файла");
            return;
        }
    };

    let mut tx = match File::open(config.input2) {
        Ok(mut file) => {
            let mut ser = String::new();
            file.read_to_string(&mut ser)
                .expect("Не удалось прочитать файл с входным вектором");
            parse_xs(&ser).expect("Неверный формат файла с входным вектором")
        }
        Err(_) => {
            println!("Не удалось прочитать граф из указанного файла");
            return;
        }
    };

    let mut output = File::create(config.output1)
        .expect("Не удалось создать файл для вывода");

    for layer in data.weights {
        for w in layer {
            let tensor = Tensor::<Backend, 1>::from_data(w.as_slice());
            tx = sigmoid(tensor * tx);
        }
    }

    output
        .write_all(tx.to_data().to_string().as_bytes())
        .expect("Не удалось записать вывод в файл");
}
