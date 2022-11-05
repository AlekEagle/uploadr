#pragma once
#include "../curlyfries/curlyfries.hpp"
#include <string>

namespace Template {
  struct ParserData {
      std::string filename;
      std::string input;
      curlyfries::Response *response;
  };
} // namespace Template

namespace Function {
  class CustomFunction {
    public:
      virtual const std::string name() = 0;
      virtual const int minParams() = 0;
      virtual std::string
      call(Template::ParserData *parserData, std::list<std::string> params) = 0;
      CustomFunction(){};
  };
}