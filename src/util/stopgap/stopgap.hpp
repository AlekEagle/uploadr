#pragma once
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

// StopgapFile: a temporary file created by Stopgap
class StopgapFile {
  private:
    std::filesystem::path path;
    bool deleteOnDestruct;

  public:
    StopgapFile(std::filesystem::path path, bool deleteOnDestruct = true)
        : path(path), deleteOnDestruct(deleteOnDestruct) {
    }
    std::filesystem::path getPath() {
      return path;
    }

    void setExtension(std::string ext);

    void pipe(std::ostream &stream);
    void pipe(std::ofstream &stream);
    void pipe(std::ostringstream &stream);

    void pipe(const std::istream &stream);
    void pipe(const std::ifstream &stream);
    void pipe(const std::istringstream &stream);

    // Test for a file extension
    std::string testExtension();

    // Get the contents of the file as a char vector
    std::vector<char> getContents();

    // Write a char vector to the file
    void writeContents(std::vector<char> contents);

    // Delete the copy constructor and assignment operator
    // (we don't want to copy the file)
    StopgapFile(const StopgapFile &) = delete;
    StopgapFile &operator=(const StopgapFile &) = delete;

    // Use the destructor to delete the file
    ~StopgapFile() {
      if (deleteOnDestruct) {
        std::filesystem::remove(path);
      }
    }
};

// Stopgap: a temporary file management system
class Stopgap {
  private:
    std::filesystem::path base;
    std::string dir;

  public:
    Stopgap()
        : base(std::filesystem::temp_directory_path()),
          dir(std::to_string(std::rand())) {
    }

    Stopgap(std::filesystem::path base)
        : base(base), dir(std::to_string(std::rand())) {
    }

    Stopgap(std::filesystem::path base, std::string dir)
        : base(base), dir(dir) {
    }
    StopgapFile createFile();
    StopgapFile createFile(std::string extension);

    // Delete the copy constructor and assignment operator
    Stopgap(const Stopgap &) = delete;
    Stopgap &operator=(const Stopgap &) = delete;

    // Delete the stopgap directory and all files in it when the object is
    // destroyed
    ~Stopgap() {
      std::filesystem::remove_all(base / dir);
    }
};