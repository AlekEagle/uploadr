# Uploadr

Uploadr is a simple CLI tool made to be a drop-in replacement for the now deprecated [sharenix](https://github.com/Francesco149/sharenix).

## Compatible with

### Built and tested on

- KDE neon (It's essentially Ubuntu 22.04 LTS)
- Arch Linux

### Untested but should generally work with

- Any Debian/Ubuntu based Linux distribution
- Most UNIX systems
- macOS (maybe? you'd probably have to tweak some dependencies or something. Personally I wouldn't trust it being very stable)

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

## Building

### Debian/Ubuntu based distros

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

```bash
# Clone the repo and the submodules and cd into it
git clone --recursive https://github.com/AlekEagle/uploadr.git && cd uploadr
# Create a build directory and cd into it
mkdir build && cd build
cmake ..
make
```

If you want to install it, run `sudo make install`.

## TODO

- [x] Completely utility functionality of uploadr (The uploading part)
- [x] Use templating strings for upload requests and responses
- [x] Upload files specified from command line
- [x] Determine the content type and upload data from stdin
- [x] Determine the content type and upload data from the clipboard
- [x] Put response URL on the clipboard after successful upload
- [x] Have a greppable history of uploaded files
- [x] Automatically delete old files from history (configurable)
- [ ] Display notification of upload status
- [ ] Display notification when an upload is successful
- [ ] Successful upload notifications have action buttons to open the various URLs returned from successful upload responses.
- [ ] Display a thumbnail of the uploaded file using thumbnail URL returned from upload (if present)
- [ ] Make a PKGBUILD for MPR
- [ ] Make a PKGBUILD for AUR (If it's possible to have a PKGBUILD for both MPR and AUR)
