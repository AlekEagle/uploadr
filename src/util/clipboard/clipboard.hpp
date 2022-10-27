#pragma once
#include <clip.h>
#include <string>

namespace Clipboard {
  bool setClipboardText(const char *text);
  std::string getClipboardText();
}