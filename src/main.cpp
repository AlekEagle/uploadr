#include "util/config/config.hpp"
#include "util/curlyfries/curlyfries.hpp"
#include "util/flags/flags.hpp"
#include <iostream>

const std::string HELP_MSG =
    "Usage: uploadr [OPTIONS] [FILE|-]"
    "\nSpecify a file to upload, use - to read from stdin, or don't specify "
    "anything to upload the clipboard contents."
    "\nOPTIONS:"
    "\n  -h, --help                     Show this help message and exit"
    "\n  -c=CONFIG, --config=CONFIG     Specify the config directory (must be "
    "an absolute path)";

using namespace std;

int main(int argc, char **argv) {
  // Parse the arguments
  Flags args(argc, argv);

  // Check if the help flag was passed, or if no arguments were passed
  if (args.getBool("h") || args.getBool("help")) {
    // Print the help message
    cout << HELP_MSG << endl;

    // Exit
    return 0;
  }

  // Test curlyfries
  curlyfries::CurlyFryPost request("http://localhost:23456");
  request.addHeader("Content-Type: application/json");
  request.setBody("{\"test\": \"test\"}");
  // Make the request and get the response
  ostringstream response = request.fetch();
  // Print the response
  cout << response.str() << endl;
  return 0;
}