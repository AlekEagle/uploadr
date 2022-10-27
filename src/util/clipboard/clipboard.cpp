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
    return clip::get_text();
  } else {
    // Return an empty string
    return "";
  }
}

