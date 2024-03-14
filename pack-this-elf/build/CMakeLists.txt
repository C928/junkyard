cmake_minimum_required(VERSION 3.21)
project(pack_this_elf C)

set(CMAKE_C_STANDARD 11)

include_directories(../include)

set(OBJS ../src/packer.o ../src/sections.o ../src/error.o ../src/unpack_xor.o)

add_executable(pack-this-elf ${OBJS})

set_source_files_properties(
        ${OBJS}
        PROPERTIES
        EXTERNAL_OBJECT true
        GENERATED true
)
set_target_properties(pack-this-elf PROPERTIES LINKER_LANGUAGE C)