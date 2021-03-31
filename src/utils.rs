
pub fn complement(nucleo: char) -> Option<char>{
  match nucleo{
    'A' => Some('T'),
    'C' => Some('G'),
    'G' => Some('C'),
    'T' => Some('A'),
    _ => None
  }
}