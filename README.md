# Uploadr

Uploadr - Rust Edition.

Uploadr is a simple tool to upload files to a host of your choice, and then copy the URL to your clipboard for easy sharing. Written in Rust, it's fast, lightweight, and NOT cross-platform (Windows already has an all-in-one tool for this, and macOS is lame).

Previously written in C++, this is a rewrite in Rust. It's not as feature-complete as the C++ version, but it should be more stable and easier to maintain. It's also a lot faster probably.

## Roadmap

- [x] Upload stuff to things
  - [x] Configurable HTTP request method
  - [x] Configurable HTTP request headers
  - [x] Configurable HTTP request body
    - [x] multipart/form-data
    - [x] application/octet-stream
  - [x] Response Parsing
    - [x] JSON
    - [x] XML
    - [ ] URL-encoded
    - [x] Reg-exp matching
- [x] Clipboard stuff and things
  - [x] Copy to clipboard
  - [x] Upload from clipboard
    - [x] Binary data (images, etc.)
    - [x] Text data
    - [ ] File paths (file://, absolute paths) (arboard is currently very restrictive about the types of data it supports, and on some DEs, copying a file does not copy a file:// URL in text/plain format, so this is not currently possible)
- [x] Configuration stuff and things
  - [x] Default target
  - [x] Generate default config file
  - [x] Multiple upload targets
  - [x] Templating for upload target configurations.
    - [x] File related templates
      - [x] File name
      - [x] File extension
      - [x] File size
      - [x] File MIME type
    - [x] Environment variable templates
    - [x] Response related templates
      - [x] Response body
      - [x] Response headers
      - [x] Response status code
      - [x] Response URL
    - [x] Randomness related templates
      - [x] Random string
      - [x] Random number
        - [x] Integer
        - [x] Float
      - [x] Random UUID
      - [x] Random choice
    - [x] Config related templates
    - [x] Uploader config related templates
- [x] Notifications stuff and things
  - [x] Upload progress
  - [x] Upload complete
    - [x] Open URL in browser
    - [x] Open management page
    - [ ] Display upload preview (if applicable)
  - [ ] Upload failed
- [x] Upload history stuff and things
- [ ] GUI stuff and things?!?!?!?!?!? (probably not)

## Moving from v1

The configuration file format has changed slightly from the C++ version. Removing some unnecessary complexity, and adding some new features. The new format is as follows:

### Primary Config file

Old:

```json
{
  "defaultUploader": "string",
  "archive": {
    "enabled": true,
    "path": "/dev/null",
    "maxCount": 1000,
    "histFile": "string"
  },
  "clipboard": {
    "enabled": true
  },
  "notification": {
    "enabled": "bool",
    "sound": "string",
    "timeout": 5
  }
}
```

New:

```json
{
  "default_uploader": "uploader", // Now snake_case
  "archive": {
    "enabled": true,
    "path": "/dev/null", // Must be an absolute path
    "max": 1000 // Renamed to max
    // Hist file is in the same directory as the config file, so it's not necessary to specify it.
  },
  "clipboard": {
    "enabled": true,
    "read_only": false // New option to disable setting the clipboard
  },
  "notification": {
    "enabled": true,
    "timeout": 5 // Timeout in seconds
    // Sound is not used in the Rust version.
  }
}
```

### Uploader Config file

Individual uploaders are configured in their own files, and are named "{Uploader name}.uploader", this has not changed.

Old:

```json
{
  "request": {
    // MultipartFormData body
    "body": {
      "fields": {
        "field_name": "stuff",
        "other_field_name": "{content}" // The content that should be uploaded
      },
      "type": "MultipartFormData"
    },
    // Other body types are omitted for brevity
    "headers": {
      "Header": "Value"
    },
    "method": "POST",
    "url": "http://example.com"
  },
  "response": {
    "url": "stuff",
    "manageUrl": "more_stuff",
    "thumbnailUrl": "the_most_stuff"
  }
}
```

New:

```json
{
  "request": {
    "body": {
      "field": "file",
      "type": "MultipartFormData" // Can be MultipartFormData or Raw, Raw has no configurable fields.
    },
    "headers": {
      "Header": "Value"
    },
    "method": "POST",
    "url": "https://example.com"
  },
  "response": {
    "url": "stuff",
    "manage_url": "more_stuff",
    "thumbnail_url": "the_most_stuff"
  }
}
```

## Configuration Templating

Uploadr supports templating in the uploader configuration files. This allows you to use variables in your configuration that will be replaced the appropriate values at runtime. Formatted as `{ directive:parameter;...parameters }`, templates can also be nested inside each other, allowing for complex configurations.

Unclosed templates and invalid directives will cause uploadr to panic with an error message.

### Directives

- Configuration related directives

  - `config:path` - Get a value from the primary configuration file. It takes a JSON path and returns the value at that path. It must refer to only a single value, and cannot be an array or object.
  - `uploader:path` - Get a value from the uploader configuration file. It takes a JSON path and returns the value at that path. It must refer to only a single value, and cannot be an array or object.

- `env:parameter` - Get an environment variable. It takes a single parameter, which is the name of the environment variable to get. If the environment variable is not set, uploadr will panic with an error message.

- `random:type;...parameters`
  - `random:int;min;max` - Get a random integer between `min` and `max` (inclusive). `min` and `max` must be integers.
  - `random:float;min;max` - Get a random float between `min` and `max` (inclusive). `min` and `max` must be floats.
  - `random:choice;...choices` - Get a random choice from the provided choices. Choices must be separated by semicolons.
  - `random:uuid` - Get a random v4 UUID.
  - `random:string;length;alphabet` - Get a random string of the specified length. `length` must be an integer. `alphabet` is optional, and if not provided, defaults to `abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789`. It can be any string of characters, and can be any length.
- File related directives
  - `file:name` - Get the name of the file being uploaded.
  - `file:ext` - Get the extension of the file being uploaded.
  - `file:size` - Get the size of the file being uploaded.
  - `file:mime` - Get the MIME type of the file being uploaded.
- Response related directives (these are only available in the `response` section of the uploader config)
  - `response:body`
    - `response:body;json;path` (A.K.A. `json:path`) - Get a value from the response body. It takes a JSON path and returns the value at that path. It must refer to only a single value, and cannot be an array or object.
    - `response:body;xml;path` (A.K.A. `xml:path`) - Get a value from the response body. It takes an XPath and returns the value at that path. It must refer to only a single value, and cannot be an array or object. (untested but might work)
    - `response:body;regexp;pattern` (A.K.A. `regexp:pattern`) - Get a value from the response body. It takes a regular expression and returns the first match. It must refer to only a single value, and cannot be an array or object. (untested but might work)
  - `response:headers;header` - Get a value from the response headers. It takes a header name and returns the value of that header. Case-insensitive.
  - `response:status_code` - Get the status code of the response.
  - `response:url` - Get the final URL of the response. This is useful if the server redirects the request.
