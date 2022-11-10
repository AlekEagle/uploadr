#include "pigeonhole.hpp"
#include "../config/config.hpp"
#include "../cookies/cookies.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <jsoncons/json.hpp>
#include <jsoncons_ext/csv/csv.hpp>
#include <string>

// Pigeonhole, the archive manager for uploadr
namespace Pigeonhole {
  std::string curDateTimeToString() {
    time_t now = time(0);
    struct tm tstruct;
    char buf[80];
    tstruct = *localtime(&now);
    strftime(buf, sizeof(buf), "%Y%m%d_%H%M%S", &tstruct);
    return buf;
  }
  jsoncons::csv::csv_options getCSVOptions() {
    jsoncons::csv::csv_options options;
    // Tell it that the first row is a header
    options.header_lines(1);
    // We need to set the column names
    options.column_names("Filename,URL,ManageURL,ThumbURL");
    // And we need to set the quote character to be a single quote
    options.quote_char('\'');
    return options;
  }
  // The archive manager
  File Archive::addFile(std::filesystem::path path) {
    // Check if the file exists
    if (!std::filesystem::exists(path)) {
      throw std::runtime_error("File does not exist");
    }
    // Check if the file is a regular file
    if (!std::filesystem::is_regular_file(path)) {
      throw std::runtime_error("File is not a regular file");
    }
    // Create the file object and return it
    return File(config, path);
  }

  File Archive::addFile(StopgapFile file) {
    // Create the file object and return it
    return File(config, file.getPath());
  }

  std::string Archive::outputHistory() {
    std::string result = "Filename: URL | Manage URL | Thumbnail URL\n";
    // Read the history file
    std::ifstream historyFile(
      std::filesystem::path(config["archive"]["histFile"].as_string())
    );
    // Parse the CSV
    jsoncons::json history =
      jsoncons::csv::decode_csv<jsoncons::json>(historyFile, getCSVOptions());

    // Iterate over the history
    for (auto &file : history.array_range()) {
      // Add the filename
      result += file["Filename"].as_string() + ": ";
      // Add the URL
      result += file["URL"].as_string() + " | ";
      // Add the manage URL (if it exists)
      if (file.contains("ManageURL")) {
        result += file["ManageURL"].as_string() + " | ";
      } else {
        result += "N/A | ";
      }
      // Add the thumbnail URL (if it exists)
      if (file.contains("ThumbURL")) {
        result += file["ThumbURL"].as_string();
      } else {
        result += "N/A";
      }
      // Add a newline
      result += "\n";
    }
    return result;
  }

  // Individual file class
  void File::commit() {
    // Before we do anything else, make sure the archive directory exists
    std::filesystem::create_directories(config["archive"]["path"].as_string());
    // The file to archive isn't in the archive yet until this function is
    // called, that way we don't have to worry about deleting the file if
    // the user doesn't want to archive it

    // Get a timestamp for the filename
    std::string timestamp = Pigeonhole::curDateTimeToString();
    // Get the extension of the file, we want to use cookies to get the
    // extension from the magic number of the file if possible, but if
    // that fails we'll just use the extension of the file, and if the
    // file has no extension we'll just use MIME type to determine if
    // it's text or binary
    std::string extension = Cookies::getExtension(path);
    // If the extension is "???", we'll just use the extension of the
    // file
    if (extension == "???") {
      extension = path.extension();
      // Did the file have an extension?
      if (extension == "") {
        // No, it didn't, so we'll just use the MIME type to determine
        // if it's text or binary
        std::string mimetype = Cookies::getMimeType(path);
        if (mimetype == "text/plain") {
          extension = "txt";
        } else {
          extension = "bin";
        }
      } else {
        // The file had an extension, so we'll remove the dot from the
        // extension
        extension = extension.substr(1);
      }
    }

    // Create the filename
    std::string filename = timestamp + "." + extension;

    // Let's prepare to add the file to the history CSV file
    std::filesystem::path path =
      std::filesystem::path(config["archive"]["histFile"].as_string());
    // Now, we can check if the file exists, and if it doesn't, we need to
    // create it
    if (!std::filesystem::exists(path)) {
      // We need to create the file
      std::ofstream file(path);
      // And write the column names to it
      file << Pigeonhole::getCSVOptions().column_names();
      // And close the file
      file.close();
    }
    // Now, we can read the CSV file
    std::ifstream inFile(path);
    jsoncons::json history = jsoncons::csv::decode_csv<jsoncons::json>(
      inFile, Pigeonhole::getCSVOptions()
    );
    inFile.close();
    // Check if the history file has more than the maximum number of
    // entries specified in the config
    int maxEntries = config["archive"]["maxCount"].as<int>();
    if (history.size() > maxEntries) {
      // It does, so we need to remove the oldest entry
      // Get the name of the oldest entry so we can delete the file later
      std::string oldestEntry = history[0]["Filename"].as_string();
      // Remove the oldest entry
      auto first = history.array_range().begin();
      history.erase(first);
      std::filesystem::remove(
        std::filesystem::path(config["archive"]["path"].as_string()) /
        oldestEntry
      );
    }

    // Now, we can add the new entry to the history
    jsoncons::json newEntry;
    newEntry["Filename"] = filename;
    newEntry["URL"] = url;
    newEntry["ManageURL"] = manageUrl;
    newEntry["ThumbURL"] = thumbUrl;
    history.push_back(newEntry);
    // Prepare the history to be written to the CSV file
    std::ostringstream os;
    jsoncons::csv::encode_csv(history, os, Pigeonhole::getCSVOptions());

    // Copy the file to the archive
    std::filesystem::copy_file(
      path,
      std::filesystem::path(config["archive"]["path"].as_string()) / filename
    );
    // Write the history to the CSV file
    std::ofstream outFile(path);
    outFile << os.str();
    outFile.close();
  }
} // namespace Pigeonhole