#include "cookies.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <magic.h>
#include <sstream>
#include <string>
#include <vector>

using namespace std;
namespace fs = std::filesystem;

namespace Cookies {
  string getMimeType(const char *buffer, size_t size, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    string mime = magic_buffer(magic, buffer, size);
    magic_close(magic);
    return mime;
  }

  string getMimeType(vector<char> &buffer, int flags) {
    return getMimeType(buffer.data(), buffer.size(), flags);
  }

  string getMimeType(istream &stream, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    vector<char> buffer(
      (istreambuf_iterator<char>(stream)), (istreambuf_iterator<char>())
    );
    // Reset the stream to the beginning
    stream.seekg(0, ios::beg);
    string mime = magic_buffer(magic, buffer.data(), buffer.size());
    magic_close(magic);
    return mime;
  }

  string getMimeType(const string &path, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    string mime = magic_file(magic, path.c_str());
    magic_close(magic);
    return mime;
  }

  string getMimeType(const fs::path &path, int flags) {
    return getMimeType(path.string(), flags);
  }

  string getExtension(const char *buffer, size_t size, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    string ext = magic_buffer(magic, buffer, size);
    magic_close(magic);
    return ext;
  }

  string getExtension(vector<char> &buffer, int flags) {
    return getExtension(buffer.data(), buffer.size(), flags);
  }

  string getExtension(istream &stream, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    vector<char> buffer(
      (istreambuf_iterator<char>(stream)), (istreambuf_iterator<char>())
    );
    // Reset the stream to the beginning
    stream.seekg(0, ios::beg);
    string ext = magic_buffer(magic, buffer.data(), buffer.size());
    magic_close(magic);
    return ext;
  }

  string getExtension(const string &path, int flags) {
    magic_t magic = magic_open(flags);
    magic_load(magic, NULL);
    string ext = magic_file(magic, path.c_str());
    magic_close(magic);
    return ext;
  }

  string getExtension(const fs::path &path, int flags) {
    return getExtension(path.string(), flags);
  }
} // namespace Cookies