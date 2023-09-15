#include "util/clipboard/clipboard.hpp"
#include "util/config/config.hpp"
#include "util/cookies/cookies.hpp"
#include "util/curlyfries/curlyfries.hpp"
#include "util/flags/flags.hpp"
#include "util/not-notify/not-notify.hpp"
#include "util/pigeonhole/pigeonhole.hpp"
#include "util/stopgap/stopgap.hpp"
#include "util/syntactic/syntactic.hpp"
#include "version.h"
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
  "Uploadr v" PROJECT_VER
  "\nUsage: uploadr [OPTIONS] [FILE|-]"
  "\nSpecify a file to upload, use - to read from stdin, or don't specify "
  "anything to upload the clipboard contents."
  "\nOPTIONS:"
  "\n  -h, --help                        Show this help message and exit"
  "\n  -H, --history                     Output a greppable history of all "
  "files you've uploaded"
  "\n  -c=CONFIG, --config=CONFIG        Specify the config directory (must "
  "be an absolute path)"
  "\n  -u=UPLOADER, --uploader=UPLOADER  Override the uploader specified in "
  "the config file";

void debug(std::string msg) {
  if (getenv("DEBUG") != NULL) {
    cout << msg << endl;
  }
}

void openURL(std::string url) {
  std::string cmd = "xdg-open " + url + " > /dev/null 2>&1";
  system(cmd.c_str());
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

    // Check if the version flag was passed
    if (args.getBool("v") || args.getBool("version")) {
      // Print the version
      cout << PROJECT_VER << endl;

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

    // See if the user wants to store uploads in the archive
    Pigeonhole::Archive *archive;
    if (config->get("archive")["enabled"].as_bool()) {
      archive = new Pigeonhole::Archive(*config);
    } else {
      archive = NULL;
    }

    // Check if the user wants to output the history
    if (args.getBool("H") || args.getBool("history")) {
      // If the archive is disabled, tell the user
      if (archive == NULL) {
        cerr << "Archive is disabled, cannot output history" << endl;
        return 1;
      }

      // Output the history
      cout << archive->outputHistory() << endl;

      // Exit
      return 0;
    }

    // Check if the user wants notifications
    bool notify = config->get("notification")["enabled"].as_bool();
    if (notify) {
      // Initialize libnotify
      if (!NotNotify::init("Uploadr")) {
        cerr << "Failed to initialize libnotify" << endl;
        return 1;
      }
    }

    NotNotify::Notification *notification;

    // Create a stopgap object to store our files in
    Stopgap stopgap;

    // Create a stopgap file to store the file contents in
    std::string fnameOverride;
    StopgapFile file(stopgap.createFile());
    if (args.arguments[0] == "-") { // Read from stdin
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

      // Read the contents of stdin into the stopgap file
      file.pipe(std::cin);

      // Test for a file extension
      std::string ext = file.testExtension();
      // If the extension is "???", we'll just use txt
      if (ext == "???") {
        ext = "txt";
        file.setExtension("txt");
      } else {
        file.setExtension(ext);
      }

    } else if (args.arguments[0] == "") { // Read from clipboard
      // Before we do anything, check if the clipboard actually has any data
      if (!Clipboard::has()) {
        // If not, tell the user they're an idiot and exit
        cerr << "Error: No data was found in the clipboard" << endl;
        return 1;
      }
      // Awesome, the clipboard has data, let's get it and put it in our
      // fileBuffer
      // Create a temporary char vector to store the clipboard contents in
      std::vector<char> clipboardContents;
      // Get the clipboard contents
      Clipboard::get(clipboardContents);
      // Write the clipboard contents to the stopgap file
      file.writeContents(clipboardContents);
      std::string ext = Cookies::getExtension(clipboardContents);
      // Get the file extension from the clipboard contents
      fnameOverride = "clipboard-content." + ext;
      // If the extension is "???", we'll just use txt
      if (fnameOverride == "clipboard-content.???") {
        fnameOverride = "clipboard-content.txt";
        file.setExtension("txt");
      } else {
        file.setExtension(ext);
      }
    } else { // Read from a file
      // If the user specified a file, use that
      // Check if the file exists
      if (!std::filesystem::exists(std::filesystem::path(args.arguments[0]))) {
        // If not, tell the user they're an idiot and exit
        cerr << "Error: File does not exist" << endl;
        return 1;
      }
      // If it does, use it
      file.pipe(std::ifstream(args.arguments[0], std::ios::binary));
      // and use the filename as the filename
      fnameOverride = std::filesystem::path(args.arguments[0]).filename();
      file.setExtension(std::filesystem::path(args.arguments[0]).extension());
    }

    // Now that we have the uploader and the file info, we can get curlyfries
    // setup

    // Create a new CurlyFry
    curlyfries::CurlyFry *curlyFry = new curlyfries::CurlyFry();

    // Take care of the easy stuff first
    curlyFry->setUrl(uploader->get("request")["url"].as_string());
    curlyFry->setMethod(uploader->get("request")["method"].as_string());

    // Create a new syntactic object
    Syntactic::Data data;
    data.fileName =
      fnameOverride == "" ? file.getPath().filename().string() : fnameOverride;
    data.filePath = file.getPath().string();
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
            // We can't use the buffer we have, so just give it the filepath
            multipartFormData.push_back(
              new cURLpp::FormParts::File(field.key(), file.getPath())
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
            // Use our buffer from earlier and turn it into a string
            std::string fileContents(
              file.getContents().begin(), file.getContents().end()
            );
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
            // Use our buffer from earlier and turn it into a string
            std::string fileContents(
              file.getContents().begin(), file.getContents().end()
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
            // Use our buffer from earlier and turn it into a string
            std::string fileContents(
              file.getContents().begin(), file.getContents().end()
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
        // Give curlyfries the buffer we have
        curlyFry->setBody(file.getContents());
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

    // Add the User-Agent header to the request
    curlyFry->addHeader(string("User-Agent"), string("Uploadr " PROJECT_VER));

    // If the user want's notifications, we'll send them a notification
    if (notify) {
      notification = new NotNotify::Notification(
        "Uploading", "Uploading " + data.fileName + " to " + uploader->getName()
      );
      notification->makePersistent();
      notification->show();
    }

    // Now that we've set up the request, we can send it
    int responseCode = curlyFry->send();

    // Now that we've sent the request and gotten the response, we can delete
    // the notification (We have to if we want to send another one)
    if (notify) {
      delete notification;
      // Create a new notification to tell the user if the upload was
      // successful or not
      notification = new NotNotify::Notification();

      notification->setTimeout(
        config->get("notification")["timeout"].as<int>() * 1000
      );
    }

    // Check if the response code is 2XX
    if (responseCode / 100 == 2) {
      // The upload was successful
      if (notify) {
        notification->updateNotification(
          "Upload Successful",
          "Uploaded " + data.fileName + " to " + uploader->getName()
        );
      }
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

      // Does a Pigeonhole::Archive instance exist? (Did the user enable
      // archiving?)
      if (archive) {
        // Since we don't allow stopgap to use the assignment operator, we'll
        // create a file just with the path
        Pigeonhole::File archiveFile(archive->addFile(file.getPath()));
        // Add the urls to the file
        archiveFile.setURL(responseUrl);
        archiveFile.setManageURL(manageUrl);
        archiveFile.setThumbURL(thumbnailUrl);
        // Commit the file to the archive
        archiveFile.commit();
      }

      // Print the response urls
      cout << responseUrl << endl;
      // If we have a notification object, we'll create a default action to
      // open the response url, as well as a button to open the response url
      if (notify) {
        notification->addAction(
          "default", "Open in Browser",
          [](NotifyNotification *notification, char *action, void *data) {
            std::string url = (char *)data;
            std::string cmd = "xdg-open " + url + " > /dev/null 2>&1";
            system(cmd.c_str());
          },
          (void *)responseUrl.c_str()
        );
        notification->addAction(
          "open-response", "Open in Browser",
          [](NotifyNotification *notification, char *action, void *data) {
            std::string url = (char *)data;
            std::string cmd = "xdg-open " + url + " > /dev/null 2>&1";
            system(cmd.c_str());
          },
          (void *)responseUrl.c_str()
        );
      }
      if (!manageUrl.empty()) {
        cout << manageUrl << endl;
        if (notify) {
          // If we have a notification object, we'll create a button to open the
          // manage url
          notification->addAction(
            "open-manage", "Manage",
            [](NotifyNotification *notification, char *action, void *data) {
              std::string url = (char *)data;
              std::string cmd = "xdg-open " + url + " > /dev/null 2>&1";
              system(cmd.c_str());
            },
            (void *)manageUrl.c_str()
          );
        }
      }
      if (!thumbnailUrl.empty()) {
        cout << thumbnailUrl << endl;
        // TODO: if we have a notification object, get the thumbnail from the
        // url and set it as the notification's icon
      }

      // Did the user enable clipboard copying?
      if (config->get("clipboard")["enabled"].as_bool()) {
        // Copy the response url to the clipboard
        Clipboard::set(responseUrl);
      }

      if (notify) {
        notification->show();
        // Start the event loop so we can handle notification actions.
        // This will block the thread until the notification is closed
        notification->runLoop();
      }
      return 0;
    } else {
      // The upload failed
      if (notify) {
        notification->updateNotification(
          "Upload Failed",
          "Failed to upload " + data.fileName + " to " + uploader->getName()
        );
      }
      // TODO: Handle non okay requests better
      // until then, print the response
      cout << curlyFry->getResponse()->body.str() << endl;
      if (notify) {
        notification->show();
      }
      return 1;
    }
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