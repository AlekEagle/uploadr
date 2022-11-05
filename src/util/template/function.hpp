#pragma once
#include "../curlyfries/curlyfries.hpp"
#include "template-data.hpp"
#include "template.hpp"
#include <list>
#include <string>

namespace Function {
  class CustomFunctionBase64 : public CustomFunction {
    public:
      static const char *b64Alphabet;
      const std::string name() {
        return "base64";
      }
      const int minParams() {
        return 1;
      }
      std::string
      call(Template::ParserData *parserData, std::list<std::string> params);
  };

  class CustomFunctionFilename : public CustomFunction {
    public:
      const std::string name() {
        return "filename";
      }
      const int minParams() {
        return 1;
      }
      std::string
      call(Template::ParserData *parserData, std::list<std::string> params);
  };

  class CustomFunctionInput : public CustomFunction {
    public:
      const std::string name() {
        return "input";
      }
      const int minParams() {
        return 1;
      }
      std::string
      call(Template::ParserData *parserData, std::list<std::string> params);
  };
} // namespace Function