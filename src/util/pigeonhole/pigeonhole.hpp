#pragma once
#include "../config/config.hpp"
#include "../cookies/cookies.hpp"
#include "../stopgap/stopgap.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <jsoncons_ext/csv/csv.hpp>
#include <string>

/*
  The history CSV file is a CSV file with the following columns:
  - Filename: The filename of the file in the archive, always
  timestamp.extension
  - URL: The URL that the file can be accessed at
  - ManageURL: The URL that the file can be managed at (If provided by API)
  - ThumbURL: The URL that the thumbnail of the file can be accessed at, if any
  (If provided by API)
*/

// Pigeonhole, the archive manager for uploadr
namespace Pigeonhole {
  std::string curDateTimeToString();
  jsoncons::csv::csv_options getCSVOptions();
  // Individual file class
  class File {
    private:
      std::filesystem::path path;
      std::string url;
      std::string manageUrl;
      std::string thumbUrl;
      Config::MainConfig &config;

    public:
      File(Config::MainConfig &config, std::filesystem::path path)
          : config(config), path(path) {
      }
      std::filesystem::path getPath() {
        return path;
      }
      std::string getURL() {
        return url;
      }
      std::string getManageURL() {
        return manageUrl;
      }
      std::string getThumbURL() {
        return thumbUrl;
      }
      void setURL(std::string url) {
        this->url = url;
      }
      void setManageURL(std::string manageUrl) {
        this->manageUrl = manageUrl;
      }
      void setThumbURL(std::string thumbUrl) {
        this->thumbUrl = thumbUrl;
      }
      void commit();
  };
  // The archive manager
  class Archive {
    private:
      Config::MainConfig &config;

    public:
      Archive(Config::MainConfig &config) : config(config) {
      }
      File addFile(std::filesystem::path path);
      File addFile(StopgapFile file);
      Config::MainConfig &getConfig() {
        return config;
      }
      std::string outputHistory();
  };
} // namespace Pigeonhole