cmake_minimum_required(VERSION 3.12)

set(PICO_SDK_PATH "D:/noam4/pipico/pico-sdk")
set(CMAKE_EXPORT_COMPILE_COMMANDS 1)

include(pico_sdk_import.cmake)

set(PROJECT_NAME temp-humidity)
project(${PROJECT_NAME} C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

if (PICO_SDK_VERSION_STRING VERSION_LESS "1.3.0")
    message(FATAL_ERROR "Raspberry Pi Pico SDK version 1.3.0 (or later) required. Your version is ${PICO_SDK_VERSION_STRING}")
endif()

pico_sdk_init()

file(GLOB PROJ_SRC src/*)
add_executable(${PROJECT_NAME} ${PROJ_SRC})

pico_add_extra_outputs(${PROJECT_NAME})

target_link_libraries(${PROJECT_NAME} pico_stdlib hardware_uart)
