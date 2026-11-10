use crate::utils::config::Config;
use crate::utils::curlyfries::{CurlyFry, Response};
use crate::utils::file::File;
use anyhow::{anyhow, bail, Context, Result};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::Value;
use serde_json_path::JsonPath;
use std::iter::Peekable;
use std::str::Chars;
use uuid::Uuid;

const DIRECTIVE_START: char = '{';
const DIRECTIVE_END: char = '}';
const DIRECTIVE_PARAMETER_SEPARATOR: char = ':';
const PARAMETER_SEPARATOR: char = ';';
const SPECIAL_ESCAPE: char = '\\';

#[derive(PartialEq)]
enum DirectiveState {
  ReadDirective,
  ReadParameters,
}

pub struct Templator<'a> {
  pub config: &'a Config,
  pub file: Option<&'a File>,
  pub curlyfry: Option<&'a CurlyFry>,
}

/// Only these characters can be escaped, a backslash before anything else is kept as-is. (So regexes like `\d` work)
fn is_special(char: char) -> bool {
  matches!(
    char,
    DIRECTIVE_START
      | DIRECTIVE_END
      | DIRECTIVE_PARAMETER_SEPARATOR
      | PARAMETER_SEPARATOR
      | SPECIAL_ESCAPE
  )
}

/// Handle a backslash, the escaped character (or the backslash itself) goes into the current buffer.
fn push_escaped(buffer: &mut String, chars: &mut Peekable<Chars>) {
  match chars.peek() {
    Some(&next) if is_special(next) => {
      buffer.push(next);
      chars.next();
    }
    // Not an escape sequence (or a trailing backslash), keep the backslash literally.
    _ => buffer.push(SPECIAL_ESCAPE),
  }
}

/// Get a parameter, or error if it's missing.
fn parameter<'p>(parameters: &'p [String], index: usize, directive: &str) -> Result<&'p str> {
  parameters.get(index).map(String::as_str).ok_or_else(|| {
    anyhow!(
      "The {} directive is missing parameter {}.",
      directive,
      index + 1
    )
  })
}

/// Strings are used as-is (no JSON quotes), everything else is JSON encoded.
fn value_to_string(value: &Value) -> String {
  match value {
    Value::String(s) => s.clone(),
    other => other.to_string(),
  }
}

/// Query a JSON value with a JSON path that must match exactly one value.
fn query_one(value: &Value, path: &str) -> Result<String> {
  let json_path =
    JsonPath::parse(path).with_context(|| format!("Invalid JSON path \"{}\"", path))?;
  let value = json_path.query(value).exactly_one().map_err(|err| {
    anyhow!(
      "The JSON path \"{}\" did not match exactly one value: {}",
      path,
      err
    )
  })?;
  Ok(value_to_string(value))
}

/// Match a regex, returns the first capture group, or the whole match if there are no groups.
fn regex_match(pattern: &str, body: &str) -> Result<String> {
  let regex =
    regex::Regex::new(pattern).with_context(|| format!("Invalid regex \"{}\"", pattern))?;
  let captures = regex
    .captures(body)
    .ok_or_else(|| anyhow!("The regex \"{}\" did not match the response body.", pattern))?;
  Ok(
    captures
      .get(1)
      .or_else(|| captures.get(0))
      .map_or_else(String::new, |m| m.as_str().to_string()),
  )
}

fn parse_json(body: &str) -> Result<Value> {
  serde_json::from_str(body).context("The response body is not valid JSON")
}

fn parse_xml(body: &str) -> Result<Value> {
  serde_xml_rs::from_str(body).context("The response body is not valid XML")
}

impl<'a> Templator<'a> {
  pub fn new(config: &'a Config) -> Self {
    Templator {
      config,
      file: None,
      curlyfry: None,
    }
  }

  pub fn set_file(&mut self, file: &'a File) {
    self.file = Some(file);
  }

  pub fn set_curlyfry(&mut self, curlyfry: &'a CurlyFry) {
    self.curlyfry = Some(curlyfry);
  }

  pub fn format(&mut self, template: &str) -> Result<String> {
    self
      .directive_iterator(template)
      .with_context(|| format!("Failed to format template \"{}\"", template))
  }

