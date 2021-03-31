mod graph;
mod dot;

use argparse::{ArgumentParser, Collect, Store};
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, stdin, BufWriter, stdout};
use std::process::Command;

fn main() {

  let mut input_file = String::new();
  let mut output_file = String::new();
  let mut dot_format = String::new();
  let mut dot_options: Vec<String> = Vec::new();
  let mut dot_path = String::new();

  // Parsing arguments
  {
    let mut args = ArgumentParser::new();
    args.set_description("Convert BCALM de Bruijn graphs to DOT diagrams.");
    args.refer(&mut input_file)
        .add_argument("INPUT", Store,"BCALM FASTA file to convert. If no provided the file will be read from stdin.");
    args.refer(&mut output_file)
        .add_option(&["-o"], Store, "Output DOT file. If no provided the output file will depend on input one.")
        .metavar("FILE");
    args.refer(&mut dot_format)
        .add_option(&["-d", "--dot"], Store, "Invoke dot to generate the graph with the specified format type at the end.")
        .metavar("TYPE");
    args.refer(&mut dot_options)
        .add_argument("DOT OPTIONS", Collect, "Options passed to dot.");
    args.refer(&mut dot_path)
        .envvar("DOT_PATH");
    args.parse_args_or_exit();
  }

  // Defualting arguments
  if input_file != "" && output_file == ""{
    let mut path = PathBuf::from(&input_file);
    path.set_extension("gv");
    output_file = String::from(path.to_str().unwrap());
  }
  if dot_format != "" && dot_path == ""{
    dot_path = dot::find_dot();
  }

  // Read BCAL FASTA file and generate graph
  let mut graph;
  {
    graph = match input_file.len(){
      0 => graph::Graph::from(BufReader::new(stdin())),
      _ => graph::Graph::from(BufReader::new(File::open(input_file).unwrap()))
    };
  }

  // Write DOT output file
  {
    match output_file.len(){
      0 => graph.plot(&mut BufWriter::new(stdout())),
      _ => graph.plot(&mut BufWriter::new(File::create(&output_file).unwrap()))
    };
  }

  // Inovke dot
  if dot_format != ""{
    let path_split: Vec<_>= dot_path.split(" ").collect();
    Command::new(path_split[0])
      .args(&path_split[1..])
      .arg("-O")
      .args(dot_options)
      .arg(format!("-T{}", dot_format))
      .arg(output_file)
      .spawn().unwrap();
  }

}

