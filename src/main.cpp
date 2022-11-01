#include "util/config/config.hpp"
#include "util/flags/flags.hpp"
#include <iostream>

const std::string HELP_MSG =
    "Usage: uploadr [OPTIONS] [FILE]"
    "\n OPTIONS:"
    "\n  -h, --help    Show this help message and exit";

using namespace std;

int main(int argc, char **argv) {
  // Parse the arguments
  Flags args(argc, argv);

  // Check if the help flag was passed, or if no arguments were passed
  if (args.getBool("h") || args.getBool("help") || argc == 1) {
    // Print the help message
    cout << HELP_MSG << endl;

    // Exit
    return 0;
  }
}