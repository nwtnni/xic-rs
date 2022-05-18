cmake_minimum_required(VERSION 2.6)
# based on kdelibs/kjs/CMakeLists.txt
project(qtxi)

include(CheckCXXCompilerFlag)

IF(WIN64)
    MESSAGE(WARNING " Windows build. Tricky workarounds enabled, plus this is untested since the 32 -> 64-bit migration.")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -mno-cygwin")
    SET(QT_QMAKE_EXECUTABLE ./qmake-wrap.sh)
    SET(QT_MOC_EXECUTABLE ./moc-wrap.sh)
    EXECUTE_PROCESS(COMMAND cygpath -m ${CMAKE_CURRENT_SOURCE_DIR}/ OUTPUT_VARIABLE QIDL_DIR)
ELSE(WIN64)
  SET(QIDL_DIR ${CMAKE_CURRENT_SOURCE_DIR}/)
ENDIF(WIN64)

# handy default option from the FAQ...
IF(NOT CMAKE_BUILD_TYPE)
  SET(CMAKE_BUILD_TYPE Debug CACHE STRING
      "Choose the type of build, options are: None Debug Release RelWithDebInfo MinSizeRel."
      FORCE)
ENDIF(NOT CMAKE_BUILD_TYPE)

set(Qt4_FIND_REQUIRED 1)
include(FindQt4)

find_library(Xi NAMES xi PATHS ${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_BINARY_DIR} NO_DEFAULT_PATH)

if(Xi)
    set(Xi_FOUND TRUE)
    MESSAGE("Found libxi in ${Xi}, will build examples")
else(Xi)
    MESSAGE(WARNING " libxi not found. Won't build examples.")
    MESSAGE(WARNING " copy it here, and remove CMakeCache.txt to try again.")
endif(Xi)

include_directories(${QT_INCLUDES} ${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_BINARY_DIR})


########### makebindings ###############
set(makebindings_SRCS
    generator/types.cpp
    generator/codeprinter.cpp
    generator/driver.cpp
    generator/lexer.cpp
    generator/parser.cpp
    generator/bindgen.cpp
   )
ADD_EXECUTABLE(makebindings ${makebindings_SRCS})

# and the custom command
ADD_CUSTOM_COMMAND(OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/bindqt.h ${CMAKE_CURRENT_BINARY_DIR}/bindqt.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/qt.ixi
  COMMAND ${CMAKE_CURRENT_BINARY_DIR}/makebindings ${QIDL_DIR}
  DEPENDS makebindings ${CMAKE_CURRENT_SOURCE_DIR}/qt.idl
          ${CMAKE_CURRENT_SOURCE_DIR}/bindqt.h.in
          ${CMAKE_CURRENT_SOURCE_DIR}/bindqt.cpp.in
          ${CMAKE_CURRENT_SOURCE_DIR}/qt.ixi.in
)

########### next target ###############

set (QtXi_MOC_HEADERS xiqslot.h)
qt4_wrap_cpp (QtXi_MOC_OUTFILES ${QtXi_MOC_HEADERS})
set(QtXi_LIB_SRCS
    ${CMAKE_CURRENT_BINARY_DIR}/bindqt.cpp
    gcpin.cpp
    xiqobjectholder.cpp
    xiqwrapperfactory.cpp
    xiqproxywidgets.cpp
    xiqslot.cpp
    )


add_library(QtXi STATIC ${QtXi_LIB_SRCS} ${QtXi_MOC_OUTFILES})

########### next target ###############

if (Xi_FOUND)
    ADD_EXECUTABLE(onewidget tests/onewidget.cpp)
    TARGET_LINK_LIBRARIES(onewidget QtXi ${Xi} ${QT_QTGUI_LIBRARY} ${QT_QTCORE_LIBRARY})

    ADD_EXECUTABLE(layouts tests/layouts.cpp)
    TARGET_LINK_LIBRARIES(layouts QtXi ${Xi} ${QT_QTGUI_LIBRARY} ${QT_QTCORE_LIBRARY})

    ADD_EXECUTABLE(widgetevents tests/widgetevents.cpp)
    TARGET_LINK_LIBRARIES(widgetevents QtXi ${Xi} ${QT_QTGUI_LIBRARY} ${QT_QTCORE_LIBRARY})

    ADD_EXECUTABLE(animate tests/animate.cpp)
    TARGET_LINK_LIBRARIES(animate QtXi ${Xi} ${QT_QTGUI_LIBRARY} ${QT_QTCORE_LIBRARY})

    ADD_EXECUTABLE(animate-fancy tests/animate-fancy.cpp)
    TARGET_LINK_LIBRARIES(animate-fancy QtXi ${Xi} ${QT_QTGUI_LIBRARY} ${QT_QTCORE_LIBRARY})
endif(Xi_FOUND)

