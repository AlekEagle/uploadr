#include "util/cloning-machine/cloning-machine.hpp"
#include "util/config/config.hpp"
#include "util/curlyfries/curlyfries.hpp"
#include "util/flags/flags.hpp"
#include "util/pigeonhole/pigeonhole.hpp"
#include "util/syntactic/syntactic.hpp"
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <jsoncons/json.hpp>
#include <magic.h>
#include <pugixml.hpp>
#include <unistd.h>

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

void debug(string msg) {
  if (getenv("DEBUG") != NULL) {
    cout << msg << endl;
  }
}

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

    // Setup pigeonhole
    Pigeonhole::Archive pigeonhole(*config);

    // Get the path to the file to upload
    std::filesystem::path filePath;
    if (args.arguments[0] == "-") {
      // TODO: this is broken, everything ends up completely empty
      // Before we do anything, check if stdin is a TTY
      if (isatty(fileno(stdin))) {
        // If it is, print an error message and exit
        cerr << "Error: stdin is a TTY" << endl;
        return 1;
      }
      // Before we do anything, check if stdin actually has any data
      if (std::cin.peek() == std::ifstream::traits_type::eof()) {
        // If not, tell the user they're an idiot and exit
        cerr << "Error: No data was passed to stdin" << endl;
        return 1;
      }
      // Use piegonhole to create a file we can later upload
      // clone the stdin stream
      vector<char> buffer(
        (istreambuf_iterator<char>(std::cin)), (istreambuf_iterator<char>())
      );

      std::string filename = pigeonhole.addFile(buffer);
      filePath =
        std::filesystem::path(config->get("archive")["path"].as_string()) /
        filename;

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
    //   - Binary (Text is the same as Binary. It's just a string)

    /*
    -----------------------------------------------------------------------------------------
                                       BODY CONSTRUCTION
    -----------------------------------------------------------------------------------------
    */

    // Check if the body type is undefined
    if (uploader->get("request")["body"].contains("type")) {
      // Get the body type
      std::string bodyType =
        uploader->get("request")["body"]["type"].as_string();

      // Check if the body type is MultiPartFormData
      // TODO: There's probably a better way to do this, like iterating over the
      //       fields first and then then deciding what to do with the body,
      //       that way we don't have to make so for statements. Sounds like its
      //       time for a refactor, and the initial implementation isn't even
      //       done yet!
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
      } else if (bodyType == "FormUrlEncoded") {
        // Create a string to store the form url encoded data
        std::string formUrlEncodedData;
        // Get the raw fields object
        jsoncons::json fields = uploader->get("request")["body"]["fields"];
        // Use the fields object to add each field
        // Fields are stored as an object, so we can iterate through it
        for (auto &field : fields.object_range()) {
          // Find a field with the value "{content}"
          if (field.value().as_string() == "{content}") {
            // Since we can't add files to form url encoded data, we'll just
            // read the file and add it to the form url encoded data. (It better
            // be a text file!)
            // Open the file
            std::ifstream file(filePath);
            // Read the file
            std::string fileContents(
              (std::istreambuf_iterator<char>(file)),
              std::istreambuf_iterator<char>()
            );
            // Before we add the file contents to the form url encoded data,
            // we need to make sure it's url encoded
            formUrlEncodedData += field.key() + "=" +
                                  curlyfries::CurlyFry::escape(fileContents) +
                                  "&";
          } else {
            // Parse the field value and add it to the form url encoded data
            formUrlEncodedData += field.key() + "=" +
                                  syntactic.parse(field.value().as_string()) +
                                  "&";
          }
        }
        // After we've added all the fields, remove the trailing "&" and add
        // the form url encoded data to the request
        formUrlEncodedData.pop_back();
        curlyFry->setBody(formUrlEncodedData);
        // Set the content type to application/x-www-form-urlencoded
        curlyFry->addHeader(
          "Content-Type", "application/x-www-form-urlencoded"
        );
      } else if (bodyType == "JSON") {
        // Create a json object to store the json data
        jsoncons::json jsonData;
        // Get the raw fields object
        jsoncons::json fields = uploader->get("request")["body"]["fields"];
        // For now, we only can work with shallow json objects, no nested
        // objects or arrays
        // Use the fields object to add each field
        // Fields are stored as an object, so we can iterate through it
        for (auto &field : fields.object_range()) {
          // Find a field with the value "{content}"
          if (field.value().as_string() == "{content}") {
            // Since we can't add files to json data, we'll just read the file
            // and add it to the json data. (It better be a text file!)
            // Open the file
            std::ifstream file(filePath);
            // Read the file
            std::string fileContents(
              (std::istreambuf_iterator<char>(file)),
              std::istreambuf_iterator<char>()
            );
            // Add the file contents to the json data
            jsonData[field.key()] = fileContents;
          } else {
            // Parse the field value and add it to the json data
            jsonData[field.key()] = syntactic.parse(field.value().as_string());
          }
        }

        // After we've added all the fields, add the json data to the request
        curlyFry->setBody(jsonData);
        // We don't need to set the content type, because it's already set when
        // we pass a json object for the body
      } else if (bodyType == "XML") {
        // Use a pugixml document to store the xml data
        pugi::xml_document xmlData;
        // Get the raw fields object
        jsoncons::json fields = uploader->get("request")["body"]["fields"];
        // For now, we only can work with shallow xml documents, no nested
        // elements
        // Use the fields object to add each field
        // Fields are stored as an object, so we can iterate through it
        for (auto &field : fields.object_range()) {
          // Find a field with the value "{content}"
          if (field.value().as_string() == "{content}") {
            // Since we can't add files to xml data, we'll just read the file
            // and add it to the xml data. (It better be a text file!)
            // Open the file
            std::ifstream file(filePath);
            // Read the file
            std::string fileContents(
              (std::istreambuf_iterator<char>(file)),
              std::istreambuf_iterator<char>()
            );
            // Add the file contents to a pugixml node and add it to the xml
            // document
            pugi::xml_node node = xmlData.append_child(field.key().c_str());
            node.append_child(pugi::node_pcdata)
              .set_value(fileContents.c_str());
          } else {
            // Parse the field value and add it to the xml data
            pugi::xml_node node = xmlData.append_child(field.key().c_str());
            node.append_child(pugi::node_pcdata)
              .set_value(syntactic.parse(field.value().as_string()).c_str());
          }
        }

        // After we've added all the fields, add the xml data to the request
        curlyFry->setBody(xmlData);
      } else if (bodyType == "Binary" || bodyType == "Text") {
        // We can't use fields with binary data, so we'll just read the file
        // and add it directly to the request
        // Open a file stream to the file and pass it to the request
        std::ifstream file(filePath);
        curlyFry->setBody(file);
        // Set the content type to application/octet-stream
        curlyFry->addHeader("Content-Type", "application/octet-stream");
      } else {
        // We don't know what body type it is, so we'll just fail
        // spectacularly
        throw std::runtime_error("Unknown body type: " + bodyType);
      }
    } else {
      // Assume the body type is None
      // Do nothing
    }

    // Get the raw headers object
    jsoncons::json headers = uploader->get("request")["headers"];

    // Add the headers to the request
    for (const auto &header : headers.object_range()) {
      curlyFry->addHeader(header.key(), header.value().as_string());
    }

    // Now that we've set up the request, we can send it
    int responseCode = curlyFry->send();

    // Check if the response code is 2XX
    if (responseCode / 100 == 2) {
      // Use syntactic to parse the responseUrl
      std::string responseUrl =
        syntactic.parse(uploader->get("response")["url"].as_string());
      std::string manageUrl;
      std::string thumbnailUrl;
      // Check if the uploader expects other URLs (i.e. a manage url or
      // thumbnail url)
      if (uploader->get("response").contains("manageUrl")) {
        // Use syntactic to parse the manageUrl
        manageUrl =
          syntactic.parse(uploader->get("response")["manageUrl"].as_string());
      }

      if (uploader->get("response").contains("thumbnailUrl")) {
        // Use syntactic to parse the thumbnailUrl
        thumbnailUrl =
          syntactic.parse(uploader->get("response")["thumbnailUrl"].as_string()
          );
      }

      // Print the response urls
      cout << responseUrl << endl;
      if (!manageUrl.empty()) {
        cout << manageUrl << endl;
      }
      if (!thumbnailUrl.empty()) {
        cout << thumbnailUrl << endl;
      }
    } else {
      // TODO: Handle non okay requests
      // until then, print the response
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