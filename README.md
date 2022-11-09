# Uploadr

Uploadr is a simple CLI tool made to be a drop-in replacement for the now deprecated [sharenix](https://github.com/Francesco149/sharenix).

## Compatible With

### Built and Tested on

- KDE neon (It's essentially Ubuntu 22.04 LTS)
- Arch Linux

### Untested but should generally work with

- Any Debian/Ubuntu based Linux distribution
- Most UNIX systems
- macOS (maybe? notifications won't work and you'd probably have to tweak some dependencies or something. Personally I wouldn't trust it being very stable)

The only strict requirements is that you must be using X11 (Wayland is unsupported and won't be actively maintained by me, feel free to contribute though!)

## NOT Compatible With

- Windows

## Dependencies

- [clip](https://github.com/dacap/clip)
- [curl](https://curl.haxx.se/)
- [curlpp](https://www.curlpp.org/)
- [jsoncons](https://github.com/danielaparker/jsoncons)
- [libnotify](https://developer.gnome.org/libnotify/)
- [libpng](http://www.libpng.org/pub/png/libpng.html)
- [x11-dev](https://www.x.org/wiki/)
- [libmagic](https://www.darwinsys.com/file/)

## Installing Dependencies

To build uploadr we first have to get everything ready. Let's install the build dependencies, most runtime dependencies are bundled with uploadr when you recursively pull, so you shouldn't have to worry about those.

### Debian/Ubuntu Based Distros

```bash
sudo apt install libcurlpp-dev libcurl4-openssl-dev libjsoncpp-dev libnotify-dev libpng-dev libx11-dev libmagic-dev cmake build-essential git
```

### Arch Linux

```bash
sudo pacman -S curl jsoncpp libnotify libpng xorgproto libmagic cmake make git
# libcurlpp is not available in the official repos, so you'll have to get it from the AUR
# using your favorite AUR helper, here's an example with paru:
paru -S libcurlpp
```

## Building

Clone the repository recursively, to get additional dependencies that we don't (and probably can't (unless you're an Arch user with the AUR (I'm looking at you random stranger using Arch))) need to install from package repositories, make a build directory, configure cmake, and build uploadr.

```bash
# Clone the repo and the submodules and cd into it
git clone --recursive https://github.com/AlekEagle/uploadr.git && cd uploadr
# Create a build directory and cd into it
mkdir build && cd build
# Configure cmake and have cmake build Makefiles
cmake ..
# Build!
make
```

## Installation

If you would like to install uploadr so it can be accessible system-wide, follow these instructions here:

### Built from Source

```bash
# After running make from earlier.
# CWD: uploadr/build/
sudo make install
```

## Uninstallation

If you no longer need or want uploadr to be installed system-side, follow these instructions here:

### Built from Source

```bash
# CWD: uploadr/build/
sudo make uninstall
```

## Updating

### Built from Source

```bash
# CWD: uploadr/
# Pull recursively to keep our other important packages up-to-date
git pull --resursive
# Delete the build directory. This isn't strictly necessary, but might prevent strange build issues.
rm -rf build
# If this fails, you might've ran the make command as root, simply run the command again but as root.
# Make a new clean build directory and enter it
mkdir build && cd build
# Configure cmake and have cmake build Makefiles
cmake ..
# Build!
make
# Update installed binaries and libraries
sudo make install
```

## TODO

- [x] Completely utility functionality of uploadr (The uploading part)
- [x] Use templating strings for upload requests and responses
- [x] Upload files specified from command line
- [x] Determine the content type and upload data from stdin
- [x] Determine the content type and upload data from the clipboard
- [x] Put response URL on the clipboard after successful upload
- [x] Have a greppable history of uploaded files
- [x] Automatically delete old files from history (configurable)
- [x] Display notification of upload status
- [x] Display notification when an upload is successful
- [x] Successful upload notifications have action buttons to open the various URLs returned from successful upload responses.
- [x] Display notification when an upload fails.
- [ ] Display additional information when upload fails.
- [ ] Display a thumbnail of the uploaded file using thumbnail URL returned from upload (if present)
- [ ] Make a PKGBUILD for MPR
- [ ] Make a PKGBUILD for AUR (If it's possible to have a PKGBUILD for both MPR and AUR)
