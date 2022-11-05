#include "util/config/config.hpp"
#include "util/curlyfries/curlyfries.hpp"
#include "util/flags/flags.hpp"
#include "util/syntactic/syntactic.hpp"
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <filesystem>
#include <fstream>
#include <iostream>

const std::string HELP_MSG =
  "Usage: uploadr [OPTIONS] [FILE|-]"
  "\nSpecify a file to upload, use - to read from stdin, or don't specify "
  "anything to upload the clipboard contents."
  "\nOPTIONS:"
  "\n  -h, --help                        Show this help message and exit"
  "\n  -c=CONFIG, --config=CONFIG        Specify the config directory (must "
  "be an absolute path)"
  "\n  -u=UPLOADER, --uploader=UPLOADER  Override the uploader specified in "
  "the config file";

using namespace std;

int main(int argc, char **argv) {
  try {
    // Parse the arguments
    Flags args(argc, argv);

    // Check if the help flag was passed, or if no arguments were passed
    if (args.getBool("h") || args.getBool("help")) {
      // Print the help message
      cout << HELP_MSG << endl;

      // Exit
      return 0;
    }

    // If the user specified a different config directory, use that instead
    Config::MainConfig *config;
    if (args.exists("c") || args.exists("config")) {
      config = new Config::MainConfig(
        std::filesystem::path(args.exists("c") ? args["c"] : args["config"])
      );
    } else {
      config = new Config::MainConfig();
    }

    // If the user specified a different uploader, use that instead
    Config::UploaderConfig *uploader;
    if (args.exists("u") || args.exists("uploader")) {
      uploader = new Config::UploaderConfig(
        args.exists("u") ? args["u"] : args["uploader"], config
      );
    } else {
      uploader = new Config::UploaderConfig(
        config->get("defaultUploader").as_string(), config
      );
    }

    // Get the path to the file to upload
    std::filesystem::path filePath;
    if (args.arguments[0] == "-") {
      // TODO: Read from stdin
      // until we can read from stdin, just fail spectacularly
      throw std::runtime_error("Reading from stdin is not yet supported");
    } else if (args.arguments[0] == "") {
      // TODO: Read from clipboard
      // until we can read from the clipboard, just fail spectacularly
      throw std::runtime_error("Reading from the clipboard is not yet supported"
      );
    } else {
      // Use the file specified by the user
      filePath = std::filesystem::path(args.arguments[0]);
    }

    // Now that we have the uploader, we can set up curlyfries with the
    // uploader's config

    // Create a new CurlyFry
    curlyfries::CurlyFry *curlyFry = new curlyfries::CurlyFry();

    // Take care of the easy stuff first
    curlyFry->setUrl(uploader->get("request")["url"].as_string());
    curlyFry->setMethod(uploader->get("request")["method"].as_string());

    // Create a new syntactic object
    Syntactic::Data data;
    data.fileName = filePath.filename().string();
    data.filePath = filePath.string();
    data.response = curlyFry->getResponse();
    Syntactic::Syntactic syntactic(data);

    // Now we need to determine what body to send
    // The request.body.type field determines what type of body to send
    // It can be one of the following:
    //   - undefined (it wasn't provided, assume None)
    //   - None
    //   - MultipartFormData
    //   - FormUrlEncoded
    //   - JSON
    //   - XML
    //   - Binary
    //   - Text

    // Check if the body type is undefined
    if (uploader->get("request")["body"].contains("type")) {
      // Get the body type
      std::string bodyType =
        uploader->get("request")["body"]["type"].as_string();

      // Check if the body type is MultiPartFormData
      if (bodyType == "MultipartFormData") {
        // Create the multipart form data
        cURLpp::Forms multipartFormData;
        // Get the raw fields object
        jsoncons::json fields = uploader->get("request")["body"]["fields"];
        // Use the fields object to add each field
        // Fields are stored as an object, so we can iterate through it
        for (auto &field : fields.object_range()) {
          // Find a field with the value "{content}"
          if (field.value().as_string() == "{content}") {
            // Add the file to the multipart form data
            multipartFormData.push_back(
              new cURLpp::FormParts::File(field.key(), filePath.string())
            );
          } else {
            // Parse the field value and add it to the multipart form data
            multipartFormData.push_back(new cURLpp::FormParts::Content(
              field.key(), syntactic.parse(field.value().as_string())
            ));
          }
        }
        // After we've added all the fields, add the multipart form data to the
        // request
        curlyFry->setBody(multipartFormData);
      }
    }

    // Get the raw headers object
    jsoncons::json headers = uploader->get("request")["headers"];

    // Add the headers to the request
    for (const auto &header : headers.object_range()) {
      curlyFry->addHeader(header.key(), header.value().as_string());
    }

    // Now that we've set up the request, we can send it
    // Send the request
    int responseCode = curlyFry->send();

    // Check if the response code is 2XX
    if (responseCode / 100 == 2) {
      // TODO: Parse the response
      // until we can parse the response, just output the response
      cout << curlyFry->getResponse()->body.str() << endl;
    } else {
      // The response code isn't 2XX
      // Print the response body
      cout << curlyFry->getResponse()->body.str() << endl;
    }
    return 0;
  } catch (const Config::ConfigError &e) {
    // Print the error message
    cerr << e.what() << endl;

    // Exit
    return 1;
  }
  // Catch standard exceptions
  catch (const std::exception &e) {
    // Print the error message
    cerr << e.what() << endl;

    // Exit
    return 1;
  }
  // Catch all other exceptions
  catch (...) {
    // Print the error message
    cerr << "An unknown error occurred" << endl;

    // Exit
    return 1;
  }
}