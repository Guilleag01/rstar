use std::{fs::File, io::Write};

use clap::Parser;
use rstar::files::get_elements_from_path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of the directory to list
    #[arg()]
    path: String,
}

fn main() {
    let _args = Args::parse();
    // let test = FileData::default();
    // let x = test.

    let mut file = File::create("test2.tar").unwrap();

    let data = &get_elements_from_path(&"test_dir".to_string())[0];

    file.write_all(&data.get_binary_header()).unwrap();

    // for e in get_elements_from_path(&"test_dir".to_string()) {
    //     println!("{:?}", e.get_binary_header());

    //     file.write(e.get_binary_header());
    // }
}
