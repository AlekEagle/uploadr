#pragma once
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <iostream>
#include <list>
#include <sstream>
#include <string>

namespace curlyfries {
  class CurlyFry {
    protected:
      // The request
      cURLpp::Easy request;
      // Headers
      std::list<std::string> headers;

    public:
      CurlyFry(std::string url, std::string method = "GET");
      ~CurlyFry();

      // Add a header
      void addHeader(std::string header);
      // Clear the headers
      void clearHeaders();
      // Set the request body
      void setBody(std::string body);
      // Set the request method
      void setMethod(std::string method);
      // Make the request and return the stream containing the response
      std::ostringstream fetch();
  };

  class CurlyFryPost : public CurlyFry {
    public:
      CurlyFryPost(std::string url);
  };
} // namespace curlyfries