#include "clipboard.hpp"
#include <clip.h>
#include <string>


bool Clipboard::setClipboardText(const char *text)
{
  return clip::set_text(text);
}

std::string Clipboard::getClipboardText()
{
  // Check if the clipboard contains text
  if (clip::has(clip::text_format()))
  {
    // Get the text from the clipboard
    std::string text;
    clip::get_text(text);
    return text;
  }
  else
  {
    // Return an empty string
    return "";
  }
}

