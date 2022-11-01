#include "flags.hpp"
#include <iostream>
#include <string>

Flags::Flags(int argc, char **argv) {
  argCount = 0;
  // Begin parsing the arguments
  /*
  How the arguments should be parsed:
    Short tail flags can be combined, e.g. -abc is the same as -a -b -c.
    A value can be specified for an argument, e.g. -a=value or
    --flag=value.
    If a flag is specified multiple times, the last value is used.
    If a flag is specified without a value, the value is set to true.
    If chained short tail flag are specified with a value, the value is
    applied to the last flag, e.g. -abc=value is the same as -a -b -c=value.
    If there's an argument without a flag, it's added to the arguments list.
  */

  // Loop through the arguments
  for (int i = 1; i < argc; i++) {
    // Get the current argument
    std::string arg = argv[i];

    // Check if the argument is a flag
    if (arg[0] == '-') {
      // Check if the argument is a long tail flag
      if (arg[1] == '-') {
        // Get the flag name
        std::string flag = arg.substr(2, arg.find('=') - 2);

        // Check if the flag has a value
        if (arg.find('=') != std::string::npos) {
          // Get the flag value
          std::string value = arg.substr(arg.find('=') + 1);

          // Set the flag value
          flagMap[flag] = value;
        } else {
          // Set the flag value to true
          flagMap[flag] = "true";
        }
      } else {
        // Check if the short tail flag is chained
        if (arg.length() > 2 && arg[2] != '=') {
          // Loop through the chained flags
          for (int j = 1; j < arg.length(); j++) {
            // Get the flag name
            std::string flag = arg.substr(j, 1);

            // Check if the flag is the last in the chain
            if (j == arg.length() - 1 || arg[j + 1] == '=') {
              // Check if the flag has a value
              if (arg.find('=') != std::string::npos) {
                // Get the flag value
                std::string value = arg.substr(arg.find('=') + 1);

                // Set the flag value
                flagMap[flag] = value;
                // Break the loop
                break;
              } else {
                // Set the flag value to true
                flagMap[flag] = "true";
                // Break the loop
                break;
              }
            } else {
              // Set the flag value to true
              flagMap[flag] = "true";
            }
          }
        } else {
          // Get the flag name
          std::string flag = arg.substr(1, 1);

          // Check if the flag has a value
          if (arg.find('=') != std::string::npos) {
            // Get the flag value
            std::string value = arg.substr(arg.find('=') + 1);

            // Set the flag value
            flagMap[flag] = value;
          } else {
            // Set the flag value to true
            flagMap[flag] = "true";
          }
        }
      }
    } else {
      // Add the argument to the arguments list
      arguments[argCount] = argv[i];
      argCount++;
    }
  }
}

Flags::~Flags() {
}

std::string Flags::operator[](std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return flagMap[flag];
  } else {
    // Return an empty string
    return "";
  }
}

std::string Flags::operator[](const char *flag) {
  std::string flagString = flag;

  // Use the string version of the operator
  return operator[](flagString);
}

char **Flags::getArguments() {
  return arguments;
}

int Flags::getArgCount() {
  return argCount;
}

std::string Flags::getString(std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return flagMap[flag];
  } else {
    // Return an empty string
    return "";
  }
}

std::string Flags::getString(const char *flag) {
  std::string flagString = flag;

  // Use the string version of the function
  return getString(flagString);
}

int Flags::getInt(std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return std::stoi(flagMap[flag]);
  } else {
    // Return 0
    return 0;
  }
}

int Flags::getInt(const char *flag) {
  std::string flagString = flag;

  // Use the string version of the function
  return getInt(flagString);
}

float Flags::getFloat(std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return std::stof(flagMap[flag]);
  } else {
    // Return 0
    return 0;
  }
}

float Flags::getFloat(const char *flag) {
  std::string flagString = flag;

  // Use the string version of the function
  return getFloat(flagString);
}

double Flags::getDouble(std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return std::stod(flagMap[flag]);
  } else {
    // Return 0
    return 0;
  }
}

double Flags::getDouble(const char *flag) {
  std::string flagString = flag;

  // Use the string version of the function
  return getDouble(flagString);
}

bool Flags::getBool(std::string flag) {
  // Check if the flag exists
  if (flagMap.find(flag) != flagMap.end()) {
    // Return the flag's value
    return flagMap[flag] == "true";
  } else {
    // Return false
    return false;
  }
}

bool Flags::getBool(const char *flag) {
  std::string flagString = flag;

  // Use the string version of the function
  return getBool(flagString);
}