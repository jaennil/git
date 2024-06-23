use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use flate2::read::ZlibDecoder;

use std::fmt;

use clap::{Parser, Subcommand};

const BASE_FOLDER: &str = ".git";
const OBJECTS_FOLDER: &str = "objects";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

enum ObjectType {
    Blob,
}

#[derive(Subcommand)]
enum Command {
    Init,
    CatFile {
        #[arg(short)]
        pretty_print: bool,
        object_hash: String,
    },
}

enum AppError {
    InvalidSha,
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSha => write!(f, "object hash must be valid sha1"),
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for AppError {}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print: _,
            object_hash,
        } => {
            if object_hash.len() != 40 {
                return Err(Box::new(AppError::InvalidSha));
            }

            let object_folder = &object_hash[..2];
            let object_file = &object_hash[2..];

            let path: PathBuf = [BASE_FOLDER, OBJECTS_FOLDER, object_folder, object_file]
                .iter()
                .collect();

            let file = File::open(path)?;

            let mut z = ZlibDecoder::new(file);
            let mut s = String::new();

            z.read_to_string(&mut s)?;
            let s: String = s.split('\0').skip(1).collect();
            println!("{s}");
        }
    }

    Ok(())
}
