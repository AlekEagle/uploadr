#include "config.hpp"
#include <filesystem>
#include <fstream>
#include <jsoncons/json.hpp>

namespace fs = std::filesystem;

bool Config::checkConfig()
{
  if (!fs::exists(CONFIG_DIR))
  {
    return false;
  }
  if (!fs::exists(CONFIG_FILE))
  {
    return false;
  }
  return true;
}

bool Config::initConfig() {
  // Create the config directory if it doesn't exist
  if (!fs::exists(CONFIG_DIR))
  {
    fs::create_directory(CONFIG_DIR);
  }

  // Create the config file, overwrite if it already exists
  std::ofstream config(CONFIG_FILE);
  // Write the default config to the file
  jsoncons::json configJson = jsoncons::json::object();

}