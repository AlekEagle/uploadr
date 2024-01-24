# Uploadr

Uploadr - Rust Edition.

Uploadr is a simple tool to upload files to a host of your choice, and then copy the URL to your clipboard for easy sharing. Written in Rust, it's fast, lightweight, and NOT cross-platform (Windows already has an all-in-one tool for this, and macOS is lame).

Previously written in C++, this is a rewrite in Rust. It's not as feature-complete as the C++ version, but it should be more stable and easier to maintain. It's also a lot faster probably.

## Roadmap

- [ ] Upload stuff to things
  - [x] Configurable HTTP request method
  - [x] Configurable HTTP request headers
  - [x] Configurable HTTP request body
    - [x] multipart/form-data
    - [x] application/octet-stream
  - [x] Response Parsing
    - [x] JSON
    - [ ] XML
    - [ ] URL-encoded
    - [ ] Reg-exp matching
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
- [ ] Notifications stuff and things
  - [ ] Upload progress
  - [ ] Upload complete
    - [ ] Open URL in browser
    - [ ] Open management page
    - [ ] Display upload preview (if applicable)
  - [ ] Upload failed
- [ ] Upload history stuff and things
- [ ] GUI stuff and things?!?!?!?!?!? (probably not)
