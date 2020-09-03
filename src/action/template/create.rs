// ask for private or public template
// if public try to push
use log;
use std::io::ErrorKind;

use crate::cli::input;
use crate::config::{Config};
use crate::out;
use crate::repository::{Repository, RepositoryError};
use crate::utils;

use clap::ArgMatches;

pub fn create(config: &mut Config, args: &ArgMatches) {
    let repository_name = args.value_of("repository");
    let template_name = args.value_of("template");

    // Get repository name from user input
    let repository_name = if repository_name.is_none() {
      let repositories = config.get_repositories();
      match input::select("repository", &repositories) {
        Ok(value) => value,
        Err(error) => match error.kind() {
          ErrorKind::InvalidData => {
            out::error::no_repositories();
            return;
          },
          _ => std::process::exit(130),
        },
      }
    } else {
      utils::lowercase(repository_name.unwrap())
    };

  // Load repository
  let mut repository = match Repository::new(config, &repository_name) {
    Ok(repository) => repository,
    Err(error) => return match error {
        RepositoryError::NotFound => out::error::repository_not_found(&repository_name),
        _ => out::error::unknown(),
    },
  };

  match repository.init() {
    Ok(_) => (),
    Err(_) => (),
  };

  // Get template name from user input
  let template_name = if template_name.is_none() {
      match input::text("template name", false) {
          Some(value) => value,
          None => return,
      }
  } else {
      String::from(template_name.unwrap())
  };

  // validate name
  let templates = repository.get_templates();
  if templates.contains(&template_name) {
      // TODO error
      return;
  }

  match repository.create_template(&template_name) {
      Ok(()) => (),
      Err(error) => {
          log::error!("{}", error);
      }
  };
}
