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
    - [ ] application/x-www-form-urlencoded
    - [ ] application/json
    - [ ] text/plain
    - [ ] application/octet-stream
  - [x] Response Parsing
    - [x] JSON
    - [ ] XML
    - [ ] URL-encoded
    - [ ] Reg-exp matching
- [ ] Clipboard stuff and things
  - [ ] Copy to clipboard
  - [ ] Upload from clipboard
    - [ ] Binary data (images, etc.)
    - [ ] Text data
    - [ ] File paths (file://, absolute paths)
- [x] Configuration stuff and things
  - [x] Default target
  - [x] Generate default config file
  - [x] Multiple upload targets
- [ ] Notifications stuff and things
  - [ ] Upload progress
  - [ ] Upload complete
    - [ ] Open URL in browser
    - [ ] Open management page
    - [ ] Display upload preview (if applicable)
  - [ ] Upload failed
- [ ] Upload history stuff and things
- [ ] GUI stuff and things?!?!?!?!?!? (probably not)
