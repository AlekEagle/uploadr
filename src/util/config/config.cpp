#include "config.hpp"
#include <filesystem>
#include <fstream>
#include <jsoncons/json.hpp>

namespace fs = std::filesystem;

namespace Config {

  fs::path MainConfig::getDefaultConfigPath() {
    return fs::path(getenv("HOME")) / ".config" / "uploadr";
  }

  jsoncons::json MainConfig::getDefaultConfig() {
    jsoncons::json config;
    config["defaultUploader"] = "imgur";
    config["archive"]["enabled"] = true;
    config["archive"]["path"] =
      (fs::path(getenv("HOME")) / ".local" / "share" / "uploadr" / "archive")
        .u8string();
    config["archive"]["histFile"] = (fs::path(getenv("HOME")) / ".local" /
                                     "share" / "uploadr" / "history.csv")
                                      .u8string();
    config["archive"]["maxCount"] = 1000;
    config["clipboard"]["enabled"] = true;
    config["notification"]["enabled"] = true;
    config["notification"]["timeout"] = 5;
    config["notification"]["sound"] = jsoncons::json::null();

    return config;
  }

  jsoncons::json MainConfig::getDefaultImgurConfig() {
    jsoncons::json config;
    config["request"]["method"] = "POST";
    config["request"]["url"] = "https://api.imgur.com/3/image";
    config["request"]["headers"]["Authorization"] = "Client-ID eb13b71463957f7";
    config["request"]["body"]["type"] = "MultipartFormData";
    config["request"]["body"]["fields"]["image"] = "{content}";
    config["response"]["url"] = "{json:$.data.link}";
    config["response"]["manageUrl"] =
      "https://imgur.com/delete/{json:$.data.deletehash}";

    return config;
  }

  MainConfig::MainConfig(fs::path configPath) {
    this->configPath = configPath;
    if (!fs::exists(configPath)) {
      fs::create_directories(configPath);
      fs::create_directories(configPath / "uploaders");

      std::ofstream configFile((configPath / "config.json").u8string());
      configFile << jsoncons::pretty_print(MainConfig::getDefaultConfig());
    }

    std::ifstream configFile((configPath / "config.json").u8string());
    this->config = jsoncons::json::parse(configFile);

    if (!fs::exists(configPath / "uploaders" / "imgur.uploader")) {
      std::ofstream imgurConfigFile(
        (configPath / "uploaders" / "imgur.uploader").u8string()
      );
      imgurConfigFile << jsoncons::pretty_print(
        MainConfig::getDefaultImgurConfig()
      );
    }
  }

  MainConfig::~MainConfig() {
    this->configPath = NULL;
    this->config = NULL;
  }

  fs::path MainConfig::getConfigPath() {
    return this->configPath;
  }

  jsoncons::json MainConfig::get(std::string key) {
    return this->config[key];
  }

  jsoncons::json MainConfig::operator[](std::string key) {
    return this->config[key];
  }

  jsoncons::json MainConfig::getRaw() {
    return this->config;
  }

  UploaderConfig::UploaderConfig(std::string name, MainConfig *mainConfig) {
    this->uploaderName = name;
    this->configPath =
      mainConfig->getConfigPath() / "uploaders" / (name + ".uploader");
    this->mainConfig = mainConfig;
    if (!fs::exists(this->configPath)) {
      // if the uploader doesn't exist, don't create it like we do with the main
      // config, because we don't know what the uploader should look like
      throw ConfigError("Uploader '" + name + "' does not exist.");
    }

    std::ifstream configFile(this->configPath.u8string());
    this->config = jsoncons::json::parse(configFile);
  }

  UploaderConfig::~UploaderConfig() {
    this->configPath = NULL;
    this->config = NULL;
    this->mainConfig = NULL;
  }

  fs::path UploaderConfig::getConfigPath() {
    return this->configPath;
  }

  jsoncons::json UploaderConfig::get(std::string key) {
    return this->config[key];
  }

  jsoncons::json UploaderConfig::operator[](std::string key) {
    return this->config[key];
  }

  jsoncons::json UploaderConfig::getRaw() {
    return this->config;
  }

  std::string UploaderConfig::getName() {
    return this->uploaderName;
  }

  ConfigError::ConfigError(std::string message) {
    this->message = message;
  }

  const char *ConfigError::what() const throw() {
    return this->message.c_str();
  }
} // namespace Config