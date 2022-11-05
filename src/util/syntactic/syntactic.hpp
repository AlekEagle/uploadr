// A set of classes and functions for finding, parsing, and validating template
// strings in a string.
#pragma once
#include "../curlyfries/curlyfries.hpp"
#include <list>
#include <string>

namespace Syntactic {
  const char START = '{';
  const char END = '}';
  const char ESCAPE = '\\';
  const char PARAM_START = ':';
  const char PARAM_DELIM = '|';

  // Syntactic::Data
  // A struct containing the data for all necessary information to fully
  // parse a template string.
  struct Data {
      // The name of the file we're uploading (if any)
      std::string fileName;
      // The path to the file we're uploading (if any)
      std::string filePath;
      // The response from the server after uploading the file (if any)
      curlyfries::Response *response;
  };

  // The base syntactic class
  class SyntacticBase {
    protected:
      SyntacticBase(){};
      virtual std::string
      call(std::string name, std::list<std::string> params) = 0;

    private:
      std::string parse(std::string text, bool isFunction, int start, int &end);

    public:
      virtual std::string parse(std::string text);
  };

  // The syntactic class to be used for parsing template strings
  class Syntactic : public SyntacticBase {
    private:
      Data data;

    protected:
      std::string call(std::string name, std::list<std::string> params);

    public:
      Syntactic(Data data);
  };

  // The base syntactic error class
  class SyntacticError : public std::exception {
    protected:
      std::string message;

    public:
      SyntacticError(std::string message);
      virtual const char *what() const throw();
  };

  // The syntactic error class for empty function names
  class EmptyFunctionNameError : public SyntacticError {
    public:
      EmptyFunctionNameError();
  };

  // The syntactic error class for invalid function names
  class UnknownFunctionError : public SyntacticError {
    public:
      UnknownFunctionError(std::string name);
  };

  // The syntactic error for missing function parameters
  class MissingFunctionParameterError : public SyntacticError {
    public:
      MissingFunctionParameterError(std::string name, int given, int expected);
  };

  // The syntactic error for no parameters given to the random function
  class NoRandomParametersError : public SyntacticError {
    public:
      NoRandomParametersError();
  };

  // The syntactic error for using response functions when the response is not
  // ready
  class ResponseNotReadyError : public SyntacticError {
    public:
      ResponseNotReadyError();
  };
} // namespace Syntactic