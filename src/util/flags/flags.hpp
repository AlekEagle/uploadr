#pragma once
#include <map>
#include <string>

class Flags {
  private:
    // A map of flags and their values
    std::map<std::string, std::string> flagMap;

  public:
    // Create a new instance of the flags class.
    Flags(int argc, char **argv);
    // Close and destroy the flags class.
    ~Flags();

    // A list of arguments
    std::map<int, std::string> arguments;
    // Returns the value of the specified flag. If the flag does not exist,
    // returns empty string.
    std::string operator[](std::string flag);
    std::string operator[](const char *flag);
    // Get the value of a flag as a string.
    std::string getString(std::string flag);
    std::string getString(const char *flag);
    // Get the value of a flag as an integer.
    int getInt(std::string flag);
    int getInt(const char *flag);
    // Get the value of a flag as a float.
    float getFloat(std::string flag);
    float getFloat(const char *flag);
    // Get the value of a flag as a double.
    double getDouble(std::string flag);
    double getDouble(const char *flag);
    // Get the value of a flag as a boolean.
    bool getBool(std::string flag);
    bool getBool(const char *flag);
    // Check if a flag exists.
    bool exists(std::string flag);
    bool exists(const char *flag);
};
