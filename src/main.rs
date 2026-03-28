// src/main.rs

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

mod app;
mod text;
mod screen;
mod widget;
mod usr;
mod util;

use crate::app::App;
use std::{
  fs, env, 
  io::{self, stdout},
};

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let default_path = String::from(".gemset");
  let init_path    = args.get(1).unwrap_or(&default_path); 

  let mut stdout = stdout();
  App::run(init_path, &mut stdout)
}
