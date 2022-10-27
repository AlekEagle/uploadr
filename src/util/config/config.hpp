#pragma once
#include <filesystem>

namespace fs = std::filesystem;

namespace Config {
  const fs::path CONFIG_DIR = fs::path(getenv("HOME")) / ".config" / "uploadr";
  const fs::path CONFIG_FILE = CONFIG_DIR / "config.json";

  bool checkConfig();
  bool initConfig();
}