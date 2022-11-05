#pragma once
#include "../curlyfries/curlyfries.hpp"
#include "function.hpp"
#include "template-data.hpp"
#include <list>
#include <string>

// A set of classes for parsing template strings and replacing variables in
// uploader configs. Strongly inspired by ShareX's templating system.
namespace Template {
  const char START = '{';
  const char END = '}';
  const char ESCAPE = '\\';
  const char PARAM_START = ':';
  const char PARAM_DELIM = '|';

  // The base class for all template parser helpers
  class Base {
    private:
      std::string text;
      std::string _parse(bool func, int start, int &end);

    protected:
      virtual std::string
      call(std::string funcName, std::list<std::string> params);

    public:
      Base(std::string text);
      virtual std::string parse() = 0;
  };

  class Parser : public Base {
    private:
      static std::list<Function::CustomFunction> customFunctions;

    public:
      Template::ParserData *parserData;
      Parser(Template::ParserData *parserData);
      std::string parse();

    protected:
      std::string call(std::string funcName, std::list<std::string> params);
  };

  class TemplateException : public std::exception {
    private:
      std::string message;

    public:
      TemplateException(std::string message);
      const char *what() const throw();
  };
} // namespace Template