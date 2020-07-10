
function(install_binutils_symlink name src)
    install(CODE "file(CREATE_LINK ${src} ${CMAKE_INSTALL_FULL_BINDIR}/${name} SYMBOLIC)")
endfunction()