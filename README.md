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
    // history.csv and the uploads/ folder are stored inside this path, so it's not necessary to specify a history file.
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

Unclosed templates and invalid directives will cause uploadr to exit with an error message.

`:` separates the directive name from its parameters, and `;` separates parameters. Any `:` after the first one is part of the parameter, so values like `https://example.com:8080` work as-is. To use `{`, `}`, `;`, `:` or `\` literally, escape them with a backslash (e.g. `\{`, `\;`, `\\`). A backslash before any other character is kept as-is, so regex classes like `\d` don't need escaping. The output of a nested template is used literally, it is not templated again.

Names, numbers, and paths in parameters have surrounding whitespace trimmed, but choices, alphabets, regex patterns, and `env` default values are used exactly as written.

### Directives

- Configuration related directives
  - `config:path` - Get a value from the primary configuration file. It takes a JSON path and returns the value at that path. It must refer to only a single value. Strings are returned without quotes.
  - `uploader:path` - Get a value from the uploader configuration file. It takes a JSON path and returns the value at that path. It must refer to only a single value. Strings are returned without quotes.

- `env:name;default` - Get an environment variable. The first parameter is the name of the environment variable to get. `default` is optional, and is used if the variable is not set. If the environment variable is not set and there is no default, uploadr will exit with an error message.

- `random:type;...parameters`
  - `random:int;min;max` - Get a random integer between `min` and `max` (inclusive). `min` and `max` must be integers (defaulting to 1 and 100), and `min` can't be greater than `max`.
  - `random:float;min;max` - Get a random float from `min` (inclusive) up to `max` (exclusive), or exactly `min` if they're equal. `min` and `max` must be floats (defaulting to 1 and 100), and `min` can't be greater than `max`.
  - `random:choice;...choices` - Get a random choice from the provided choices. Choices must be separated by semicolons, and at least one is required.
  - `random:uuid` - Get a random v4 UUID.
  - `random:string;length;alphabet` - Get a random string of the specified length. `length` must be an integer. `alphabet` is optional, and if not provided, defaults to `abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789`. It can be any string of characters (including Unicode), and can be any non-zero length.
- File related directives
  - `file:name` - Get the name of the file being uploaded.
  - `file:ext` - Get the extension of the file being uploaded.
  - `file:size` - Get the size of the file being uploaded.
  - `file:mime` - Get the MIME type of the file being uploaded.
- Response related directives (these are only available in the `response` section of the uploader config)
  - `response:body`
    - `response:body;json;path` (A.K.A. `json:path`) - Get a value from the response body. It takes a JSON path and returns the value at that path. It must refer to only a single value. Strings are returned without quotes.
    - `response:body;xml;path` (A.K.A. `xml:path`) - Get a value from the response body. The XML is converted to JSON first, so it takes a JSON path (not an XPath) and returns the value at that path. It must refer to only a single value. (untested but might work)
    - `response:body;regex;pattern` (A.K.A. `regex:pattern`) - Get a value from the response body. It takes a regular expression and returns the first capture group of the first match, or the whole match if the pattern has no groups. If the pattern doesn't match, uploadr exits with an error. Escape `{`, `}` and `;` in patterns (e.g. `\d\{3\}`).
  - `response:headers;header` - Get a value from the response headers. It takes a header name and returns the value of that header. Case-insensitive.
  - `response:status_code` - Get the status code of the response.
  - `response:url` - Get the final URL of the response. This is useful if the server redirects the request.
