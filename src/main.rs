use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match journal::run(&args) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
