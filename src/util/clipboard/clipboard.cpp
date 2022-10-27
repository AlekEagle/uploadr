#include "clipboard.hpp"
#include <clip.h>

void Clipboard::setClipboard(const char *text)
{
  clip::set_text(text);
}