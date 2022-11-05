#include "syntactic.hpp"
#include <base64.h>
#include <iostream>
#include <jsoncons/json.hpp>
#include <jsoncons_ext/jsonpath/jsonpath.hpp>
#include <list>
#include <pugixml.hpp>
#include <string>

namespace Syntactic {
  std::string SyntacticBase::parse(std::string text) {
    // Create an int for end, since we need to pass it by reference
    // and string::length() returns std::size_t
    int end = text.length();
    return parse(text, false, 0, end);
  }

  std::string
  SyntacticBase::parse(std::string text, bool isFunction, int start, int &end) {
    std::string result = "";
    bool escaped = false;
    int i;

    for (i = start; i < text.length(); i++) {
      char c = text[i];

      if (!escaped) {
        if (c == START) {
          std::string parsed = parse(text, true, i + 1, i);
          result += parsed;
          continue;
        } else if (c == END || c == PARAM_DELIM) {
          break;
        } else if (c == ESCAPE) {
          escaped = true;
          continue;
        } else if (isFunction && c == PARAM_START) {
          std::list<std::string> params;

          do {
            std::string parsed = parse(text, false, i + 1, i);
            params.push_back(parsed);
          } while (i < text.length() && text[i] == PARAM_DELIM);

          end = i;

          return call(result, params);
        }
      }

      escaped = false;
      result += c;
    }

    end = i;

    if (isFunction) {
      return call(result, {});
    }

    return result;
  }

  Syntactic::Syntactic(Data data) {
    this->data = data;
  }

  std::string Syntactic::call(std::string name, std::list<std::string> params) {
    if (name.empty()) {
      throw EmptyFunctionNameError();
    }

    // TODO: Make syntactic functions modular, that way we can add more
    // functions without having to bloat this class (and specifically this
    // method)

    if (name == "filename") { // filename function: Returns the file name
      return data.fileName;
    } else if (name == "filepath") { // filepath function: Returns the file path
      return data.filePath;
    } else if (name == "random") { // random function: Takes all of the
                                   // parameters as string values and chooses a
                                   // random one
      if (params.empty()) {
        throw NoRandomParametersError();
      }
      int random = rand() % params.size();
      std::list<std::string>::iterator it = params.begin();
      std::advance(it, random);
      return *it;
    } else if (name == "response") { // response function: Takes the response
                                     // body from the server and returns it as a
                                     // string
      return data.response->body.str();
    } else if (name == "header") { // header function: Takes the response header
                                   // specified with the first parameter and
                                   // returns it as a string, if the header
                                   // doesn't exist it returns an empty string

      // Is the response ready?
      if (!data.response->ready) {
        throw ResponseNotReadyError();
      }

      // Is there a header specified?
      if (params.size() == 0) {
        throw MissingFunctionParameterError("header", 0, 1);
      }

      std::string header = params.front();
      if (data.response->headers.find(header) != data.response->headers.end()) {
        return data.response->headers[header];
      } else {
        return "";
      }
    } else if (name == "base64") { // base64 function: Take all parameters, join
                                   // them together, convert the end result to
                                   // base64, and return that
      if (params.empty()) {
        return "";
      }

      std::string paramsJoined = params.front();
      params.pop_front();

      for (std::string param : params) {
        paramsJoined += param;
      }

      return base64_encode(paramsJoined, true);
    } else if (name == "json") { // json function: Take the first parameter as a
                                 // jsonpath, use the body from the response as
                                 // the json, and return the result

      // Is the response ready?
      if (!data.response->ready) {
        throw ResponseNotReadyError();
      }

      if (params.size() == 0) {
        throw MissingFunctionParameterError("json", 0, 1);
      }

      std::string jsonPath = params.front();

      jsoncons::json json = jsoncons::json::parse(data.response->body.str());
      jsoncons::json result = jsoncons::jsonpath::json_query(json, jsonPath)[0];

      return result.as_string();
    } else if (name == "xml") { // xml function: Same premise as JSON, but for
                                // XML
      // Is the response ready?
      if (!data.response->ready) {
        throw ResponseNotReadyError();
      }

      if (params.size() == 0) {
        throw MissingFunctionParameterError("xml", 0, 1);
      }

      std::string xmlPath = params.front();

      pugi::xml_document doc;
      pugi::xml_parse_result result =
        doc.load_string(data.response->body.str().c_str());

      if (!result) {
        throw std::runtime_error(result.description());
      }

      pugi::xpath_node_set nodes = doc.select_nodes(xmlPath.c_str());

      if (nodes.size() == 0) {
        return "";
      }

      return nodes[0].node().child_value();
    }

    throw UnknownFunctionError(name);
  }

  SyntacticError::SyntacticError(std::string message) {
    this->message = message;
  }

  const char *SyntacticError::what() const throw() {
    return message.c_str();
  }

  EmptyFunctionNameError::EmptyFunctionNameError()
      : SyntacticError("Empty function name") {
  }

  UnknownFunctionError::UnknownFunctionError(std::string name)
      : SyntacticError("Invalid function name: " + name) {
  }

  MissingFunctionParameterError::MissingFunctionParameterError(
    std::string name, int given, int expected
  )
      : SyntacticError(
          "Invalid number of parameters for function " + name + ": " +
          std::to_string(given) + " was given, but expected " +
          std::to_string(expected)
        ) {
  }

  NoRandomParametersError::NoRandomParametersError()
      : SyntacticError(
          "No parameters given to random function, at least one is required"
        ) {
  }

  ResponseNotReadyError::ResponseNotReadyError()
      : SyntacticError("Response not ready for function") {
  }

} // namespace Syntactic