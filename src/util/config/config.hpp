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

    // Initializes the config directory and files with default values. Returns 0
    // on success, 1 on failure, and 2 if the config directory already exists.
    // If force is true, the config directory will be overwritten and will never
    // return 2.
    int init(bool force = false);
};