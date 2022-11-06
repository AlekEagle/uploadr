#pragma once
#include "../config/config.hpp"
#include "../cookies/cookies.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
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
  // The archive manager
  class Archive {
    private:
      Config::MainConfig &config;

    public:
      static std::string curDateTimeToString() {
        time_t now = time(0);
        struct tm tstruct;
        char buf[80];
        tstruct = *localtime(&now);
        strftime(buf, sizeof(buf), "%Y-%m-%d.%X", &tstruct);
        return buf;
      }
      Archive(Config::MainConfig &config) : config(config) {
      }
      std::string addFile(std::istream &stream);
      std::string addFile(std::filesystem::path path);
      std::string addFile(std::string path);

      void setURLs(
        std::string filename, std::string url, std::string manageurl,
        std::string thumburl
      );
      void
      setURLs(std::string filename, std::string url, std::string manageurl);
      void setURLs(std::string filename, std::string url);
  };
} // namespace Pigeonhole