  fn directive_iterator(&mut self, template: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(char) = chars.next() {
      match char {
        SPECIAL_ESCAPE => push_escaped(&mut result, &mut chars),
        // Read the directive and add its value to the result.
        DIRECTIVE_START => result.push_str(&self.read_directive(&mut chars)?),
        DIRECTIVE_END => bail!("Unmatched directive end."),
        // Otherwise, just add the character to the result.
        _ => result.push(char),
      }
    }

    Ok(result)
  }

  /// Read a directive (the opening brace has already been consumed) up to its matching end, and evaluate it.
  /// Nested directives in parameters are evaluated while reading, so their output is never templated again.
  fn read_directive(&mut self, chars: &mut Peekable<Chars>) -> Result<String> {
    let mut directive = String::new();
    let mut parameters: Vec<String> = Vec::new();
    let mut parameter = String::new();
    let mut d_state = DirectiveState::ReadDirective;

    loop {
      // If the directive was never finished...
      let Some(char) = chars.next() else {
        bail!("Unmatched directive start.");
      };
      match d_state {
        // If we're reading the directive...
        DirectiveState::ReadDirective => match char {
          SPECIAL_ESCAPE => push_escaped(&mut directive, chars),
          DIRECTIVE_START => bail!("Sub-directives in the directive name are not allowed."),
          // We're done reading the directive.
          DIRECTIVE_END => break,
          // Switch the directive state to read parameters.
          DIRECTIVE_PARAMETER_SEPARATOR => d_state = DirectiveState::ReadParameters,
          PARAMETER_SEPARATOR => {
            bail!("Parameter separators are not allowed in the directive name.")
          }
          // Add the character to the directive name.
          _ => directive.push(char),
        },
        // If we're reading the parameters...
        DirectiveState::ReadParameters => match char {
          SPECIAL_ESCAPE => push_escaped(&mut parameter, chars),
          // We're stepping into a sub-directive, add its value to the parameter.
          DIRECTIVE_START => parameter.push_str(&self.read_directive(chars)?),
          // We're done reading the parameters.
          DIRECTIVE_END => break,
          // Start the next parameter.
          PARAMETER_SEPARATOR => parameters.push(std::mem::take(&mut parameter)),
          // Everything else (including ':') is part of the parameter.
          _ => parameter.push(char),
        },
      }
    }
    if d_state == DirectiveState::ReadParameters {
      parameters.push(parameter);
    }

    self.handle_directive(&directive, &parameters)
  }

