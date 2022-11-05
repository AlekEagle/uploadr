# Uploadr

Uploadr is a simple CLI tool made to be a drop-in replacement for the now deprecated [sharenix](https://github.com/Francesco149/sharenix).

## Dependencies

- [clip](https://github.com/dacap/clip)
- [curl](https://curl.haxx.se/)
- [curlpp](https://www.curlpp.org/)
- [jsoncons](https://github.com/danielaparker/jsoncons)
- [libnotify](https://developer.gnome.org/libnotify/)
- [libpng](http://www.libpng.org/pub/png/libpng.html)
- [x11-dev](https://www.x.org/wiki/)

## Building

### Debian/Ubuntu based distros

```bash
sudo apt install libcurlpp-dev libcurl4-openssl-dev libjsoncpp-dev libnotify-dev libpng-dev libx11-dev cmake build-essential git
```

### Arch Linux

```bash
sudo pacman -S curl jsoncpp libnotify libpng xorgproto cmake make git
# libcurlpp is not available in the official repos, so you'll have to get it from the AUR
# using your favorite AUR helper (e.g. paru -S libcurlpp)
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

## Installation

You can't (yet).

## TODO

- [ ] Finish uploadr
- [ ] Make a PKGBUILD for MPR
- [ ] Make a PKGBUILD for AUR (If it's possible to have a PKGBUILD for both MPR and AUR)
- [x] Include a default uploader (imgur)
- [x] Implement custom uploader syntax (similar to ShareX's)
