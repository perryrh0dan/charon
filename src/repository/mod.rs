use std::fs;
use std::io::Error;
use std::path::Path;

use crate::config::Config;
use crate::git;
use crate::renderer;

extern crate custom_error;
use custom_error::custom_error;

pub mod template;

pub struct Repository {
  pub directory: String,
  pub config: git::RepoOptions,
  pub templates: Vec<template::Template>,
}

custom_error! {pub RepositoryError
  InitializationError = "Unable to initialize repository",
  TemplateNotFound = "Unable to find template",
  LoadingErrors = "Unable to load templates",
}

impl Repository {
  pub fn new(config: &Config, url: &str) -> Result<Repository, RepositoryError> {
    // Create template dir if not exists
    let r = fs::create_dir_all(Path::new(&config.templates_dir));
    match r {
      Ok(fc) => fc,
      Err(_error) => return Err(RepositoryError::InitializationError),
    }

    let cfg = config.get_repository_config(url).unwrap();
    let repository_name = base64::encode_config(&cfg.url, base64::URL_SAFE);
    let repository_dir = String::from(&config.templates_dir) + "/" + &repository_name;

    let mut repository = Repository {
      directory: repository_dir,
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

  pub fn get_templates(&self) -> Vec<String> {
    let mut templates = Vec::<String>::new();

    for template in &self.templates {
      templates.push(String::from(&template.name));
    }

    return templates;
  }

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
    let repository_name = base64::encode_config(&self.config.url, base64::URL_SAFE);
    let repository_dir = String::from(&config.templates_dir) + "/" + &repository_name;
    let r = fs::create_dir_all(Path::new(&repository_dir));
    match r {
      Ok(fc) => fc,
      Err(error) => return Err(error),
    }

    // Initialize git repository if enabled
    if self.config.enabled {
      match git::init(&repository_dir, &self.config.url) {
        Ok(()) => (),
        Err(error) => match error {
          git::GitError::InitError => println!("Init Error"),
          git::GitError::AddRemoteError => println!("Add Remote Error"),
        },
      };
      match git::update(&repository_dir, &self.config) {
        Ok(()) => (),
        Err(_e) => renderer::errors::update_templates(),
      }
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
      let meta = template::meta::load_meta(&path).unwrap();

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

pub fn get_repositories(config: &Config) -> Vec<String> {
  let mut repositories = Vec::<String>::new();

  for entry in fs::read_dir(&config.templates_dir).unwrap() {
    let entry = &entry.unwrap();

    if !entry.path().is_dir() {
      continue;
    }

    let name_encoded = entry.file_name().into_string().unwrap_or_default();
    let name_decoded = String::from_utf8_lossy(&base64::decode(name_encoded).unwrap()).to_owned().to_string();

    repositories.push(name_decoded);
  }

  return repositories;
}
