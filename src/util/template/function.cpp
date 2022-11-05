#include "function.hpp"
#include "template.hpp"
#include <iostream>
#include <list>
#include <string>

namespace Function {
  const char *CustomFunctionBase64::b64Alphabet =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

  std::string CustomFunctionBase64::call(
    Template::ParserData *parserData, std::list<std::string> params
  ) {
    std::string input = params.front();
    std::string result = "";
    int i = 0;
    int j = 0;
    unsigned char charArray3[3];
    unsigned char charArray4[4];

    if (input.empty()) {
      return result;
    }

    while (input[i]) {
      charArray3[j++] = input[i++];
      if (j == 3) {
        charArray4[0] = (charArray3[0] & 0xfc) >> 2;
        charArray4[1] =
          ((charArray3[0] & 0x03) << 4) + ((charArray3[1] & 0xf0) >> 4);
        charArray4[2] =
          ((charArray3[1] & 0x0f) << 2) + ((charArray3[2] & 0xc0) >> 6);
        charArray4[3] = charArray3[2] & 0x3f;

        for (j = 0; (j < 4); j++) {
          result += b64Alphabet[charArray4[j]];
        }
        j = 0;
      }
    }

    if (j) {
      for (i = j; i < 3; i++) {
        charArray3[i] = '\0';
      }

      charArray4[0] = (charArray3[0] & 0xfc) >> 2;
      charArray4[1] =
        ((charArray3[0] & 0x03) << 4) + ((charArray3[1] & 0xf0) >> 4);
      charArray4[2] =
        ((charArray3[1] & 0x0f) << 2) + ((charArray3[2] & 0xc0) >> 6);

      for (i = 0; (i < j + 1); i++) {
        result += b64Alphabet[charArray4[i]];
      }

      while ((j++ < 3)) {
        result += '=';
      }
    }

    return result;
  }

  std::string CustomFunctionFilename::call(
    Template::ParserData *parserData, std::list<std::string> params
  ) {
    return parserData->filename;
  }

  std::string CustomFunctionInput::call(
    Template::ParserData *parserData, std::list<std::string> params
  ) {
    return parserData->input;
  }
} // namespace Function