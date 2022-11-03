#include "curlyfries.hpp"
#include <curlpp/Easy.hpp>
#include <curlpp/Infos.hpp>
#include <curlpp/Options.hpp>
#include <curlpp/cURLpp.hpp>
#include <iostream>
#include <sstream>
#include <string>

namespace curlyfries {
  CurlyFry::CurlyFry() {
    // Initialize curlpp
    cURLpp::initialize();
    // Set the response stream
    request.setOpt(new cURLpp::Options::WriteStream(&response));
  }

  CurlyFry::CurlyFry(std::string url, std::string method) {
    // Initialize curlpp
    cURLpp::initialize();
    // Set the response stream
    request.setOpt(new cURLpp::Options::WriteStream(&response));
    // Set the request URL
    request.setOpt(new cURLpp::Options::Url(url));
    // Set the request method
    request.setOpt(new cURLpp::Options::CustomRequest(method));
  }

  CurlyFry::~CurlyFry() {
    // Cleanup curlpp
    cURLpp::terminate();
  }

  void CurlyFry::setUrl(std::string url) {
    request.setOpt(new cURLpp::Options::Url(url));
  }

  void CurlyFry::addHeader(std::string header) {
    // Add the header to the headers list
    headers.push_back(header);
  }

  void CurlyFry::addHeader(std::string key, std::string value) {
    // Add the header to the headers list
    headers.push_back(key + ": " + value);
  }

  void CurlyFry::addHeader(std::string key, std::list<std::string> values) {
    // Add the header to the headers list
    std::string header = key + ": ";
    for (std::string value : values) {
      header += value + ", ";
    }
    header.pop_back();
    header.pop_back();
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
    // Set the content type (assume text/plain)
    this->addHeader("Content-Type", "text/plain");
  }

  void CurlyFry::setBody(std::string body, std::string contentType) {
    // Set the request body
    request.setOpt(new cURLpp::Options::PostFields(body));
    // Set the request body size
    request.setOpt(new cURLpp::Options::PostFieldSize(body.length()));
    // Set the content type
    this->addHeader("Content-Type", contentType);
  }

  void CurlyFry::setBody(cURLpp::Forms form) {
    // Set the request body
    request.setOpt(new cURLpp::Options::HttpPost(form));
    // We don't need to set the content type because cURLpp does it for us
  }

  void CurlyFry::setBody(jsoncons::json json) {
    // Set the request body
    request.setOpt(new cURLpp::Options::PostFields(json.as_string()));
    // Set the request body size
    request.setOpt(new cURLpp::Options::PostFieldSize(json.as_string().length())
    );
    // Set the content type
    this->addHeader("Content-Type", "application/json");
  }

  void CurlyFry::setBody(pugi::xml_document xml) {
    // Take the XML document and convert it to a string
    std::ostringstream xmlStream;
    xml.save(xmlStream);
    // Set the request body
    request.setOpt(new cURLpp::Options::PostFields(xmlStream.str()));
    // Set the request body size
    request.setOpt(new cURLpp::Options::PostFieldSize(xmlStream.str().length())
    );
    // Set the content type
    this->addHeader("Content-Type", "application/xml");
  }

  void CurlyFry::setBody(std::istream &stream) {
    // Set the request body
    request.setOpt(new cURLpp::Options::ReadStream(&stream));
    // Set the content type (assume application/octet-stream)
    this->addHeader("Content-Type", "application/octet-stream");
  }

  void CurlyFry::setBody(std::istream &stream, std::string contentType) {
    // Set the request body
    request.setOpt(new cURLpp::Options::ReadStream(&stream));
    // Set the content type
    this->addHeader("Content-Type", contentType);
  }

  void CurlyFry::clearBody() {
    // Clear the request body
    request.setOpt(new cURLpp::Options::PostFields());
    // Clear the request body size
    request.setOpt(new cURLpp::Options::PostFieldSize(0));
    // Clear all headers (we can't exclusively clear the content type)
    this->clearHeaders();
  }

  void CurlyFry::setMethod(std::string method) {
    // Set the request method
    request.setOpt(new cURLpp::Options::CustomRequest(method));
  }

  int CurlyFry::send() {
    // Set the headers
    request.setOpt(new cURLpp::Options::HttpHeader(headers));
    // Send the request
    request.perform();
    // Return the status code
    return cURLpp::infos::ResponseCode::get(request);
  }

  std::ostringstream *CurlyFry::getResponse() {
    // Return the response stream
    return &response;
  }

} // namespace curlyfries