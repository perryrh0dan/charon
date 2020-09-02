use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};

use crate::config::{Config, RepositoryOptions};
use crate::git;
use crate::out;
use crate::template;
use crate::meta;
use crate::utils;

extern crate custom_error;
use custom_error::custom_error;

pub struct Repository {
  pub directory: String,
  pub config: RepositoryOptions,
  pub templates: Vec<template::Template>,
}

custom_error! {pub RepositoryError
  InitializationError = "Unable to initialize repository",
  NotFound = "Repository not found",
  TemplateNotFound = "Unable to find template",
  LoadingErrors = "Unable to load templates",
}

impl Repository {
  pub fn new(config: &Config, name: &str) -> Result<Repository, RepositoryError> {
    let cfg = match config.get_repository_config(name) {
      Option::Some(cfg) => cfg,
      Option::None => return Err(RepositoryError::NotFound),
    };

    let directory = String::from(&config.template_dir) + "/" + &utils::lowercase(name);

    let mut repository = Repository {
      directory: directory,
      config: cfg,
      templates: Vec::<template::Template>::new(),
    };

    match repository.ensure_repository_dir(&config) {
      Ok(()) => (),
      Err(_error) => return Err(RepositoryError::InitializationError),
    };

    repository.load_templates();

    return Ok(repository);
  }

  pub fn delete_repository(config: &Config, name: &str) -> Result<(), Error> {
    let mut repository_dir = PathBuf::from(&config.template_dir);
    repository_dir.push(&utils::lowercase(name));

    log::info!("Delete repository directory {}", &repository_dir.to_owned().to_str().unwrap());
    match fs::remove_dir_all(repository_dir) {
      Ok(()) => (),
      Err(error) => {
        log::error!{"{}", error}
        return Err(error)
      }
    }

    return Ok(());
  }

  pub fn create_template(&self, name: &str) -> Result<(), Error> {
    let repository_path = Path::new(&self.directory);
    let template_path = repository_path.join(&name);

    // Create template directory
    fs::create_dir(&template_path)?;

    // Create meta.json
    let meta_path = template_path.join("meta.json");
    let mut meta_file = File::create(meta_path)?;

    // Create meta data
    let mut meta = meta::default();
    meta.kind = Some(String::from("template"));
    meta.name = Some(name.to_owned());
    meta.version = Some(String::from("1.0.0"));

    let meta_data = serde_json::to_string_pretty(&meta).unwrap();
    meta_file.write(meta_data.as_bytes())?;

    out::success::template_created(&template_path.to_str().unwrap());

    Ok(())
  }

  /// Return list of all template names in this repository
  pub fn get_templates(&self) -> Vec<String> {
    let mut templates = Vec::<String>::new();

    for template in &self.templates {
      templates.push(utils::lowercase(&template.name));
    }

    return templates;
  }

  /// Return template with given name
  pub fn get_template_by_name(
    &self,
    name: &str,
  ) -> Result<&template::Template, RepositoryError> {
    for template in &self.templates {
      if template.name == *name {
        return Ok(template);
      }
    }

    return Err(RepositoryError::TemplateNotFound);
  }

  fn ensure_repository_dir(&self, config: &Config) -> Result<(), Error> {
    let mut directory = PathBuf::from(&config.template_dir);
    directory.push(&utils::lowercase(&self.config.name));

    if !directory.exists() {
      match fs::create_dir(&directory) {
        Ok(_) => (),
        Err(error) => return Err(error),
      }
    }

    // Initialize git repository if enabled
    if !self.config.git_options.enabled {
      return Ok(());
    }

    // check if directory is already a git repository
    let already_initialized = match git2::Repository::open(&directory) {
      Ok(_) => true,
      Err(_error) => false,
    };

    // initialize git
    if !already_initialized {
      match git::init(&directory, &self.config.git_options.url.clone().unwrap()) {
        Ok(()) => (),
        Err(error) => {
          log::error!("{}", error);
          match error {
            git::GitError::InitError => (),
            git::GitError::AddRemoteError => (),
          };
        },
      };
    }

    match git::update(&directory, &self.config.git_options) {
      Ok(()) => (),
      Err(_e) => out::error::update_templates(),
    }

    Ok(())
  }

  fn load_templates(&mut self) {
    let mut templates = Vec::<template::Template>::new();

    // check if folder exists
    match fs::read_dir(&self.directory) {
      Ok(fc) => fc,
      Err(_error) => return,
    };

    // Loop at all entries in templates directory
    for entry in fs::read_dir(&self.directory).unwrap() {
      let entry = &entry.unwrap();
      // check if entry is file, if yes skip entry
      if !entry.path().is_dir() {
        continue;
      }

      let path = entry.path().to_string_lossy().into_owned();
      let meta = match meta::load_meta(&path) {
        Ok(meta) => meta,
        Err(error) => {
          log::error!("{}", error);
          continue;
        }
      };

      // If type is None or unqual template skip entry
      if meta.kind.is_none() || meta.kind != Some(String::from("template")) {
        continue;
      }

      let template = match template::Template::new(&entry) {
        Ok(template) => template,
        Err(_error) => continue,
      };

      templates.push(template);
    }
    self.templates = templates;
  }
}
