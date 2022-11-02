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
  config["defaultUploader"] = jsoncons::json::null();
  config["archive"]["enabled"] = true;
  config["archive"]["path"] =
      (fs::path(getenv("HOME")) / ".local" / "share" / "uploadr" / "archive")
          .u8string();
  config["archive"]["histFile"] = (fs::path(getenv("HOME")) / ".local" /
                                   "share" / "uploadr" / "history.csv")
                                      .u8string();
  config["clipboard"]["enabled"] = true;
  config["notification"]["enabled"] = true;
  config["notification"]["timeout"] = 5;
  config["notification"]["sound"] = jsoncons::json::null();

  return config;
}

Config::Config(fs::path configPath) {
  this->configPath = configPath;
  if (!fs::exists(configPath)) {
    fs::create_directories(configPath);
    fs::create_directories(configPath / "uploaders");

    std::ofstream configFile((configPath / "config.json").u8string());
    configFile << jsoncons::pretty_print(Config::getDefaultConfig());
  }

  std::ifstream configFile((configPath / "config.json").u8string());
  this->config = jsoncons::json::parse(configFile);
}

Config::~Config() {
  this->configPath = NULL;
  this->config = NULL;
}