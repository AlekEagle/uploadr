#include "curlyfries.hpp"
#include <curlpp/Easy.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <iostream>
#include <sstream>
#include <string>

namespace curlyfries {
  CurlyFry::CurlyFry(std::string url, std::string method) {
    // Initialize curlpp
    cURLpp::initialize();
    // Set the request URL
    request.setOpt(new cURLpp::Options::Url(url));
    // Set the request method
    request.setOpt(new cURLpp::Options::CustomRequest(method));
  }

  CurlyFry::~CurlyFry() {
    // Cleanup curlpp
    cURLpp::terminate();
  }

  void CurlyFry::addHeader(std::string header) {
    // Add the header to the headers list
    headers.push_back(header);
  }

  void CurlyFry::clearHeaders() {
    // Clear the headers list
    headers.clear();
  }

  void CurlyFry::setBody(std::string body) {
    // Set the request body
    request.setOpt(new cURLpp::Options::PostFields(body));
    // Set the request body size
    request.setOpt(new cURLpp::Options::PostFieldSize(body.length()));
  }

  void CurlyFry::setMethod(std::string method) {
    // Set the request method
    request.setOpt(new cURLpp::Options::CustomRequest(method));
  }

  std::ostringstream CurlyFry::fetch() {
    // Set the request headers
    request.setOpt(new cURLpp::Options::HttpHeader(headers));
    // Tell curlpp to write the response to a stream
    std::ostringstream responseStream;
    request.setOpt(new cURLpp::Options::WriteStream(&responseStream));
    // Make the request
    request.perform();
    // Return the response stream
    return responseStream;
  }

  CurlyFryPost::CurlyFryPost(std::string url) : CurlyFry(url, "POST") {
  }

} // namespace curlyfries