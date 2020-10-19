use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::{PathBuf};

use crate::config::{Config, RepositoryOptions};
use crate::context::Context;
use crate::error::RunError;
use crate::git;
use crate::template;

use crate::repository::{Repository, CopyOptions};
use crate::utils;

#[derive(Debug)]
pub struct DefaultRepository {
  pub directory: PathBuf,
  pub templates: Vec<template::Template>,
}

impl Repository for DefaultRepository {
  fn get_config(&self) -> RepositoryOptions {
    return RepositoryOptions{
      name: String::from("Templates"),
      description: Some(String::from("Mono repository templates")),
      git_options: git::Options::new(),
    };
  }

  fn copy_template(&self, ctx: &Context, opts: &CopyOptions) -> Result<(), RunError> {
    let template = self.get_template_by_name(&opts.template_name)?;

    // Initialize template
    template.init(ctx, &opts.target, &opts.render_context)?;

    // Create info file
    template.create_info(&opts.target)?;

    Ok(())
  }

  fn get_template_values(&self, template_name: &str) -> Result<HashSet<String>, RunError> {
    let template = self.get_template_by_name(&template_name)?;

    let mut values = HashSet::new();
    values.extend(template.meta.get_values());

    Ok(values)
  }

  /// Return list of all template names in this repository
  fn get_template_names(&self) -> Vec<String> {
    let mut templates = Vec::<String>::new();

    for template in &self.templates {
      templates.push(utils::lowercase(&template.name));
    }

    return templates;
  }

  /// Return template with given name
  fn get_template_by_name(&self, name: &str) -> Result<&template::Template, RunError> {
    for template in &self.templates {
      if template.name == *name {
        return Ok(template);
      }
    }

    return Err(RunError::Template(String::from("Not found")));
  }
}

impl DefaultRepository {
  pub fn new(config: &Config, name: &str) -> Result<DefaultRepository, RunError> {
    log::info!("Loading repository: {}", name);

    let directory = config.templates_dir;

    let mut repository = DefaultRepository {
      directory: directory,
      templates: Vec::<template::Template>::new(),
    };

    // Load templates
    repository.load_templates()?;

    return Ok(repository);
  }

  fn load_templates(&self) -> Result<(), RunError> {
    self.templates = Vec::<template::Template>::new();

    // check if folder exists
    match fs::read_dir(&self.directory) {
      Ok(fc) => fc,
      Err(error) => return Err(RunError::IO(error)),
    };

    // Loop at all entries in repository directory
    for entry in fs::read_dir(&self.directory).unwrap() {
      let entry = &entry.unwrap();
      // check if entry is file, if yes skip entry
      if !entry.path().is_dir() {
        continue;
      }

      let meta = match meta::load(&entry.path()) {
        Ok(meta) => meta,
        Err(error) => {
          log::error!("{}", error);
          continue;
        }
      };

      // Skip if type is not template
      if meta.kind != meta::Type::TEMPLATE {
        continue;
      }

      let template = match template::Template::new(&entry.path()) {
        Ok(template) => template,
        Err(error) => {
          log::error!("{}", error);
          continue;
        }
      };

      self.templates.push(template);
    }
  }
}