  fn response(&self, directive: &str) -> Result<&'a Response> {
    self
      .curlyfry
      .and_then(|curlyfry| curlyfry.response.as_ref())
      .ok_or_else(|| {
        anyhow!(
          "Use of {} directive before response is available.",
          directive
        )
      })
  }

  fn handle_directive(&mut self, directive: &str, parameters: &[String]) -> Result<String> {
    // trim all whitespace from the directive name.
    let directive = directive.trim();
    // If the directive is empty...
    if directive.is_empty() {
      bail!("Empty directive.");
    }
    // Parameters are not trimmed (whitespace can matter in choices, alphabets and regexes),
    // names, numbers and paths are trimmed where they're used.
    let param = |index: usize| parameter(parameters, index, directive);
    match directive {
      "config" => query_one(&self.config.data.to_value()?, param(0)?.trim()),
      "uploader" => query_one(&self.config.uploader.to_value()?, param(0)?.trim()),
      "env" => {
        let name = param(0)?.trim();
        match (std::env::var(name), parameters.get(1)) {
          (Ok(value), _) => Ok(value),
          // Optional default value.
          (Err(_), Some(default)) => Ok(default.clone()),
          (Err(err), None) => Err(anyhow!(
            "Invalid environment variable \"{}\": {}",
            name,
            err
          )),
        }
      }
      "random" => {
        let mut rng = rand::thread_rng();
        // Which random function to use.
        match param(0)?.trim() {
          "int" => {
            let min = parameters
              .get(1)
              .and_then(|min| min.trim().parse::<i64>().ok())
              .unwrap_or(1);
            let max = parameters
              .get(2)
              .and_then(|max| max.trim().parse::<i64>().ok())
              .unwrap_or(100);
            if min > max {
              bail!("random:int min ({}) is greater than max ({}).", min, max);
            }
            Ok(rng.gen_range(min..=max).to_string())
          }
          "float" => {
            let min = parameters
              .get(1)
              .and_then(|min| min.trim().parse::<f64>().ok())
              .unwrap_or(1.0);
            let max = parameters
              .get(2)
              .and_then(|max| max.trim().parse::<f64>().ok())
              .unwrap_or(100.0);
            if !min.is_finite() || !max.is_finite() || min > max {
              bail!(
                "random:float needs finite numbers with min ({}) <= max ({}).",
                min,
                max
              );
            }
            if min == max {
              Ok(min.to_string())
            } else {
              Ok(rng.gen_range(min..max).to_string())
            }
          }
          "choice" => {
            // the rest of the parameters are the choices.
            parameters[1..]
              .choose(&mut rng)
              .cloned()
              .ok_or_else(|| anyhow!("random:choice needs at least one choice."))
          }
          "uuid" => Ok(Uuid::new_v4().to_string()),
          "string" => {
            let length = parameters
              .get(1)
              .and_then(|length| length.trim().parse::<usize>().ok())
              .unwrap_or(10);
            let alphabet: Vec<char> = match parameters.get(2) {
              Some(alphabet) => alphabet.chars().collect(),
              None => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                .chars()
                .collect(),
            };
            if alphabet.is_empty() {
              bail!("random:string alphabet is empty.");
            }
            Ok(
              (0..length)
                .filter_map(|_| alphabet.choose(&mut rng))
                .collect(),
            )
          }
          other => bail!("Unknown random function \"{}\".", other),
        }
      }
      "response" => {
        let response = self.response(directive)?;
        match param(0)?.trim() {
          "status_code" => Ok(response.status_code.to_string()),
          "url" => Ok(response.url.clone()),
          "headers" => {
            let name = param(1)?.trim().to_lowercase();
            response
              .headers
              .get(&name)
              .cloned()
              .ok_or_else(|| anyhow!("Header \"{}\" not found.", name))
          }
          "body" => match param(1)?.trim() {
            "json" => query_one(&parse_json(&response.body)?, param(2)?.trim()),
            "xml" => query_one(&parse_xml(&response.body)?, param(2)?.trim()),
            "regex" => regex_match(param(2)?, &response.body),
            other => bail!("Unknown body type \"{}\".", other),
          },
          other => bail!("Unknown response function \"{}\".", other),
        }
      }
      // Alias for response:body;json
      "json" => query_one(
        &parse_json(&self.response(directive)?.body)?,
        param(0)?.trim(),
      ),
      // Alias for response:body;xml
      "xml" => query_one(
        &parse_xml(&self.response(directive)?.body)?,
        param(0)?.trim(),
      ),
      // Alias for response:body;regex
      "regex" => regex_match(param(0)?, &self.response(directive)?.body),
      "file" => {
        let file = self
          .file
          .ok_or_else(|| anyhow!("Use of file directive before file is available."))?;
        match param(0)?.trim() {
          "name" => Ok(file.name.clone()),
          "ext" => Ok(file.ext.clone()),
          "mime" => Ok(file.mime.clone()),
          "size" => Ok(file.buffer.len().to_string()),
          other => bail!("Unknown file function \"{}\".", other),
        }
      }
      _ => bail!("Unknown directive: {}", directive),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::config::{ConfigData, UploaderData};
  use std::collections::HashMap;

  fn config() -> Config {
    let data = ConfigData {
      default_uploader: "test:uploader".to_owned(),
      ..ConfigData::default()
    };
    Config {
      data,
      uploader: UploaderData::default(),
    }
  }

  fn curlyfry(body: &str) -> CurlyFry {
    let mut curlyfry = CurlyFry::new().unwrap();
    let mut headers = HashMap::new();
    headers.insert("x-delete-token".to_owned(), "abc123".to_owned());
    curlyfry.response = Some(Response {
      status_code: 201,
      url: "https://example.com/final".to_owned(),
      headers,
      body: body.to_owned(),
    });
    curlyfry
  }

  fn format(template: &str) -> Result<String> {
    let config = config();
    let mut templator = Templator::new(&config);
    templator.format(template)
  }

  fn format_response(template: &str, body: &str) -> Result<String> {
    let config = config();
    let curlyfry = curlyfry(body);
    let mut templator = Templator::new(&config);
    templator.set_curlyfry(&curlyfry);
    templator.format(template)
  }

  #[test]
  fn plain_text_is_unchanged() {
    assert_eq!(
      format("https://example.com/upload").unwrap(),
      "https://example.com/upload"
    );
  }

  #[test]
  fn escapes() {
    // An escaped backslash is emitted once.
    assert_eq!(format(r"a\\b").unwrap(), r"a\b");
    // Escaped braces are literal.
    assert_eq!(format(r"\{not a directive\}").unwrap(), "{not a directive}");
    // A backslash before a non-special character is kept.
    assert_eq!(format(r"C:\dir").unwrap(), r"C:\dir");
    // A trailing backslash is kept.
    assert_eq!(format(r"trailing\").unwrap(), r"trailing\");
    // Escapes inside parameters go into the parameter, not the result.
    assert_eq!(
      format(r"[{env:UPLOADR_TEST_MISSING;a\;b\:c\}\\}]").unwrap(),
      r"[a;b:c}\]"
    );
  }

  #[test]
  fn colons_in_parameters_are_kept() {
    assert_eq!(
      format("{env:UPLOADR_TEST_MISSING;https://example.com:8080}").unwrap(),
      "https://example.com:8080"
    );
  }

  #[test]
  fn env_directive() {
    std::env::set_var("UPLOADR_TEST_ENV", "value");
    assert_eq!(format("{env:UPLOADR_TEST_ENV}").unwrap(), "value");
    assert_eq!(format("{ env: UPLOADR_TEST_ENV }").unwrap(), "value");
    assert!(format("{env:UPLOADR_TEST_DEFINITELY_MISSING}").is_err());
  }

  #[test]
  fn nested_directives() {
    std::env::set_var("UPLOADR_TEST_NESTED", "UPLOADR_TEST_INNER");
    std::env::set_var("UPLOADR_TEST_INNER", "inner value");
    assert_eq!(
      format("{env:{env:UPLOADR_TEST_NESTED}}").unwrap(),
      "inner value"
    );
    // Output of nested directives is not templated again.
    std::env::set_var("UPLOADR_TEST_BRACES", "{env:UPLOADR_TEST_INNER}");
    assert_eq!(
      format("{env:UPLOADR_TEST_MISSING;{env:UPLOADR_TEST_BRACES}}").unwrap(),
      "{env:UPLOADR_TEST_INNER}"
    );
  }

  #[test]
  fn errors_instead_of_panics() {
    assert!(format("{").is_err());
    assert!(format("}").is_err());
    assert!(format("{}").is_err());
    assert!(format("{nope}").is_err());
    assert!(format("{random:nope}").is_err());
    assert!(format("{random:int;10;1}").is_err());
    assert!(format("{random:choice}").is_err());
    assert!(format("{random:string;5;}").is_err());
    assert!(format("{json:$.a}").is_err());
    assert!(format("{file:name}").is_err());
    assert!(format("{config:$.nope}").is_err());
    assert!(format(r"{regex:\d{3}}").is_err());
  }

  #[test]
  fn config_strings_are_unquoted() {
    assert_eq!(
      format("{config:$.default_uploader}").unwrap(),
      "test:uploader"
    );
    assert_eq!(format("{config:$.archive.max}").unwrap(), "100");
    assert_eq!(format("{config:$.notification.enabled}").unwrap(), "true");
    assert_eq!(format("{uploader:$.request.method}").unwrap(), "POST");
    assert_eq!(
      format("{uploader:$.request.url}").unwrap(),
      "https://api.imgur.com/3/image"
    );
  }

  #[test]
  fn random_ranges() {
    for _ in 0..200 {
      let value: i64 = format("{random:int;1;3}").unwrap().parse().unwrap();
      assert!((1..=3).contains(&value));
      let value: f64 = format("{random:float;1.5;2.5}").unwrap().parse().unwrap();
      assert!((1.5..2.5).contains(&value));
    }
    assert_eq!(format("{random:int;7;7}").unwrap(), "7");
    assert_eq!(format("{random:int; -2 ; -2 }").unwrap(), "-2");
    assert_eq!(format("{random:float;2.5;2.5}").unwrap(), "2.5");
    // Both ends of an inclusive int range are reachable.
    let seen: std::collections::HashSet<String> = (0..500)
      .map(|_| format("{random:int;0;1}").unwrap())
      .collect();
    assert_eq!(seen.len(), 2);
  }

  #[test]
  fn random_choice_and_string() {
    for _ in 0..50 {
      let choice = format("{random:choice;a;b;c}").unwrap();
      assert!(["a", "b", "c"].contains(&choice.as_str()));
    }
    assert_eq!(format("{random:choice; spaced }").unwrap(), " spaced ");
    let string = format("{random:string;8;äö}").unwrap();
    assert_eq!(string.chars().count(), 8);
    assert!(string.chars().all(|c| c == 'ä' || c == 'ö'));
    assert_eq!(format("{random:string}").unwrap().len(), 10);
    assert_eq!(format("{random:uuid}").unwrap().len(), 36);
  }

  #[test]
  fn json_response() {
    let body = r#"{"data":{"link":"https://i.example.com/a.png","id":42}}"#;
    assert_eq!(
      format_response("{json:$.data.link}", body).unwrap(),
      "https://i.example.com/a.png"
    );
    assert_eq!(format_response("{json:$.data.id}", body).unwrap(), "42");
    assert_eq!(
      format_response("{response:body;json;$.data.link}", body).unwrap(),
      "https://i.example.com/a.png"
    );
    assert!(format_response("{json:$.data.missing}", body).is_err());
    assert!(format_response("{json:$.a}", "not json").is_err());
  }

  #[test]
  fn regex_response() {
    let body = "id=12345 url=https://example.com/f/12345";
    // First capture group.
    assert_eq!(format_response(r"{regex:id=(\d+)}", body).unwrap(), "12345");
    // No group, whole match.
    assert_eq!(
      format_response(r"{regex:https://\S+}", body).unwrap(),
      "https://example.com/f/12345"
    );
    // Escaped braces for quantifiers.
    assert_eq!(format_response(r"{regex:\d\{3\}}", body).unwrap(), "123");
    assert_eq!(
      format_response(r"{response:body;regex;url=(\S+)}", body).unwrap(),
      "https://example.com/f/12345"
    );
    // No match is an error.
    assert!(format_response(r"{regex:nothing(here)}", body).is_err());
    assert!(format_response(r"{regex:(}", body).is_err());
  }

  #[test]
  fn response_fields() {
    assert_eq!(
      format_response("{response:status_code}", "").unwrap(),
      "201"
    );
    assert_eq!(
      format_response("{response:url}", "").unwrap(),
      "https://example.com/final"
    );
    assert_eq!(
      format_response("{response:headers;X-Delete-Token}", "").unwrap(),
      "abc123"
    );
    assert!(format_response("{response:headers;x-missing}", "").is_err());
  }

  #[test]
  fn file_directives() {
    let config = config();
    let file = File::new(b"hello".to_vec(), "notes.md".to_owned());
    let mut templator = Templator::new(&config);
    templator.set_file(&file);
    assert_eq!(
      templator
        .format("{file:name} {file:ext} {file:mime} {file:size}")
        .unwrap(),
      "notes.md md text/plain 5"
    );
  }

  #[test]
  fn default_imgur_header() {
    let config = config();
    let mut templator = Templator::new(&config);
    let header = config
      .uploader
      .request
      .headers
      .get("Authorization")
      .unwrap();
    std::env::remove_var("UPLOADR_IMGUR_CLIENT_ID");
    assert_eq!(
      templator.format(header).unwrap(),
      "Client-ID eb13b71463957f7"
    );
  }
}
