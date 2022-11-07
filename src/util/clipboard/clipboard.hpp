#pragma once
#include <clip.h>
#include <string>
#include <vector>

namespace Clipboard {
  bool has();
  bool get(std::vector<char> &buffer);
  bool set(const std::string text);
} // namespace Clipboard