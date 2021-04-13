//! Tries to get the path of `dot` from a set of known paths.
//! Operates in differents way based on os.

use std::path::Path;

const NOT_FOUND: &str= "dot executable not found. Please insert it into system PATH or set the environment variable DOT_PATH";

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfile", target_os = "android"))]
/// Tries to get the path of `dot` from a set of known paths.
pub fn find_dot() -> String{
  if which::which("dot").is_ok() {
    return String::from("dot");
  }
  if Path::new("/usr/bin/env").exists(){
    return String::from("/usr/bin/env dot");
  }
  if Path::new("/usr/bin/dot").exists(){
    return String::from("/usr/bin/dot");
  }
  panic!(NOT_FOUND);
}

#[cfg(target_os = "macos")]
pub fn find_dot() -> String{
  if let Ok(_) = which::which("dot") {
    return String::from("dot");
  }
  if Path::new("/usr/local/bin/env").exists(){
    return String::from("/usr/local/bin/env dot");
  }
  if Path::new("/usr/local/bin/dot").exists(){
    return String::from("/usr/local/bin/dot");
  }
  panic!(NOT_FOUND);
}

#[cfg(target_os = "windows")]
pub fn find_dot() -> String{
  if let Ok(_) = which::which("dot.exe") {
    return String::from("dot.exe");
  }
  if let Ok(_) = which::which("dot") {
    return String::from("dot");
  }
  //String::from("C:/Program Files (x86)/Graphviz 2.28/bin/dot.exe")
  panic!(NOT_FOUND);
}

#[cfg(target_os = "ios")]
pub fn find_dot() -> String{
  panic!("Calling dot is not supported on this target os");
}
