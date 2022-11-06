#include "pigeonhole.hpp"
#include "../config/config.hpp"
#include "../cookies/cookies.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <jsoncons_ext/csv/csv.hpp>
#include <string>

// Pigeonhole, the archive manager for uploadr
namespace Pigeonhole {

  std::string Archive::addFile(std::istream &stream) {
    // First, get a timestamp for the file
    std::string timestamp = Archive::curDateTimeToString();
    // Next, get the file's extension
    std::string extension = Cookies::getExtension(stream);
    // Next, if the extension is empty, fall back to the mime type
    if (extension.empty()) {
      std::string mime = Cookies::getMimeType(stream);
      // If the MIME type is text/plain, fall back to .txt
      if (mime == "text/plain") {
        extension = "txt";
      } else { // We have no idea what this is, so just use .bin
        extension = "bin";
      }
    }
    // with the timestamp and extension, we can now create the filename, and
    // the path to the file (using the path from the config as a base)
    std::string filename = timestamp + "." + extension;
    std::filesystem::path path =
      std::filesystem::path(config.get("archive")["path"].as_string()) /
      filename;
    // Make sure the directory exists
    std::filesystem::create_directories(path.parent_path());
    // Now, we can create the file
    std::ofstream file(path);
    // And copy the stream into it
    file << stream.rdbuf();
    // Finally, return the filename
    return filename;
  }

  std::string Archive::addFile(std::filesystem::path path) {
    // First, get a timestamp for the file
    std::string timestamp = Archive::curDateTimeToString();
    // Next, get the file's extension
    std::string extension = Cookies::getExtension(path);
    // Next, if the extension is empty, fall back to the mime type
    if (extension.empty()) {
      std::string mime = Cookies::getMimeType(path);
      // If the MIME type is text/plain, fall back to .txt
      if (mime == "text/plain") {
        extension = "txt";
      } else { // We have no idea what this is, so just use .bin
        extension = "bin";
      }
    }
    // with the timestamp and extension, we can now create the filename, and
    // the path to the file (using the path from the config as a base)
    std::string filename = timestamp + "." + extension;
    std::filesystem::path newpath =
      std::filesystem::path(config.get("archive")["path"].as_string()) /
      filename;
    // Make sure the directory exists
    std::filesystem::create_directories(newpath.parent_path());
    // Now, we can copy the file
    std::filesystem::copy(path, newpath);
    // Finally, return the filename
    return filename;
  }

  std::string Archive::addFile(std::string path) {
    return addFile(std::filesystem::path(path));
  }

  void Archive::setURLs(
    std::string filename, std::string url, std::string manageurl,
    std::string thumburl
  ) {
    // First, we need to get the path to the CSV file
    std::filesystem::path path =
      std::filesystem::path(config.get("archive")["histFile"].as_string());
    // Next, we need to configure how to read the CSV file
    jsoncons::csv::csv_options options;
    // We need to set the column names
    options.column_names("Filename,URL,ManageURL,ThumbURL");
    // And we need to set the quote character to be a single quote
    options.quote_char('\'');
    // Now, we can check if the file exists, and if it doesn't, we need to
    // create it
    if (!std::filesystem::exists(path)) {
      // We need to create the file
      std::ofstream file(path);
      // And write the column names to it
      file << options.column_names();
      // And close the file
      file.close();
    }
    // Now, we can open the file for parsing
    std::ifstream ifile(path);
    // And parse it
    jsoncons::json data =
      jsoncons::csv::decode_csv<jsoncons::json>(ifile, options);
    // we can close the istream now, as we don't need it anymore
    ifile.close();
    // Next, check if the number of rows is greater than maxCount from the
    // config
    if (data.size() > config.get("archive")["maxCount"]) {
      // If it is, we need to remove the oldest entry
      // Get the oldest entry, since we need to know the filename to delete the
      // old file too
      std::string oldFilename = data[0]["Filename"].as<std::string>();
      // Remove the oldest entry
      data.erase(0);
      // Now, we need to delete the old file
      std::filesystem::remove(
        std::filesystem::path(config.get("archive")["path"].as_string()) /
        oldFilename
      );
    }
    // Add the new entry (use a printf-style string to make it easier to read)
    char *str;
    asprintf(
      &str, "'%s','%s','%s','%s'", filename.c_str(), url.c_str(),
      manageurl.c_str(), thumburl.c_str()
    );
    data.push_back(str);
    // Prepare the CSV data for writing
    std::ostringstream os;
    jsoncons::csv::encode_csv(data, os, options);
    // Now, we can open the file for writing
    std::ofstream ofile(path);
    // And write the data to it
    ofile << os.str();
  }

  void Archive::setURLs(
    std::string filename, std::string url, std::string manageurl
  ) {
    return setURLs(filename, url, manageurl, "");
  }

  void Archive::setURLs(std::string filename, std::string url) {
    return setURLs(filename, url, "", "");
  }
} // namespace Pigeonhole