use serde_json_path::JsonPath;
use crate::utils::config::Config;
use crate::utils::file::File;
use crate::utils::curlyfries::CurlyFry;
use rand;
use uuid::Uuid;


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
  pub file: Option<File>,
  pub curlyfry: Option<&'a CurlyFry>,
}

impl<'a> Templator<'a> {
  pub fn new(config: &'a Config) -> Self {
    Templator { config, file: None, curlyfry: None }
  }

  pub fn set_file(&mut self, file: File) {
    self.file = Some(file);
  }

  pub fn set_curlyfry(&mut self, curlyfry: &'a CurlyFry) {
    self.curlyfry = Some(curlyfry);
  }

  pub fn format(&mut self, template: &str) -> String {
    self.directive_iterator(template)
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

    result
  }
  
  fn handle_directive(&mut self, directive: &str, parameters: &[String]) -> String {
    // trim all whitespace from the directive name.
    let directive = directive.trim();
    // If the directive is empty...
    if directive.is_empty() {
      // Panic.
      panic!("Empty directive.");
    }
    // Trim all whitespace from the parameters.
    let parameters: Vec<String> = parameters.iter().map(|parameter| self.directive_iterator(parameter.trim())).collect();
    match directive {
      "config" => {
        let config = &self.config.data.to_value();
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid JSON path");
        let value = path.query(config).exactly_one();
        match value {
          Ok(value) => {
            value.to_string()
          }
          Err(_) => {
            panic!("Invalid JSON path.");
          }
        }
      }
      "uploader" => {
        let config = &self.config.uploader.to_value();
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid JSON path");
        let value = path.query(config).exactly_one();
        match value {
          Ok(value) => {
            value.to_string()
          }
          Err(_) => {
            panic!("Invalid JSON path.");
          }
        }
      }
      "env" => std::env::var(parameters.first().unwrap()).expect("Invalid environment variable"),
      "random" => {
        // Which random function to use.
        match parameters.first().unwrap().as_str() {
          "int" => {
            let min = parameters.get(1).unwrap().parse::<i64>().unwrap_or(1);
            let max = parameters.get(2).unwrap().parse::<i64>().unwrap_or(100);
            (rand::random::<i64>() % (max - min) + min).to_string()
          }
          "float" => {
            let min = parameters.get(1).unwrap().parse::<f64>().unwrap_or(1.0);
            let max = parameters.get(2).unwrap().parse::<f64>().unwrap_or(100.0);
            (rand::random::<f64>() % (max - min) + min).to_string()
          }
          "choice" => {
            // the rest of the parameters are the choices.
            let choices = &parameters[1..];
            choices[rand::random::<usize>() % choices.len()].clone()
          }
          "uuid" => {
            Uuid::new_v4().to_string()
          }
          "string" => {
            let length = parameters.get(1).unwrap().parse::<usize>().unwrap_or(10);
            let alphabet = {
              let default_alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
              parameters.get(2).unwrap_or(&default_alphabet).to_string()
            };
            let mut result = String::new();
            for _ in 0..length {
              result.push(alphabet.chars().nth(rand::random::<usize>() % alphabet.len()).unwrap());
            }
            result
          }
          _ => panic!("Unknown random function.")
        }
      }
      "response" => {
        let curlyfry = self.curlyfry.expect("Use of response directive before response is available.");
        match parameters.first().unwrap().as_str() {
          "status_code" => {
            curlyfry.response.as_ref().unwrap().status_code.to_string()
          }
          "url" => {
            curlyfry.response.as_ref().unwrap().url.clone()
          }
          "headers" => {
            let headers = &curlyfry.response.as_ref().unwrap().headers;
            headers.get(&parameters.get(1).unwrap().to_lowercase()).expect("Header not found").to_string()
          }
          "body" => {
            match parameters.get(1).unwrap().as_str() {
              "json" => {
                let json = serde_json::from_str(&curlyfry.response.as_ref().unwrap().body).expect("Invalid JSON");
                let path = JsonPath::parse(parameters.get(2).unwrap()).expect("Invalid JSON path");
                let value = path.query(&json).exactly_one();
                match value {
                  Ok(value) => {
                    value.as_str().unwrap().to_string()
                  }
                  Err(_) => {
                    panic!("Invalid JSON path.");
                  }
                }
              }
              "xml" => {
                let xml = serde_xml_rs::from_str(&curlyfry.response.as_ref().unwrap().body).expect("Invalid XML");
                let path = JsonPath::parse(parameters.get(2).unwrap()).expect("Invalid XML path");
                let value = path.query(&xml).exactly_one();
                match value {
                  Ok(value) => {
                    value.as_str().unwrap().to_string()
                  }
                  Err(_) => {
                    panic!("Invalid XML path.");
                  }
                }
              }
              "regex" => {
                let flag = parameters.get(2).unwrap().as_str();
                let regex = regex::Regex::new(parameters.get(3).unwrap()).expect("Invalid regex");
                let captures = regex.captures(&curlyfry.response.as_ref().unwrap().body).expect("No match");
                captures.get(1).unwrap().as_str().to_string()
              }
              _ => panic!("Unknown body type.")
            }
          }
          _ => panic!("Unknown response function.")
        }
      }
      // Alias for response:body;json
      "json" => {
        let curlyfry = self.curlyfry.expect("Use of json directive before response is available.");
        let json = serde_json::from_str(&curlyfry.response.as_ref().unwrap().body).expect("Invalid JSON");
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid JSON path");
        let value = path.query(&json).exactly_one();
        match value {
          Ok(value) => {
            value.as_str().unwrap().to_string()
          }
          Err(_) => {
            panic!("Invalid JSON path.");
          }
        }
      }
      // Alias for response:body;xml
      "xml" => {
        let curlyfry = self.curlyfry.expect("Use of xml directive before response is available.");
        let xml = serde_xml_rs::from_str(&curlyfry.response.as_ref().unwrap().body).expect("Invalid XML");
        let path = JsonPath::parse(parameters.first().unwrap()).expect("Invalid XML path");
        let value = path.query(&xml).exactly_one();
        match value {
          Ok(value) => {
            value.as_str().unwrap().to_string()
          }
          Err(_) => {
            panic!("Invalid XML path.");
          }
        }
      }
      // Alias for response:body;regex
      "regex" => {
        let curlyfry = self.curlyfry.expect("Use of regex directive before response is available.");
        let flag = parameters.get(1).unwrap().as_str();
        let regex = regex::Regex::new(parameters.get(2).unwrap()).expect("Invalid regex");
        let captures = regex.captures(&curlyfry.response.as_ref().unwrap().body).expect("No match");
        captures.get(1).unwrap().as_str().to_string()
      }
      "file" => {
        let file = self.file.as_ref().expect("Use of file directive before file is available.");
        match parameters.first().unwrap().as_str() {
          "name" => {
            file.name.clone()
          }
          "ext" => {
            file.ext.clone()
          }
          "mime" => {
            file.mime.clone()
          }
          "size" => {
            file.buffer.len().to_string()
          }
          _ => panic!("Unknown file function.")
        }
      }
      _ => panic!("Unknown directive: {}", directive),
    }
  }
}