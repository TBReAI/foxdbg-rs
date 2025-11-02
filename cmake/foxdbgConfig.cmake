get_filename_component(_FOXDBG_PKGDIR "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
set(_FOXDBG_INC "${_FOXDBG_PKGDIR}/include")

if (WIN32)
  set(_FOXDBG_LIB "${_FOXDBG_PKGDIR}/lib/foxdbg.lib")
else()
  set(_FOXDBG_LIB "${_FOXDBG_PKGDIR}/lib/libfoxdbg.a")
endif()

if (NOT EXISTS "${_FOXDBG_LIB}")
  message(FATAL_ERROR "foxdbg library not found at ${_FOXDBG_LIB}")
endif()

add_library(foxdbg::foxdbg STATIC IMPORTED GLOBAL)
set_target_properties(foxdbg::foxdbg PROPERTIES
  IMPORTED_LOCATION "${_FOXDBG_LIB}"
  INTERFACE_INCLUDE_DIRECTORIES "${_FOXDBG_INC}"
)