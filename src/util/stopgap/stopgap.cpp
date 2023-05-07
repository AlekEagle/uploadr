#include "stopgap.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>
#include <time.h>
#include <vector>

// Stopgap: a temporary file management system
StopgapFile Stopgap::createFile() {
  // Seed std::rand
  std::srand(time(NULL));
  // Create a random filename for the file
  std::string filename = std::to_string(std::rand());
  // Create the path to the file
  std::filesystem::path path = base / dir / filename;
  // Make sure the directory exists
  std::filesystem::create_directories(path.parent_path());
  // Create the file
  std::ofstream file(path);
  file.close();
  // Return the file
  return StopgapFile(path);
}
StopgapFile Stopgap::createFile(std::string extension) {
  // Create a random filename for the file
  std::string filename = std::to_string(std::rand());
  // Create the path to the file
  std::filesystem::path path = base / dir / (filename + "." + extension);
  // Make sure the directory exists
  std::filesystem::create_directories(path.parent_path());
  // Create the file
  std::ofstream file(path);
  file.close();
  // Return the file
  return StopgapFile(path);
}

// StopgapFile: a temporary file created by Stopgap

void StopgapFile::pipe(std::ostream &stream) {
  // Open the file
  std::ifstream file(path);
  // Write the data to the stream
  stream << file.rdbuf();
  // Close the file
  file.close();
}

void StopgapFile::pipe(std::ofstream &stream) {
  // Open the file
  std::ifstream file(path);
  // Write the data to the stream
  stream << file.rdbuf();
  // Close the file
  file.close();
}

void StopgapFile::pipe(std::ostringstream &stream) {
  // Open the file
  std::ifstream file(path);
  // Write the data to the stream
  stream << file.rdbuf();
  // Close the file
  file.close();
}

void StopgapFile::pipe(const std::istream &stream) {
  // Open the file
  std::ofstream file(path);
  // Write the data to the file
  file << stream.rdbuf();
  // Close the file
  file.close();
}

void StopgapFile::pipe(const std::ifstream &stream) {
  // Open the file
  std::ofstream file(path);
  // Write the data to the file
  file << stream.rdbuf();
  // Close the file
  file.close();
}

void StopgapFile::pipe(const std::istringstream &stream) {
  // Open the file
  std::ofstream file(path);
  // Write the data to the file
  file << stream.rdbuf();
  // Close the file
  file.close();
}

std::vector<char> StopgapFile::getContents() {
  // Open the file
  std::ifstream file(path);
  // Get the file size
  file.seekg(0, std::ios::end);
  std::streampos size = file.tellg();
  file.seekg(0, std::ios::beg);
  // Create a char vector to store the file contents
  std::vector<char> contents(size);
  // Read the file contents
  file.read(contents.data(), size);
  // Close the file
  file.close();
  // Return the file contents
  return contents;
}

void StopgapFile::writeContents(std::vector<char> contents) {
  // Open the file
  std::ofstream file(path);
  // Write the file contents
  file.write(contents.data(), contents.size());
  // Close the file
  file.close();
}