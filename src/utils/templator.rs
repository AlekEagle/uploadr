use serde_json_path::JsonPath;
use crate::utils::config::Config;
use crate::utils::file::File;

const DIRECTIVE_START: char = '{';
const DIRECTIVE_END: char = '}';
const DIRECTIVE_PARAMETER_SEPARATOR: char = ':';
const PARAMETER_SEPARATOR: char = ';';
const SPECIAL_ESCAPE: char = '\\';

#[derive(PartialEq)]
enum IteratorState {
  NoDirective,
  Directive,
}

#[derive(PartialEq)]
enum DirectiveState {
  ReadDirective,
  ReadParameters,
}

pub struct Templator<'a> {
  pub config: &'a Config,
  pub file: Option<&'a File>,
  // TODO: Add Request and Response data for templating.
}

impl<'a> Templator<'a> {
  pub fn new(config: &'a Config) -> Self {
    return Templator { config, file: None };
  }

  pub fn set_file(&mut self, file: &'a File) -> () {
    self.file = Some(file);
  }

  pub fn format(&mut self, template: &str) -> String {
    return self.directive_iterator(template);
  }

  fn directive_iterator(&mut self, template: &str) -> String {
    let mut result = String::new();
    let mut i_state = IteratorState::NoDirective;
    let mut directive = String::new();
    let mut parameters: Vec<String> = Vec::new();
    let mut d_state = DirectiveState::ReadDirective;
    let mut escape_next = false;
    let mut directive_finished = false;
    let mut sub_directive_level = 0;

    for char in template.chars() {
      if char == SPECIAL_ESCAPE {
        if escape_next {
          result.push(char);
          escape_next = false;
        } else {
          escape_next = true;
        }
      }
      match i_state {
        // If we're not in a directive...
        IteratorState::NoDirective => {
          // If a directive end is encountered...
          // and we're not escaping...
          if (char == DIRECTIVE_END) && !escape_next {
            // Unmatched directive end.
            panic!("Unmatched directive end.");
          }
          // and we encounter a directive start...
          // and we're not escaping...
          if (char == DIRECTIVE_START) && !escape_next {
            // Switch the iterator state to possible directive.
            i_state = IteratorState::Directive;
          } else {
            // Otherwise, just add the character to the result.
            result.push(char);
          }
        }
        // If we're in a directive...
        IteratorState::Directive => {
          // Start handling the directive.
          match d_state {
            // If we're reading the directive...
            DirectiveState::ReadDirective => {
              // Handle the character.
              match char {
                // If we encounter a directive start...
                // and we're not escaping...
                DIRECTIVE_START if !escape_next => {
                  panic!("Sub-directives in the directive name are not allowed.");
                }
                // If we encounter a directive end...
                // and we're not escaping...
                DIRECTIVE_END if !escape_next => {
                  // We're done reading the directive.
                  directive_finished = true;
                }
                // If we encounter a directive parameter separator...
                // and we're not escaping...
                DIRECTIVE_PARAMETER_SEPARATOR if !escape_next => {
                  // Switch the directive state to read parameters.
                  d_state = DirectiveState::ReadParameters;
                  // Push an empty string to the parameters to prevent unwrapping a None.
                  parameters.push(String::new());
                }
                // If we encounter a parameter separator...
                // and we're not escaping...
                PARAMETER_SEPARATOR if !escape_next => {
                  panic!("Parameter separators are not allowed in the directive name.");
                }
                // Everything else...
                _ => {
                  // Add the character to the directive name.
                  directive.push(char);
                }
              }
            }
            // If we're reading the parameters...
            DirectiveState::ReadParameters => {
              // Handle the character.
              match char {
                // If we encounter a directive start...
                // and we're not escaping...
                DIRECTIVE_START if !escape_next => {
                  // We're stepping into a sub-directive.
                  // Add the character to the parameters.
                  parameters.last_mut().unwrap().push(char);
                  sub_directive_level += 1;
                }
                // If we encounter a directive end...
                // and we're not escaping...
                DIRECTIVE_END if !escape_next => {
                  // If we're in a sub-directive...
                  if sub_directive_level > 0 {
                    // We're exiting a sub-directive.
                    // Add the character to the parameters.
                    parameters.last_mut().unwrap().push(char);
                    sub_directive_level -= 1;
                  } else {
                    // We're done reading the parameters.
                    directive_finished = true;
                  }
                }
                // If we encounter a directive parameter separator...
                // and we're not escaping...
                DIRECTIVE_PARAMETER_SEPARATOR if !escape_next => {
                  // If we're in a sub-directive...
                  if sub_directive_level > 0 {
                    // Add the character to the parameters.
                    parameters.last_mut().unwrap().push(char);
                  } else {
                    // Switch the directive state to read parameters.
                    d_state = DirectiveState::ReadParameters;
                  }
                }
                // If we encounter a parameter separator...
                // and we're not escaping...
                PARAMETER_SEPARATOR if !escape_next => {
                  // If we're in a sub-directive...
                  if sub_directive_level > 0 {
                    // Add the character to the parameters.
                    parameters.last_mut().unwrap().push(char);
                  } else {
                    // Add the parameters to the parameters.
                    parameters.push(String::new());
                  }
                }
                // Everything else...
                _ => {
                  // Add the character to the parameters.
                  parameters.last_mut().unwrap().push(char);
                }
              
              }
            }
          }
        }
      }
      if directive_finished {
        result.push_str(self.handle_directive(&directive, &parameters).as_str());
        // Reset the directive.
        directive = String::new();
        // Reset the parameters.
        parameters = Vec::new();
        // Reset the directive state.
        d_state = DirectiveState::ReadDirective;
        // Reset the directive finished flag.
        directive_finished = false;
        // Switch the iterator state to not in a directive.
        i_state = IteratorState::NoDirective;
      }
      // Reset the escape_next flag.
      escape_next = false;
    }

    // If the directive was never finished...
    if i_state != IteratorState::NoDirective {
      panic!("Unmatched directive start.");
    }

    return result;
  }
  
  fn handle_directive(&mut self, directive: &str, parameters: &Vec<String>) -> String {
    // trim all whitespace from the directive name.
    let directive = directive.trim();
    // If the directive is empty...
    if directive.is_empty() {
      // Panic.
      panic!("Empty directive.");
    }
    // Trim all whitespace from the parameters.
    let parameters: Vec<String> = parameters.iter().map(|parameter| self.directive_iterator(&parameter.trim().to_string())).collect();
    let mut result = String::new();
    match directive {
      "config" => {
        let config = &self.config.data.to_value();
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid JSON path.");
        let value = path.query(config).exactly_one();
        match value {
          Ok(value) => {
            result.push_str(value.as_str().unwrap());
          }
          Err(_) => {
            panic!("Invalid JSON path.");
          }
        }
      }
      "uploader" => {
        let config = &self.config.uploader.to_value();
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid JSON path.");
        let value = path.query(config).exactly_one();
        match value {
          Ok(value) => {
            result.push_str(value.as_str().unwrap());
          }
          Err(_) => {
            panic!("Invalid JSON path.");
          }
        }
      }
      "env" => {
        result.push_str(std::env::var(parameters.first().unwrap()).expect("Invalid environment variable.").as_str());
      }
      _ => panic!("Unknown directive: {}", directive),
    }
    return result;
  }
}