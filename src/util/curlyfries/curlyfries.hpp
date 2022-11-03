#pragma once
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <iostream>
#include <jsoncons/json.hpp>
#include <list>
#include <pugixml.hpp>
#include <sstream>
#include <string>

namespace curlyfries {
  class CurlyFry {
    protected:
      // The request
      cURLpp::Easy request;
      // Headers
      std::list<std::string> headers;
      // Response
      std::ostringstream response;

    public:
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
      // Clear the headers
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
      void setBody(pugi::xml_document xml);
      // Set the request body (using binary input stream)
      // assume content type is application/octet-stream
      void setBody(std::istream &stream);
      // Set the request body (using binary input stream and content type)
      void setBody(std::istream &stream, std::string contentType);
      // Clear the request body
      // WARNING: This will also clear any headers that were set
      void clearBody();
      // Set the request method
      void setMethod(std::string method);
      // Send the request and return the status code
      int send();
      // Get the raw response stream
      std::ostringstream *getResponse();
      // Get the response as a string
      std::string getResponseString();
      // Get the response as a JSON object
      jsoncons::json getResponseJSON();
      // Get the response as an XML document
      pugi::xml_document getResponseXML();
      // Get response headers
      std::list<std::string> getResponseHeaders();
  };
} // namespace curlyfries