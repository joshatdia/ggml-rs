# Install script for directory: C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Release")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/cmake_install.cmake")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/Debug/ggml.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/Release/ggml.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/MinSizeRel/ggml.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/RelWithDebInfo/ggml.lib")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/Debug/ggml.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/Release/ggml.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/MinSizeRel/ggml.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/RelWithDebInfo/ggml.dll")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-cpu.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-alloc.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-backend.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-blas.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-cann.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-cpp.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-cuda.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-opt.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-metal.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-rpc.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-sycl.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/ggml-vulkan.h"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/ggml/include/gguf.h"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/Debug/ggml-base.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/Release/ggml-base.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/MinSizeRel/ggml-base.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY OPTIONAL FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/src/RelWithDebInfo/ggml-base.lib")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/Debug/ggml-base.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/Release/ggml-base.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/MinSizeRel/ggml-base.dll")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE SHARED_LIBRARY FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/bin/RelWithDebInfo/ggml-base.dll")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/share/pkgconfig" TYPE FILE FILES "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/ggml.pc")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/ggml" TYPE FILE FILES
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/ggml-config.cmake"
    "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/ggml-version.cmake"
    )
endif()

if(CMAKE_INSTALL_COMPONENT)
  set(CMAKE_INSTALL_MANIFEST "install_manifest_${CMAKE_INSTALL_COMPONENT}.txt")
else()
  set(CMAKE_INSTALL_MANIFEST "install_manifest.txt")
endif()

string(REPLACE ";" "\n" CMAKE_INSTALL_MANIFEST_CONTENT
       "${CMAKE_INSTALL_MANIFEST_FILES}")
file(WRITE "C:/Users/JoshuaGoodman/Documents/GitHub/ggml-rs/target/debug/build/ggml-rs-d3413da75aebc6f0/out/build/${CMAKE_INSTALL_MANIFEST}"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
