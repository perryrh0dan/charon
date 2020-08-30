use std::io::{Error, ErrorKind};

use dialoguer::{theme::ColorfulTheme, Input, Select, Password};

pub fn text(text: &str, allow_empty: bool ) -> Option<String> {
  match Input::<String>::with_theme(&ColorfulTheme::default())
    .with_prompt(text)
    .allow_empty(allow_empty)
    .interact()
  {
    Ok(value) => Some(value),
    Err(_error) => return None,
  }
}

pub fn confirm(text: &str) -> bool {
  let mut question = text.to_owned();
  question.push_str(" [Y/n]");

  match Input::<String>::with_theme(&ColorfulTheme::default())
    .with_prompt(&question)
    .allow_empty(false)
    .interact()
  {
    Ok(value) => value == "Y" || value == "y",
    Err(_error) => false,
  }
}

pub fn password(text: &str) -> Result<String, Error> {
  return Password::with_theme(&ColorfulTheme::default())
    .with_prompt(text)
    .interact();
}

pub fn select(name: &str, options: &Vec<String>) -> Result<String, Error> {
  if options.len() == 0 {
    return Err(Error::from(ErrorKind::InvalidData))
  };

  // TODO check
  // if options.len() == 1 {
  //    return Ok(options[0].to_owned());
  // }

  let selection = match Select::with_theme(&ColorfulTheme::default())
    .with_prompt(String::from("Select a ") + name)
    .default(0)
    .items(options)
    .interact()
  {
    Ok(selection) => selection,
    Err(error) => return Err(error),
  };
  let result = String::from(&options[selection]);
  return Ok(result);
}
