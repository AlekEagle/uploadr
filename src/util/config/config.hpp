#pragma once
#include <filesystem>
#include <jsoncons/json.hpp>
#include <string>

namespace fs = std::filesystem;

class Config {
  private:
    fs::path configPath;
    jsoncons::json config;

  public:
    // Returns the default config directory path.
    static fs::path getDefaultConfigPath();
    // Returns the default config file.
    static jsoncons::json getDefaultConfig();

    // Create a new instance of the config class. Specify a custom config
    // directory path, or use the default.
    Config(fs::path configPath = getDefaultConfigPath());
    // Close and destroy the config class.
    ~Config();

    // Returns the config directory path.
    fs::path getConfigPath();
};