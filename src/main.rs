use clap::Clap;
mod init;

/// Search for a pattern in a file and display the lines that contain it
#[derive(Clap)]
#[clap(version = "0.1", author = "Thomas P.")]
struct Opts {
    name: String,
    path: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let r = init::init_project(opts.name, opts.path);
    match r {
        Ok(v) => println!("Succesfull"),
        Err(e) => println!("Error")
    };
}

