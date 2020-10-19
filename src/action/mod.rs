mod config;
mod init;
mod repository;
mod template;
mod update;

use crate::cli::input;
use crate::config::Config;
use crate::error::RunError;
use crate::repository::Repository;
use crate::repository::custom_repository::CustomRepository;

pub struct Action {
  config: Config,
}

impl Action {
  pub fn new(config: Config) -> Action {
    let act = Action { config: config };

    return act;
  }

  /// Validat given repository name or open a new selection
  fn get_repository(&self, repository_name: Option<&str>) -> Result<Box<dyn Repository>, RunError> {
    // Get repository name from user input
    let repository_name = if repository_name.is_none() {
      let repositories = self.config.get_repositories();
      input::select("repository", &repositories)?
    } else {
      String::from(repository_name.unwrap())
    };

    // Load repository
    let repository = if repository_name == "template" {
      CustomRepository::new(&self.config, &repository_name)?
    } else {
      CustomRepository::new(&self.config, &repository_name)?
    };

    Ok(Box::new(repository))
  }
}
