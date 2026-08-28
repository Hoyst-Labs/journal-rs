use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let execution = journal::run_cli(&args);

    if let Some(output) = execution.stdout {
        println!("{output}");
    }
    if let Some(diagnostic) = execution.stderr {
        eprintln!("{diagnostic}");
    }

    process::exit(execution.exit.code());
}
