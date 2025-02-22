cmake_minimum_required(VERSION 3.8)

project(dnp3_c LANGUAGES C)

# Find dnp3
if(WIN32)
    set(CMAKE_PREFIX_PATH ${CMAKE_CURRENT_LIST_DIR}/generated/x86_64-pc-windows-msvc/cmake)
elseif(UNIX)
    set(CMAKE_PREFIX_PATH ${CMAKE_CURRENT_LIST_DIR}/generated/x86_64-unknown-linux-gnu/cmake)
endif()
find_package(dnp3 REQUIRED)

# Master example
add_executable(master_example master_example.c)
target_link_libraries(master_example PRIVATE dnp3)

# Outstation example
add_executable(outstation_example outstation_example.c)
target_link_libraries(outstation_example PRIVATE dnp3)

# Copy the DLL after build
add_custom_command(TARGET master_example POST_BUILD 
    COMMAND ${CMAKE_COMMAND} -E copy_if_different $<TARGET_FILE:dnp3> $<TARGET_FILE_DIR:master_example>
)
add_custom_command(TARGET outstation_example POST_BUILD 
    COMMAND ${CMAKE_COMMAND} -E copy_if_different $<TARGET_FILE:dnp3> $<TARGET_FILE_DIR:outstation_example>
)
