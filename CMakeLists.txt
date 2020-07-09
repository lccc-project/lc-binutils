
include(GNUInstallDirs)
include(ExternalProject)

execute_process(COMMAND cargo fetch
        COMMAND cargo update
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

add_custom_target(NAME lc-binutils
        COMMAND cargo build --target-dir ${CMAKE_CURRENT_BINARY_DIR} $<$<CONFIG:Release>:--release>
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

add_executable(ar IMPORTED)
set_property(TARGET ar PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_BINARY_DIR}/ar${CMAKE_EXECUTABLE_SUFFIX})
add_executable(lcld IMPORTED)
set_property(TARGET lcld PROPERTY IMPORTED_LOCATION ${CMAKE_CURRENT_BINARY_DIR}/lcld${CMAKE_EXECUTABLE_SUFFIX})

if(NOT WIN32)
add_custom_command(TARGET lc-binutils POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E create_symlink lcld ld.lc
        COMMAND ${CMAKE_COMMAND} -E create_symlink lcld ld64.lc
        COMMAND ${CMAKE_COMMAND} -E create_symlink lcld lc-link)
endif()

install(PROGRAMS ${CMAKE_CURRENT_BINARY_DIR}/ar${CMAKE_EXECUTABLE_SUFFIX}
        ${CMAKE_CURRENT_BINARY_DIR}/lcld${CMAKE_EXECUTABLE_SUFFIX}
        $<$<BOOL:${WIN32}>:ld.lc;ld64.lc;lc-link>
        TYPE BIN)