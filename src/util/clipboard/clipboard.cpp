#include "clipboard.hpp"
#include <clip.h>
#include <string>

namespace Clipboard {
  // Does the clipboard have data at all?
  bool has() {
    return !clip::has(clip::empty_format());
  }

  bool get(std::vector<char> &buffer) {
    bool result;
    // Check what kind of data is available
    if (clip::has(clip::image_format())) {
      // Get the image data
      clip::image image;
      result = clip::get_image(image);
      // Put the image data into the buffer
      // Tell the buffer how much data we're putting in it
      // clip::image doesn't have a size() function, so we have to get it from
      // the image spec
      clip::image_spec spec;
      clip::get_image_spec(spec);
      std::size_t size = spec.required_data_size();
      buffer.resize(size);
      // Copy the image data into the buffer
      buffer.assign(image.data(), image.data() + size);
      return result;
    } else if (clip::has(clip::text_format())) {
      // Get the text data
      std::string text;
      result = clip::get_text(text);
      // Put the text data into the buffer
      buffer.resize(text.size());
      buffer.assign(text.begin(), text.end());
      return result;
    } else {
      // No data available
      return false;
    }
  }

  bool set(const std::string text) {
    return clip::set_text(text);
  }
} // namespace Clipboard