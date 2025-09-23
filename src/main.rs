use std::{env, io, io::Write, process, fmt};
use brdb::{Brdb, fs::BrFs, schema::ReadBrdbSchema, BrReader, BrFsReader, IntoReader};

/// convert a vector array of strings to a multiline string
fn strings_to_lines<I, T>(iter: I) -> String
where
    I: Iterator<Item = T>,
    T: AsRef<str>,
{
    let mut buf = String::new();
    for name in iter {
        buf.push_str(name.as_ref());
        buf.push('\n');
    }
    buf
}

/// get brfs object based on path
#[allow(dead_code)]
#[derive(Debug)]
enum TraverseError {
  NoParentOfRoot,
  NotFound(String),
  TraverseIntoFile,
}
impl fmt::Display for TraverseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TraverseError::NoParentOfRoot => write!(f, "You tried to go above the root directory."),
      TraverseError::NotFound(path) => write!(f, "cannot access '{}': No such file or directory.", path),
      TraverseError::TraverseIntoFile => write!(f, "Tried to traverse into a file, not a folder."),
    }
  }
}

/// get a BrFs object for the given path
fn traverse<'a>(root: &'a BrFs, path: &str) -> Result<&'a BrFs, TraverseError> {
  // thanks to voximity for this function
  let mut traversal = vec![root];
  for part in path.split('/') {
    match part {
      "." => (),
      ".." => {
        traversal.pop().ok_or(TraverseError::NoParentOfRoot)?;
      }
      part => match traversal.last().ok_or(TraverseError::NoParentOfRoot)? {
        BrFs::Root(map) | BrFs::Folder(_, map) => match map.get(part) {
          Some(v) => traversal.push(v),
          None => return Err(TraverseError::NotFound(part.to_string())),
        },
        BrFs::File(_) => return Err(TraverseError::TraverseIntoFile),
      },
    }
  }
  traversal.pop().ok_or(TraverseError::NoParentOfRoot)
}

/// show files in specified path
fn list_dir(fs: BrFs, path: &str) -> Result<String, TraverseError> {
    let mut path_ = path;
    path_ = path_.trim_start_matches("/");
    path_ = path_.trim_end_matches("/");

    let sub_fs = match path_ {
        "" => &fs,
        _  => traverse(&fs, path_)?
    };

    match &sub_fs {
          BrFs::Root(map) => Ok(strings_to_lines(map.keys())),
          BrFs::Folder(_, map) => Ok(strings_to_lines(map.keys())),
          _ => {
              /* 
               * lol just show the path to the file
               * like what linux `ls` does
               */
              Ok(String::from(path_))
          }
    }
}

/// read file in brdb based on file type
fn read_file(db: BrReader<Brdb>, path: &str) -> Result<String, &str> {
    let (_file_name, file_ext) = path.split_once(".").unwrap();

    match file_ext {
        "schema" => {
            // fetch the raw file data
            let schema = db.read_file(path)
                .expect("couldnt read file")
            .as_slice()
            // convert it to a schema object
            .read_brdb_schema_with_data(
                db.global_data().expect("couldnt get global data")
            )
                .expect("couldnt read schema");

            // return a string representation of the schema
            Ok(String::from(format!("{schema}")))
        }
        "json" => {
            // get raw file bytes
            let file_bytes = db.read_file(path).expect("couldnt read file");
            // convert bytes to string
            let file: &str = str::from_utf8(&file_bytes).expect("couldnt convert file bytes to str");

            // return file as string
            Ok(String::from(file))
        }
        "mps" => {
            // get raw file bytes
            let file_bytes = db.read_file(path).expect("couldnt read file");
            std::io::stdout().write(&file_bytes);

            // return file as string
            Ok(String::from(""))
        }
        _ => {
            Err("Invalid file type")
        }
    }
}

/*
/// TODO: open a file in your favorite editor and save it into the brdb once finished
fn edit_file(db: BrReader<Brdb>, path: &str) -> Result<String, &str> {
    Err("this function isn't ready yet")
}
*/

fn main() {
    let argv: Vec<_> = env::args().collect();

    if argv.len() < 4 {
        println!("usage: {0} <world file path> <ls|read|edit> <path>", argv[0]);
        process::exit(0);
    }
    let arg_name: &str = &argv[0];
    let arg_world_path: &str = &argv[1];
    let arg_cmd: &str = &argv[2];
    let arg_file_path: &str = &argv[3].trim_start_matches("/");

    let db = Brdb::open(arg_world_path).expect("couldnt open file").into_reader();
    let fs: BrFs = db.get_fs().expect("couldnt get fs");

    let output = match arg_cmd {
        "ls" => match list_dir(fs, arg_file_path) {
             Ok(value) => value,
             Err(error) => format!("error: {error}"),
        },
        "read" => read_file(db, arg_file_path).expect("couldnt read file"),
        /* "edit" => edit_file(db, arg_file_path).expect("error"), */
        _ => String::from(format!("invalid command: {arg_cmd}. use one of: <ls|read|edit>"))
    };

    println!("{output}");
}
