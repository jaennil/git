use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use anyhow::Context as _;
use flate2::read::ZlibDecoder;

use clap::{Parser, Subcommand};

const BASE_FOLDER: &str = ".git";
const OBJECTS_FOLDER: &str = "objects";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            // TODO: add pwd
            println!("Initialized empty Git repository in ...")
        }
        Command::CatFile {
            pretty_print: _,
            object_hash,
        } => {
            anyhow::ensure!(object_hash.len() <= 40, "object hash must be valid SHA-1");
            let object_folder = &object_hash[..2];
            let object_filename = &object_hash[2..];
            let object_path: PathBuf =
                [BASE_FOLDER, OBJECTS_FOLDER, object_folder, object_filename]
                    .iter()
                    .collect();
            let object_file =
                File::open(object_path).context("can't find object file at {object_path}")?;
            let mut zlib_decoder = ZlibDecoder::new(object_file);
            let mut string_buffer = String::new();
            zlib_decoder
                .read_to_string(&mut string_buffer)
                .context("data in object is not valid UTF-8")?;
            let result: String = string_buffer.split('\0').skip(1).collect();
            println!("{result}");
        }
    }

    Ok(())
}
