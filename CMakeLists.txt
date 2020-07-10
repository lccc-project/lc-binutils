
include(GNUInstallDirs)
include(ExternalProject)

execute_process(COMMAND cargo fetch
        COMMAND cargo update
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

add_custom_target(lc-binutils ALL
        COMMAND cargo build --target-dir ${CMAKE_CURRENT_BINARY_DIR} $<$<CONFIG:Release>:--release>
        COMMENT "Building binutils with cargo"
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

set(_lcbinutils_output_dir ${CMAKE_CURRENT_BINARY_DIR}/$<IF:$<CONFIG:Release>,release,debug>)

add_executable(ar IMPORTED)
set_property(TARGET ar PROPERTY IMPORTED_LOCATION ${_lcbinutils_output_dir}/ar${CMAKE_EXECUTABLE_SUFFIX})
add_executable(lcld IMPORTED)
set_property(TARGET lcld PROPERTY IMPORTED_LOCATION ${_lcbinutils_output_dir}/lcld${CMAKE_EXECUTABLE_SUFFIX})


install(PROGRAMS ${_lcbinutils_output_dir}/ar${CMAKE_EXECUTABLE_SUFFIX}
        ${_lcbinutils_output_dir}/lcld${CMAKE_EXECUTABLE_SUFFIX}
        TYPE BIN)
include(install_binutils_symlinks.cmake)

if(NOT WIN32)
    install_binutils_symlink(ld.lc lcld)
    install_binutils_symlink(ld64.lc lcld)
    install_binutils_symlink(lc-link lcld)
    install_binutils_symlink(ranlib ar)
endif()