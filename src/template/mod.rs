use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use std::process::Command;

pub mod meta;

use crate::repository::Repository;
use crate::utils;

#[derive(Clone, Debug)]
pub struct Options {
  pub name: String,
  pub repository: Option<String>,
  pub username: Option<String>,
  pub email: Option<String>,
  pub replace: bool,
}

pub struct Template {
  pub name: String,
  pub path: String,
  pub description: Option<String>,
  pub scripts: Option<meta::Scripts>,
  pub extend: Option<Vec<String>>,
  pub exclude: Option<Vec<String>>,
}

extern crate custom_error;
use custom_error::custom_error;

custom_error! {pub TemplateError
  InitializeTemplate = "Unable to initialize template"
}

impl Template {
  pub fn new(dir: &std::fs::DirEntry) -> Result<Template, TemplateError> {
    let path = dir.path().to_string_lossy().into_owned();
    let meta = meta::load_meta(&path).unwrap();

    // If type is None or unqual template skip entry
    if meta.kind.is_none() || meta.kind != Some(String::from("template")) {
      return Err(TemplateError::InitializeTemplate);
    }

    let name;
    if meta.name.is_none() {
      name = dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    } else {
      name = meta.name.unwrap();
    }

    // make all names lowercase
    return Ok(Template {
      name: utils::lowercase(&name),
      path: path,
      description: meta.description,
      scripts: meta.scripts,
      extend: meta.extend,
      exclude: meta.exclude,
    });
  }

  pub fn copy(&self, repository: &Repository, target: &Path, opts: Options) -> Result<(), Error> {
    // get list of all super templates
    let super_templates = match &self.extend {
      None => Vec::new(),
      Some(x) => x.clone(),
    };

    for name in super_templates {
      let template = repository.get_template_by_name(&name).unwrap();
      let opts = opts.clone();
      template.copy(repository, target, opts)?;
    }

    if self.scripts.is_some() && self.scripts.as_ref().unwrap().before_install.is_some() {
      let script = self
        .scripts
        .as_ref()
        .unwrap()
        .before_install
        .as_ref()
        .unwrap();

      run_script(script, target);
    }

    self.copy_folder(&self.path, &target, &opts)?;

    Ok(())
  }

  fn copy_folder(&self, src: &str, target: &Path, opts: &Options) -> Result<(), Error> {
    // Loop at selected template directory
    for entry in fs::read_dir(src)? {
      let entry = &entry.unwrap();

      let source_path = &entry.path().to_string_lossy().into_owned();
      let source_name = &entry
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

      let mut path = target.to_path_buf();
      path.push(source_name);

      // replace placeholders in path
      path = PathBuf::from(replace_placeholders(&path.to_string_lossy(), &opts)?);

      // check if entry is directory
      if entry.path().is_dir() {
        match fs::create_dir(&path) {
          Ok(()) => (),
          Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => (),
            _ => return Err(error),
          },
        };

        self.copy_folder(&source_path, &path, opts)?
      } else {
        if is_excluded(&source_name, &self.exclude) {
          continue;
        }

        // Open file
        let mut src = File::open(Path::new(&source_path))?;
        let mut data = String::new();

        // Write to data string
        src.read_to_string(&mut data)?;

        // close file
        drop(src);

        // replace placeholders in data
        data = replace_placeholders(&data, &opts)?;

        // create file
        let mut dst = File::create(path)?;
        dst.write(data.as_bytes())?;
      }
    }

    Ok(())
  }
}

fn run_script(script: &String, target: &Path) {
  // Run before script if exists
  let output = if cfg!(target_os = "windows") {
    Command::new("cmd")
      .current_dir(target)
      .arg(script)
      .output()
      .ok()
      .expect("failed to execute process")
  } else {
    Command::new("sh")
      .current_dir(target)
      .arg("-c")
      .arg(script)
      .output()
      .ok()
      .expect("failed to execute process")
  };

  let result = output.stdout;
  println!("{}", String::from_utf8_lossy(&result));

  let error = output.stderr;
  println!("{}", String::from_utf8_lossy(&error));
}

fn is_excluded(name: &str, exclude: &Option<Vec<String>>) -> bool {
  if name == "meta.json" {
    return true;
  };

  let items = match &exclude {
    None => return false,
    Some(x) => x,
  };

  // check meta exclude
  for item in items.iter() {
    if item == &name {
      return true;
    }
  }

  return false;
}

fn replace_placeholders(data: &str, opts: &Options) -> Result<String, Error> {
  // replace placeholder with actual value
  let mut data = data.replace("{{name}}", &opts.name);

  if !opts.repository.is_none() {
    data = data.replace("{{repository}}", opts.repository.as_ref().unwrap());
  }

  if !opts.username.is_none() {
    data = data.replace("{{username}}", opts.username.as_ref().unwrap());
  }

  if !opts.email.is_none() {
    data = data.replace("{{email}}", opts.email.as_ref().unwrap());
  }

  Ok(data)
}