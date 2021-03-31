
/// Gets the complement of a nucleotide.
/// Returns None if the input char is not a valid nucleotide.
pub fn complement(nucleo: char) -> Option<char>{
  match nucleo{
    'A' => Some('T'),
    'C' => Some('G'),
    'G' => Some('C'),
    'T' => Some('A'),
    _ => None
  }
}