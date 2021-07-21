use log;
use std::collections::HashMap;

mod helpers;

extern crate handlebars;
use handlebars::Handlebars;
extern crate serde;
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct Context {
  pub name: String,
  pub repository: String,
  pub username: String,
  pub email: String,
  pub values: HashMap<String, String>,
}

pub fn render(text: &str, content: &Context) -> String {
  // create the handlebars registry
  let mut handlebars = Handlebars::new();

  // register helper methods
  handlebars.register_helper("uppercase", Box::new(helpers::uppercase_helper));
  handlebars.register_helper("lowercase", Box::new(helpers::lowercase_helper));
  handlebars.register_helper("camelcase", Box::new(helpers::camelcase_helper));
  handlebars.register_helper("pascalcase", Box::new(helpers::pascalcase_helper));
  handlebars.register_helper("snakecase", Box::new(helpers::snakecase_helper));
  handlebars.register_helper("kebabcase", Box::new(helpers::kebabcase_helper));
  handlebars.register_helper("constantcase", Box::new(helpers::constantcase_helper));
  handlebars.register_helper("capitalcase", Box::new(helpers::capitalcase_helper));

  // create the render context with the provided variables
  let context = match handlebars::Context::wraps(content) {
    Ok(context) => context,
    Err(error) => {
      log::error!("Error creating render context: Error: {}", error);
      return text.to_owned();
    }
  };

  let escaped_text = text.replace(r"\", r"\\");

  // render the template
  let result = match handlebars.render_template_with_context(&escaped_text, &context) {
    Ok(result) => result,
    Err(error) => {
      log::error!("Error rendering template: Error: {}", error);
      text.to_owned()
    }
  };

  return result;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_render_default() -> Result<(), Box<dyn std::error::Error>> {
    let text = "this is {{name}} an small test to test the basic {{repository}} features of the placeholder logic";
    let content: Context = Context {
      name: String::from("Tmpo"),
      repository: String::from("https://github.com/perryrh0dan/tmpo"),
      username: String::from("perryrh0dan"),
      email: String::from("thomaspoehlmann96@googlemail.com"),
      values: HashMap::new(),
    };

    let result = render(text, &content);

    assert_eq!(result, "this is Tmpo an small test to test the basic https://github.com/perryrh0dan/tmpo features of the placeholder logic");

    Ok(())
  }

  #[test]
  fn test_render_values() -> Result<(), Box<dyn std::error::Error>> {
    let text =
      "lets add one custom value: {{ values.full_name }} or a second one {{ values.last_name }}";
    let mut values = HashMap::new();
    values.insert(String::from("full_name"), String::from("Thomas Pöhlmann"));
    values.insert(String::from("last_name"), String::from("Pöhlmann"));
    let content: Context = Context {
      name: String::from("Tmpo"),
      repository: String::from("https://github.com/perryrh0dan/tmpo"),
      username: String::from("perryrh0dan"),
      email: String::from("thomaspoehlmann96@googlemail.com"),
      values: values,
    };

    let result = render(text, &content);

    assert_eq!(
      result,
      "lets add one custom value: Thomas Pöhlmann or a second one Pöhlmann"
    );

    Ok(())
  }

  #[test]
  fn test_render_transformation_values() -> Result<(), Box<dyn std::error::Error>> {
    let text =
      "{{ camelcase values.full_name }},{{ constantcase values.full_name }},{{ kebabcase values.full_name }},{{ lowercase values.full_name }},{{ pascalcase values.full_name }},{{ snakecase values.full_name }},{{ uppercase values.full_name }},{{ capitalcase values.full_name }}";
    let mut values = HashMap::new();
    values.insert(String::from("full_name"), String::from("ThomasPöhlmann"));
    let content: Context = Context {
      name: String::from("Tmpo"),
      repository: String::from("https://github.com/perryrh0dan/tmpo"),
      username: String::from("perryrh0dan"),
      email: String::from("thomaspoehlmann96@googlemail.com"),
      values: values,
    };

    let result = render(text, &content);

    assert_eq!(
      result,
      "thomasPöhlmann,THOMAS_PÖHLMANN,thomas-pöhlmann,thomaspöhlmann,ThomasPöhlmann,thomas_pöhlmann,THOMASPÖHLMANN,Thomas Pöhlmann"
    );

    Ok(())
  }

  #[test]
  fn test_render_unknown() -> Result<(), Box<dyn std::error::Error>> {
    let text = "lets add one custom value: {{ values.full_name }} or a second one {{ values.last_name }} and an unknown: {{ values.first_name }}";
    let mut values = HashMap::new();
    values.insert(String::from("full_name"), String::from("Thomas Pöhlmann"));
    values.insert(String::from("last_name"), String::from("Pöhlmann"));
    let content: Context = Context {
      name: String::from("Tmpo"),
      repository: String::from("https://github.com/perryrh0dan/tmpo"),
      username: String::from("perryrh0dan"),
      email: String::from("thomaspoehlmann96@googlemail.com"),
      values: values,
    };

    let result = render(text, &content);

    assert_eq!(
      result,
      "lets add one custom value: Thomas Pöhlmann or a second one Pöhlmann and an unknown: "
    );

    Ok(())
  }

  #[test]
  fn test_render_path() -> Result<(), Box<dyn std::error::Error>> {
    let text = r"C:\test\test1234\{{name}}.graphql.ts";
    let mut values = HashMap::new();
    values.insert(String::from("name"), String::from("ProductView"));
    let content: Context = Context {
      name: String::from("Tmpo"),
      repository: String::from("https://github.com/perryrh0dan/tmpo"),
      username: String::from("perryrh0dan"),
      email: String::from("thomaspoehlmann96@googlemail.com"),
      values: values,
    };

    let result = render(text, &content);

    assert_eq!(
      result,
      r"C:\\test\\test1234\Tmpo.graphql.ts"
    );

    Ok(())
  }
}
