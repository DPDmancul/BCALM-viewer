#[path="./utils.rs"]
mod utils;

use snafu::Snafu;
use std::io::{BufRead, Write};
use regex::Regex;

#[derive(Debug, Snafu)]
enum GraphError {
  #[snafu(display("Unknown '{}' nucleotide", nucleo))]
  WrongNucleotide{nucleo: char},
  #[snafu(display("Unknown nucleotide into sequence \"{}\"", seq))]
  WrongNucleotideInto{seq: String}
}

struct Node{
  seq: String,
  compl: String,
  count: Vec<u32>,
  dir: bool
}

impl Node{
  fn new(seq: String, count: Vec<u32>) -> Result<Node, GraphError>{
    if let Some(compl) = seq.chars().map(utils::complement).rev().collect(){
      Ok(Node{
        seq,
        compl,
        count,
        dir: true
      })
    }else{
      Err(GraphError::WrongNucleotideInto{seq})
    }
  }

  fn set_dir(self: &mut Self, dir: char){
    self.dir = dir == '+';
  }
  fn dir_sign(self: &Self) -> char{
    if self.dir {'+'} else {'-'}
  }
  fn is_dir(self: &Self, dir: char) -> bool{
    dir == self.dir_sign()
  }
}

struct Edge{
  from: usize,
  to: usize,
  start: char,
  end: char
}

pub struct Graph{
  nodes: Vec<Node>,
  edges: Vec<Edge>
}

impl Graph{
  fn new() -> Graph{
    Graph{nodes: Vec::new(), edges: Vec::new()}
  }

  fn append(self: &mut Self, seq: String, count: Vec<u32>) -> Result<(), GraphError>{
    self.nodes.push(Node::new(seq, count)?);
    Ok(())
  }

  fn to(self: &mut Self, start: char, to: usize, end: char){
    let from = self.nodes.len()-1;
    if from!= to{
      self.edges.push(Edge{from, to, start, end});
    }
  }

  pub fn plot(self: &mut Self, buf: &mut Box<dyn Write>){
    let mut fixed_dir = vec![false; self.nodes.len()];
    for e in self.edges.iter(){
      if self.nodes[e.from].is_dir(e.start){
        fixed_dir[e.from] = true;
        self.nodes[e.to].set_dir(e.end);
        fixed_dir[e.to] = true;
      }
    }

    buf.write(b"graph genome {\n\trankdir=\"LR\";\n").unwrap();
    for (i, node) in self.nodes.iter().enumerate(){
      writeln!(buf,
        "\t{} [label=\"{}\\n({})\\n{:?}\", shape=cds, {} margin=0.2];",
        i, node.seq, node.compl, node.count, if node.dir{""}else{"orientation=180"}
      ).unwrap();
    }
    buf.write(b"\n").unwrap();
    for e in self.edges.iter(){
      if self.nodes[e.from].is_dir(e.start){
        writeln!(buf,
          "\t{}:{} -- {}:{};", // [taillabel=\"{}\", headlabel=\"{}\"];",
          e.from, "e", //if self.nodes[e.from].dir {"e"} else {"w"},
          e.to, "w" ,//if self.nodes[e.to].dir {"w"} else {"e"},
          //e.start, e.end
        ).unwrap();
      }
    }
    buf.write(b"}\n").unwrap();
  }
}

impl std::convert::From<Box<dyn BufRead>> for Graph{
  fn from(buf: Box<dyn BufRead>) -> Graph{
    let mut graph = Graph::new();

    let count_re = Regex::new(r"ab:Z:(\d+(?: \d+)*)").unwrap();
    let link_re = Regex::new(r"L:([+-]):(\d+):([+-])").unwrap();

    let mut opt = String::new();

    for (index, line) in buf.lines().enumerate() {
      let line = line.unwrap();

      // If line is even get options
      if index%2 == 0{
        if opt.len() != 0 || line.chars().next().unwrap() != '>'{
          panic!("Syntax error at line {}: \"{}\"", index+1, line);
        }
        opt = line;
        continue;
      }

      // Get counts
      let count = match count_re.captures(&opt){
          Some(r) => r.get(1).unwrap().as_str(),
          None => ""
        }.split(' ').map(|s| s.parse().unwrap()).collect();

      // Append this node
      graph.append(line, count).unwrap();

      // Get edges
      for group in link_re.captures_iter(&opt) {
        graph.to(
          group[1].chars().next().unwrap(),
          group[2].parse().unwrap(),
          group[3].chars().next().unwrap()
        );
      }

      opt = String::new();
    }

    graph
  }
}
