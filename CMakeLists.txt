# MINIMUM CMAKE VERSION
cmake_minimum_required(VERSION 3.0)

# C++ STANDARD
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# PROJECT
project(uploadr)

# DEPENDENCY RESOLVERS
find_package(PkgConfig REQUIRED)

# DEPENDENCIES
pkg_check_modules(CURL REQUIRED curlpp)
pkg_check_modules(NOTIFY REQUIRED libnotify)
pkg_check_modules(X11 REQUIRED x11)
pkg_check_modules(PNG REQUIRED libpng)
pkg_check_modules(MAGIC REQUIRED libmagic)

# DIRECTORIES
set(SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)
set(INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/include)
set(LIB_DIR ${CMAKE_CURRENT_SOURCE_DIR}/lib)

# SOURCES
file(GLOB_RECURSE UPLOADR_SOURCES CONFIGURE_DEPENDS "${SOURCE_DIR}/*.cpp")

# LIBRARIES

# CLIP
set(CLIP_EXAMPLES OFF CACHE BOOL "Compile clip examples")
set(CLIP_TESTS OFF CACHE BOOL "Compile clip tests")
set(CLIP_X11_PNG_LIBRARY ${PNG_LIBRARIES})
add_subdirectory(${LIB_DIR}/clip)

# PUGIXML
add_subdirectory(${LIB_DIR}/pugixml)

# CPP-BASE64
add_library(cpp-base64 STATIC ${LIB_DIR}/cpp-base64/base64.cpp)

# END OF LIBRARIES

# EXECUTABLE
add_executable(uploadr ${UPLOADR_SOURCES})

# EXECUTABLE LINKING
target_link_libraries(
  uploadr
  clip
  pugixml
  cpp-base64
  ${NOTIFY_LIBRARIES}
  ${CURL_LIBRARIES}
  ${X11_LIBRARIES}
  ${PNG_LIBRARIES}
  ${MAGIC_LIBRARIES}
)

# EXECUTABLE INCLUDES
target_include_directories(
  uploadr
  PUBLIC
  ${INCLUDE_DIR}
  ${NOTIFY_INCLUDE_DIRS}
  ${CURL_INCLUDE_DIRS}
  ${X11_INCLUDE_DIRS}
  ${PNG_INCLUDE_DIRS}
  ${MAGIC_INCLUDE_DIRS}
  ${LIB_DIR}/clip
  ${LIB_DIR}/jsoncons/include
  ${LIB_DIR}/pugixml/src
  ${LIB_DIR}/cpp-base64
)

# make uninstall
add_custom_target("uninstall" COMMENT "Uninstall installed files")
add_custom_command(
  TARGET "uninstall"
  POST_BUILD
  COMMENT "Uninstall files with install_manifest.txt"
  COMMAND xargs rm -vf < install_manifest.txt || echo Nothing in
  install_manifest.txt to be uninstalled!
)