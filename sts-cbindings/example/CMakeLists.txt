cmake_minimum_required(VERSION 3.30)
project(nist_sts_c C)

set(CMAKE_C_STANDARD 11)

add_executable(nist_sts_dyn main.c)
target_link_libraries(nist_sts_dyn "${CMAKE_CURRENT_LIST_DIR}/lib/libsts.so")

add_executable(nist_sts_static main.c)
target_link_libraries(nist_sts_static "${CMAKE_CURRENT_LIST_DIR}/lib/libsts.a" m)
