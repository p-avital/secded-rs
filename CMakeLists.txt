# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.


CMAKE_MINIMUM_REQUIRED(VERSION 3.10)
PROJECT(secded)

OPTION(USE_DYN "Enable Dynamic SECDED (only useful to encode bigger than 120bits payloads)" OFF)
OPTION(NO_PANIC "Disables Panics" OFF)
ADD_EXECUTABLE(example example.c)

IF(USE_DYN)
    TARGET_COMPILE_DEFINITIONS(example PRIVATE SECDED_FEATURES_DYN)
    IF(NO_PANIC)
        ADD_CUSTOM_TARGET(secded COMMAND cargo build --release --features "dyn ffi no-panics")
    ELSE()
        ADD_CUSTOM_TARGET(secded COMMAND cargo build --release --features "dyn ffi")
    ENDIF()
ELSEIF(NO_PANIC)
    ADD_CUSTOM_TARGET(secded COMMAND cargo build --release --features "ffi no-panics")
ELSE()
    ADD_CUSTOM_TARGET(secded COMMAND cargo build --release --features "ffi")
ENDIF()

ADD_DEPENDENCIES(example secded)
TARGET_LINK_LIBRARIES(example PRIVATE ${CMAKE_CURRENT_LIST_DIR}/target/release/libsecded.a)
TARGET_LINK_LIBRARIES(example PUBLIC  pthread dl)
