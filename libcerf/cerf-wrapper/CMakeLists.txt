cmake_minimum_required(VERSION 3.14 FATAL_ERROR)
project(cerf-wrapper)
set(CMAKE_CXX_STANDARD 11)

set(BUILD_SHARED_LIBS OFF CACHE BOOL "force static library" FORCE)
set(LIB_MAN OFF CACHE BOOL "force man pages off" FORCE)
set(LIB_RUN OFF CACHE BOOL "force no executables" FORCE)
set(CERF_CPP ON CACHE BOOL "force CPP compilation" FORCE)

add_subdirectory(libcerf)

add_library(cerf-wrapper STATIC cerf-wrapper.h cerf-wrapper.cpp)
target_link_libraries(cerfcpp)