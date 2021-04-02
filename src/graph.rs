//! Represents a de Bruijn graph

#[path="./utils.rs"]
mod utils;

use snafu::Snafu;
use std::io::{BufRead, Write};
use regex::Regex;

#[derive(Debug, Snafu)]
/// Describes and error on graph generation
enum GraphError {
  #[snafu(display("Unknown '{}' nucleotide", nucleo))]
  WrongNucleotide{nucleo: char},
  #[snafu(display("Unknown nucleotide into sequence \"{}\"", seq))]
  WrongNucleotideInto{seq: String}
}

/// Represents a graph node
struct Node{
  /// Sequence of nucleotides
  seq: String,
  /// Complement of the sequence
  compl: String,
  /// vector of kmer counts
  count: Vec<u32>,
  /// Sequence direction
  dir: bool
}

impl Node{
  /// Creates a new node containing the given sequence and vector of counts.
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

  /// Set the direction of the sequence according to the direction symbol ('+' or '-')
  fn set_dir(self: &mut Self, dir: char){
    self.dir = dir == '+';
  }
  /// Get the direction symbol ('+' or '-')
  fn dir_sign(self: &Self) -> char{
    if self.dir {'+'} else {'-'}
  }
  /// Check the direction symbol ('+' or '-')
  fn is_dir(self: &Self, dir: char) -> bool{
    dir == self.dir_sign()
  }
}

/// Represents a graph edge
struct Edge{
  /// Index of starting node
  from: usize,
  /// Index of arrival node
  to: usize,
  /// Starting direction symbol
  start: char,
  /// Arrival direction symbol
  end: char
}

/// Represents a de Bruijn graph
pub struct Graph{
  nodes: Vec<Node>,
  edges: Vec<Edge>
}

impl Graph{
  /// Builds an empty graph
  fn new() -> Graph{
    Graph{nodes: Vec::new(), edges: Vec::new()}
  }

  /// Appends a new node to the graph
  fn append(self: &mut Self, seq: String, count: Vec<u32>) -> Result<(), GraphError>{
    self.nodes.push(Node::new(seq, count)?);
    Ok(())
  }

  /// Adds an edge from the last inserted node to another one
  fn to(self: &mut Self, start: char, to: usize, end: char){
    let from = self.nodes.len()-1;
    if from!= to{
      self.edges.push(Edge{from, to, start, end});
    }
  }

  /// Creates a dot file for the graph and writes it into the given buffer
  pub fn plot<T: Write>(self: &mut Self, buf: &mut T, dir: bool, symbol: bool){

    // Sets the direction of the nucleotides
    if dir{
      let mut fixed_dir = vec![false; self.nodes.len()];
      for e in self.edges.iter(){
        if self.nodes[e.from].is_dir(e.start) && !fixed_dir[e.to]{
          fixed_dir[e.from] = true;
          self.nodes[e.to].set_dir(e.end);
          fixed_dir[e.to] = true;
        }
      }
    }

    // Writes to buffer
    buf.write(b"graph genome {\n\trankdir=\"LR\";\n").unwrap();
    // Writes all nodes
    for (i, node) in self.nodes.iter().enumerate(){
      writeln!(buf,
        "\t{} [label=\"{}\\n({})\\n{:?}\", shape={} {}margin=0.2];",
        i, node.seq, node.compl, node.count, if dir{"cds"}else{"rectangle"}, if !dir || node.dir{""}else{"orientation=180 "}
      ).unwrap();
    }
    buf.write(b"\n").unwrap();
    // Writes all edges which match the direction of the sequences
    for e in self.edges.iter(){
      if self.nodes[e.from].is_dir(e.start){
        write!(buf,
          "\t{}:{} -- {}:{}",
          e.from, "e", //if self.nodes[e.from].dir {"e"} else {"w"},
          e.to, "w" ,//if self.nodes[e.to].dir {"w"} else {"e"},
        ).unwrap();
        if symbol{
          write!(buf,
            " [taillabel=\"{}\", headlabel=\"{}\"]",
            e.start, e.end
          ).unwrap();
        }
        writeln!(buf, ";").unwrap();
      }
    }
    buf.write(b"}\n").unwrap();
  }
}

impl<T: BufRead> std::convert::From<T> for Graph{
  /// Build a de Bruijn graph from FASTA file
  fn from(buf: T) -> Graph{
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
      let count = match count_re.captures(&opt) {
          Some(r) => r.get(1).unwrap().as_str(),
          None => ""
        }.split(' ').map(|s| s.parse().unwrap()).collect();

      // Append this node
      if let Err(e) = graph.append(line, count) {
        panic!("{}; on line {}", e, index+1);
      }

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
