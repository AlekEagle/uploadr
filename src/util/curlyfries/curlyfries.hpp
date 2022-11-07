#pragma once
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <iostream>
#include <jsoncons/json.hpp>
#include <list>
#include <map>
#include <pugixml.hpp>
#include <sstream>
#include <string>
#include <vector>

namespace curlyfries {
  struct Response {
      bool ready = false;
      std::ostringstream body;
      int status;
      std::map<std::string, std::string> headers;
      std::string redirectedUrl;
  };

  class CurlyFry {
    protected:
      // The request
      cURLpp::Easy request;
      // Headers
      std::list<std::string> reqHeaders;
      // The response
      Response response;

    public:
      static std::string escape(std::string str) {
        return curlpp::escape(str);
      }

      static std::string unescape(std::string str) {
        return curlpp::unescape(str);
      }
      CurlyFry();
      CurlyFry(std::string url, std::string method = "GET");
      ~CurlyFry();

      void setUrl(std::string url);
      // Add a header
      void addHeader(std::string header);
      // Add a header (key and value pair style)
      void addHeader(std::string key, std::string value);
      // Add a header (key and list of values style)
      void addHeader(std::string key, std::list<std::string> values);
      // Clear the reqHeaders
      void clearHeaders();
      // Set the request body (assume text/plain)
      void setBody(std::string body);
      // Set the request body (with content type)
      void setBody(std::string body, std::string contentType);
      // Set the request body (using multipart/form-data)
      void setBody(cURLpp::Forms form);
      // Set the request body (using JSON object)
      void setBody(jsoncons::json json);
      // Set the request body (using XML document)
      void setBody(pugi::xml_document &xml);
      // Set the request body (using binary input stream)
      // assume content type is application/octet-stream
      void setBody(std::istream &stream);
      // Set the request body (using binary input stream and content type)
      void setBody(std::istream &stream, std::string contentType);
      // Set the request body (using vector char buffer)
      // assume content type is application/octet-stream
      void setBody(std::vector<char> &buffer);
      // Set the request body (using vector char buffer and content type)
      void setBody(std::vector<char> &buffer, std::string contentType);
      // Clear the request body
      // WARNING: This will also clear any reqHeaders that were set
      void clearBody();
      // Set the request method
      void setMethod(std::string method);
      // Send the request and return the status code
      int send();
      // Get the response struct
      Response *getResponse();
  };
} // namespace curlyfries