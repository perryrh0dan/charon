use crate::config::Config;
use crate::repository;
use crate::renderer;


pub fn list(config: &Config) {
  let repository = match repository::Repository::new(config, "") {
    Ok(repository) => repository,
    Err(_error) => return,
  };

  let mut names = Vec::new();
  for template in repository.templates {
    names.push(template.name);
  }

  renderer::list_templates(&names);
}
