#include "config.hpp"
#include <filesystem>
#include <fstream>
#include <jsoncons/json.hpp>

namespace fs = std::filesystem;

fs::path Config::getDefaultConfigPath() {
  return fs::path(getenv("HOME")) / ".config" / "uploadr";
}

jsoncons::json Config::getDefaultConfig() {
  jsoncons::json config;
  config["defaultUploader"] = "imgur";
  config["archive"]["enabled"] = true;
  config["archive"]["path"] =
      (Config::getDefaultConfigPath() / "archive").u8string();
  config["clipboard"]["enabled"] = true;
  config["notification"]["enabled"] = true;
  config["notification"]["timeout"] = 5;
  config["notification"]["sound"] = NULL;

  return config;
}

Config::Config(fs::path configPath) {
  this->configPath = configPath;
}

Config::~Config() {
  this->configPath = NULL;
  this->config = NULL;
}

int Config::init(bool force) {
  // Check if the config directory already exists
  if (fs::exists(this->configPath)) {
    if (force) {
      // Remove the config directory
      fs::remove_all(this->configPath);
    } else {
      // Return 2 if the config directory already exists
      return 2;
    }
  }

  // Create the config directory
  if (!fs::create_directories(this->configPath)) {
    return 1;
  }

  // Create the config file
  std::ofstream configFile(this->configPath / "config.json");
  if (!configFile.is_open()) {
    return 1;
  }

  // Write the default config to the config file
  configFile << jsoncons::pretty_print(Config::getDefaultConfig());
  configFile.close();

  return 0;
}