use burn::backend::Wgpu;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use burn::{
    module::Module,
    nn::{Linear, LinearConfig},
    record::{FullPrecisionSettings, PrettyJsonFileRecorder, Recorder},
    tensor::{activation::sigmoid, backend::Backend, DataSerialize, Tensor},
};

#[derive(Module, Debug)]
pub struct MyModel<B: Backend> {
    layers: Vec<Linear<B>>,
}

impl<B: Backend> MyModel<B>
where
    burn::tensor::Data<<B as Backend>::FloatElem, 2>: From<DataSerialize<f32>>,
{
    fn forward(&self, data: Tensor<B, 2>) -> Tensor<B, 2> {
        let mut x = data;
        for linear in &self.layers {
            x = linear.forward(x);
            x = sigmoid(x);
        }
        if self.layers.len() % 2 != 0 {
            sigmoid(self.layers[1].forward(x))
        } else {
            x
        }
    }

    fn from_raw(layer_data: LayersData) -> MyModel<B> {
        let num_classes = layer_data.weights[0].len();
        let hidden_size = layer_data.weights[0][0].len();
        let mut layers = Vec::new();

        let mut transpose = true;
        for layer in layer_data.weights {
            let mut linear = if transpose {
                LinearConfig::new(num_classes, hidden_size).init()
            } else {
                LinearConfig::new(hidden_size, num_classes).init()
            };

            let tensor = Tensor::<B, 2>::from_data(DataSerialize {
                value: layer.into_iter().flatten().collect(),
                shape: vec![num_classes, hidden_size],
            });
            let tensor = if transpose {
                tensor.transpose()
            } else {
                tensor
            };
            linear.weight = tensor.into();
            linear.bias = None;
            layers.push(linear);
            transpose = !transpose;
        }
        MyModel { layers }
    }

    fn new() -> MyModel<B> {
        MyModel { layers: Vec::new() }
    }

    fn init_with(&self, record: MyModelRecord<B>) -> MyModel<B> {
        let mut layers = Vec::new();
        for layer_record in record.layers {
            let [num_classes, hidden_size] = layer_record.weight.shape().dims;
            layers.push(
                LinearConfig::new(num_classes, hidden_size)
                    .init_with(layer_record),
            );
        }
        MyModel { layers }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LayersData {
    weights: Vec<Vec<Vec<f32>>>,
}

fn parse_xs<B: Backend>(s: &str) -> Option<Tensor<B, 2>>
where
    burn::tensor::Data<<B as Backend>::FloatElem, 2>: From<DataSerialize<f32>>,
{
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
    Some(Tensor::<B, 2>::from_data(DataSerialize {
        shape: vec![1, nums.len()],
        value: nums,
    }))
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Сконвертировать веса из формата в задании в сериализованную модель
    Convert {
        /// Путь до файла с весами
        #[arg(long)]
        weights: PathBuf,
        /// Путь, куда записать сериализованную модель
        #[arg(long)]
        output: PathBuf,
    },
    /// Запустить вычисления НС с указанной моделью
    Run {
        /// Путь до сериализованной модели
        #[arg(long, value_name = "FILE")]
        model: PathBuf,
        /// Путь до файла с входным вектором
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
        /// Путь до файла для записи
        #[arg(long, value_name = "FILE")]
        output: PathBuf,
    },
}

fn main() {
    type MyBackend = Wgpu;

    let cli = Cli::parse();

    match cli.command {
        Commands::Convert { weights, output } => {
            let data: LayersData = match File::open(weights) {
                Ok(mut file) => {
                    let mut ser = String::new();
                    file.read_to_string(&mut ser)
                        .expect("Не удалось прочитать файл с весами");
                    serde_json::from_str(&ser)
                        .expect("Неверный формат файла с весами")
                }
                Err(_) => {
                    println!("Не удалось прочитать граф из указанного файла");
                    return;
                }
            };
            let m: MyModel<MyBackend> = MyModel::from_raw(data);
            m.save_file(
                output,
                &PrettyJsonFileRecorder::<FullPrecisionSettings>::new(),
            )
            .expect("Не удалось записать модель по указанному пути");
        }
        Commands::Run {
            model,
            input,
            output,
        } => {
            let record = PrettyJsonFileRecorder::<FullPrecisionSettings>::new()
                .load(model)
                .expect("Trained model should exist");
            let m = MyModel::new().init_with(record);

            let x = match File::open(input) {
                Ok(mut file) => {
                    let mut ser = String::new();
                    file.read_to_string(&mut ser)
                        .expect("Не удалось прочитать файл с входным вектором");
                    parse_xs::<MyBackend>(&ser)
                        .expect("Неверный формат файла с входным вектором")
                }
                Err(_) => {
                    println!("Не удалось прочитать граф из указанного файла");
                    return;
                }
            };

            let mut output = File::create(output)
                .expect("Не удалось создать файл для вывода");

            let result = m.forward(x);

            output
                .write_all(result.to_data().to_string().as_bytes())
                .expect("Не удалось записать вывод в файл");
        }
    }
}
