#include "clipboard.hpp"
#include "clip/clip.h"

void Clipboard::setClipboard(const char *text)
{
  clip::set_text(text);
}