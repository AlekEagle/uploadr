#pragma once
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
  const int DEFAULT_MAGIC_MIME_FLAGS =
    MAGIC_MIME_TYPE | MAGIC_SYMLINK | MAGIC_ERROR;
  const int DEFAULT_MAGIC_EXT_FLAGS =
    MAGIC_EXTENSION | MAGIC_SYMLINK | MAGIC_ERROR;

  string getMimeType(
    const char *buffer, size_t size, int flags = DEFAULT_MAGIC_MIME_FLAGS
  );

  string
  getMimeType(vector<char> &buffer, int flags = DEFAULT_MAGIC_MIME_FLAGS);

  string getMimeType(istream &stream, int flags = DEFAULT_MAGIC_MIME_FLAGS);

  string getMimeType(const string &path, int flags = DEFAULT_MAGIC_MIME_FLAGS);

  string
  getMimeType(const fs::path &path, int flags = DEFAULT_MAGIC_MIME_FLAGS);

  string getExtension(
    const char *buffer, size_t size, int flags = DEFAULT_MAGIC_EXT_FLAGS
  );

  string
  getExtension(vector<char> &buffer, int flags = DEFAULT_MAGIC_EXT_FLAGS);

  string getExtension(istream &stream, int flags = DEFAULT_MAGIC_EXT_FLAGS);

  string getExtension(const string &path, int flags = DEFAULT_MAGIC_EXT_FLAGS);

  string
  getExtension(const fs::path &path, int flags = DEFAULT_MAGIC_EXT_FLAGS);
} // namespace Cookies
