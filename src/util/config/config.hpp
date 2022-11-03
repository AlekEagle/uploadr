#pragma once
#include <filesystem>
#include <jsoncons/json.hpp>
#include <string>

namespace fs = std::filesystem;

namespace Config {

  class MainConfig {
    private:
      fs::path configPath;
      jsoncons::json config;

    public:
      // Returns the default config directory path.
      static fs::path getDefaultConfigPath();
      // Returns the default config file.
      static jsoncons::json getDefaultConfig();
      // Returns the default imgur uploader config.
      static jsoncons::json getDefaultImgurConfig();

      // Create a new instance of the config class. Specify a custom config
      // directory path, or use the default.
      MainConfig(fs::path configPath = getDefaultConfigPath());
      // Close and destroy the config class.
      ~MainConfig();

      // Returns the config directory path.
      fs::path getConfigPath();
      // Get a value from the config file.
      jsoncons::json get(std::string key);
      // Also include the [] operator.
      jsoncons::json operator[](std::string key);
      // Get the raw config json.
      jsoncons::json getRaw();
  };

  // Similar to the MainConfig class, but for uploader configs.
  class UploaderConfig {
    private:
      fs::path configPath;
      jsoncons::json config;
      MainConfig *mainConfig;

    public:
      UploaderConfig(std::string name, MainConfig *mainConfig);
      ~UploaderConfig();

      fs::path getConfigPath();
      jsoncons::json get(std::string key);
      jsoncons::json operator[](std::string key);
      jsoncons::json getRaw();
  };

  class ConfigError : public std::exception {
    private:
      std::string message;

    public:
      ConfigError(std::string message);
      const char *what() const throw();
  };
} // namespace Config