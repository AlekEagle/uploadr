#include "template.hpp"
#include "function.hpp"
#include "template-data.hpp"
#include <iostream>
#include <list>
#include <string>

namespace Template {
  Base::Base(std::string text) {
    this->text = text;
  }

  std::string Base::_parse(bool func, int start, int &end) {
    std::string result = "";
    bool escape = false;
    int i;

    for (i = start; i < this->text.length(); i++) {
      char curChar = this->text[i];

      if (!escape) {
        if (curChar == START) {
          std::string parsed = this->_parse(true, i + 1, i);
          result += parsed;
          continue;
        } else if (curChar == END || curChar == PARAM_DELIM) {
          break;
        } else if (curChar == ESCAPE) {
          escape = true;
          continue;
        } else if (func && curChar == PARAM_START) {
          std::list<std::string> params;

          do {
            std::string param = this->_parse(false, i + 1, i);
            params.push_back(param);
          } while (i < this->text.length() && this->text[i] == PARAM_DELIM);

          end = i;

          return this->call(result, params);
        }
      }

      escape = false;
      result += curChar;
    }

    end = i;

    if (func) {
      return this->call(result, std::list<std::string>());
    } else {
      return result;
    }
  }

  Parser::Parser(Template::ParserData *parserData) : Base(parserData->input) {
  }

  std::string Parser::parse() {
    }

  std::string
  Parser::call(std::string funcName, std::list<std::string> params) {
  }

  TemplateException::TemplateException(std::string message) {
    this->message = message;
  }
} // namespace Template