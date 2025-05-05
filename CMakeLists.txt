cmake_minimum_required(VERSION 3.15)
project(CppRustStatic)

set(CMAKE_CXX_STANDARD 17)

if(WIN32)
    set(RUST_LIB_NAME libsnix.lib)
else()
    set(RUST_LIB_NAME liblibsnix.a)
endif()

set(RUST_LIB_PATH ${CMAKE_SOURCE_DIR}/rust_lib/target/release/${RUST_LIB_NAME})

# Build Rust static library
include(ExternalProject)
ExternalProject_Add(rustlib
    PREFIX ${CMAKE_BINARY_DIR}/rustlib
    SOURCE_DIR ${CMAKE_SOURCE_DIR}/rust_lib
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release
    INSTALL_COMMAND ""
    BUILD_IN_SOURCE 1
    BUILD_BYPRODUCTS ${RUST_LIB_PATH}
)

add_executable(njq cpp/main.cpp)
add_dependencies(njq rustlib)
target_include_directories(njq PRIVATE cpp)

if(WIN32)
    target_link_libraries(njq PRIVATE ${RUST_LIB_PATH}
        ws2_32
        secur32
        userenv
        ntdll
        advapi32
        bcrypt)
else()
    target_link_libraries(njq PRIVATE ${RUST_LIB_PATH})

endif()

target_link_libraries(njq PRIVATE ${RUST_LIB_PATH})
