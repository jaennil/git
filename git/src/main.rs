use std::{
    ffi::CStr,
    fs::{self, metadata, File},
    io::{BufRead as _, BufReader, Read, Write as _},
    path::{Path, PathBuf},
    str::from_utf8,
};

use anyhow::Context as _;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

use clap::{Parser, Subcommand};
use sha1_smol::Sha1;

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
    HashObject {
        filepath: PathBuf,
        #[arg(short)]
        write: bool,
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
            let zlib_decoder = ZlibDecoder::new(object_file);
            let mut zlib_decoder_reader = BufReader::new(zlib_decoder);
            let mut buffer: Vec<u8> = Vec::new();
            zlib_decoder_reader
                .read_until(0, &mut buffer)
                .context("read header of {object_path}")?;
            let header = CStr::from_bytes_with_nul(&buffer)
                .expect("know there is exactly on nul, and it's at the end");
            let header = header
                .to_str()
                .context("object header is not valid UTF-8")?;
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    "object file header must contain space delimiting object type and size"
                );
            };
            let size = size
                .parse::<usize>()
                .context("object file has invalid size: {size}")?;
            match kind {
                "blob" => {
                    buffer.clear();
                    buffer.resize(size, 0);
                    zlib_decoder_reader
                        .read_exact(&mut buffer)
                        .context("read contents of the object")?;
                    let left_unread = zlib_decoder_reader
                        .read(&mut [0])
                        .context("validate EOF in object")?;
                    anyhow::ensure!(left_unread == 0, "object file has trailing bytes");
                    let mut stdout = std::io::stdout();
                    stdout
                        .write_all(&buffer)
                        .context("write object contents to stdout")?;
                }
                _ => anyhow::bail!("can't print {kind} yet"),
            }
        }
        Command::HashObject { filepath, write } => {
            let filebytes = fs::read(&filepath).context("read passed file contents")?;
            let mut object = b"blob ".to_vec();
            object.extend(filebytes.len().to_string().as_bytes());
            object.push(0);
            object.extend(filebytes);
            let sha1 = Sha1::from(&object).digest().to_string();
            if write {
                let dir_name = &sha1[..2];
                let filename = &sha1[2..];
                let dir_path: PathBuf = [BASE_FOLDER, OBJECTS_FOLDER, dir_name].iter().collect();
                fs::create_dir(&dir_path).context("create object directory")?;
                let mut object_filepath = PathBuf::from(&dir_path);
                object_filepath.push(filename);
                let file = File::create(object_filepath).context("create object file")?;
                let mut zlib_encoder = ZlibEncoder::new(file, Compression::default());
                zlib_encoder
                    .write_all(&object)
                    .context("write object to zlib encoder")?;
                zlib_encoder
                    .finish()
                    .context("flush zlib encoded object to the file")?;
            }
            println!("{sha1}");
        }
    }

    Ok(())
}
