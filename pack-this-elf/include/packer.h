#ifndef ELF64_PACKER_H
#define ELF64_PACKER_H

#include <elf.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <getopt.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <unistd.h>
#include <inttypes.h>
#include <math.h>
#include <fcntl.h>
#include <time.h>
#include <assert.h>

#define DEF_STUB            "xor_stub"
#define DEF_SEC_NAME        ".text"
#define DEF_NEW_SEC_NAME    ".added"
#define DEF_PRINT_SEC       false

#define BIN_STRUCT_SIZE     (sizeof(char*) * 2 + sizeof(unsigned int) + sizeof(unsigned short))
#define OPT_F_STRUCT_SIZE   (sizeof(char*) * 3 + sizeof(uint64_t) + sizeof(unsigned int) + sizeof(bool))

// unpack_xor last line: rotr_value:     dd      0xCCCCCCCC
// mov rsi, [rel rotr_value]
// 4 12 20 28

/*xor byte [rbx + rax * 8], dl
ror rdx, 8
inc rax
cmp rax, rcx
jnz loop
nop
nop
jmp 0xcccccccc */

static struct option const long_options[] =
{
    {"section-name", required_argument, NULL, 's'},
    {"new-section-name", required_argument, NULL, 'n'},
    {"encryption-key", required_argument, NULL, 'k'},
    {"rot-right-value", required_argument, NULL, 'r'},
    {"xor-encryption", no_argument, NULL, 'x'},
    {"print-section", no_argument, NULL, 'p'},
    {"help", no_argument, NULL, 'h'}
};

enum error_flags
{
    err_bad_args,
    err_bad_enc_key,
    err_bad_rotr_value,
    err_dupl_file,
    err_bad_arch,
    err_len_filename,
    err_stat_file,
    err_not_elf,
    err_sec_not_found,
    err_mem_mapping,
    err_fd_open,
    err_fd_read,
    err_fd_write,
    err_fd_seek,
    err_enc,
    err_dyn_alloc,
    err_mem_copy,
    err_add_sec,
    err_no_pt_load,
    err_no_suitable_sec,
    err_chmod
};

typedef struct binary_s
{
    char*               filename;               /* elf64 binary name */
    unsigned int        len_filename;           /* length of binary name */
    char*               filename_packed;        /* copy of original binary */
    unsigned short      cpu_arch;               /* cpu architecture, currently only support x86-64 */
} bin_t;

typedef struct options_flags_s
{
    uint64_t            enc_key;                /* encryption key, unsigned long int ( 8 bytes ) */
    char*               sec_name;               /* section to be encrypted */
    char*               new_sec_name;           /* name of created section */
    char*               stub_name;              /* encryption algorithm used (xor, custom, ...) */
    unsigned int        rotr_value;             /* random value for key rotation */
    bool                print_sec;              /* print section content before it has been encrypted, or after or both */
} opt_flag_t;


/* struct used for memory mapping */
typedef struct new_binary_s
{
    Elf64_Ehdr*         elf64_ehdr;             /* pointer to elf header for 64bits elf binaries */
    Elf64_Shdr*         elf64_shdr;             /* pointer to section header for 64bits elf binaries */
    Elf64_Phdr*         elf64_phdr;             /* pointer to program header for 64bits elf binaries */
    int**               section_content;        /* double pointer for dynamic allocation of section contents */
} new_bin_t;


/* ------------------------------- error.c ------------------------------- */

/* Printing to stderr stream potentially encountered errors. */
/* All reported errors are listed in the error_flags enum above */
int         defined_error(int err_flag, struct binary_s* exec, struct options_flags_s* opt_f);


/* ------------------------------- packer.c ------------------------------- */

/* Makes a copy of the original binary and calls it "<filename>.packed" */
int         duplicate_file(struct binary_s* bin, struct options_flags_s* opt_f);

/* Dynamically allocating space ( using mmap and calloc functions ) for the 64bits elf binary */
int         memory_mapping(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f);

/* Does the same thing as strdup() or strndup() but avoid segfault */
char*       format_filename_packed(struct binary_s* bin);

/* Parsing user input with the getopt library ( used in almost, if not all binutils binaries ) */
int         parse_option_flags(int argc, char** argv, struct binary_s* bin, struct options_flags_s* opt_f);

/* Writing changes to <filename>.packed */
int         write_new_bin(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f);

/* Calling the packer functions and managing potential errors that comes along the way */
int         main(int argc, char **argv);


/* ------------------------------ sections.c ------------------------------*/

/* Converts a most significant byte (msb) value to a less significant byte (lsb) value */
int         msb_to_lsb(int msb_value);

/* todo */
void        int_to_ascii(int int_value, uint8_t* ascii_buf, int index);

/* Generating a random unsigned int ( 4 bytes ) for key rotation while xoring section if the user hasn't chosen one */
uint32_t    random_rotr();

/* Generating a random uint64_t key ( 8 bytes ) if the user hasn't chosen one */
uint64_t    random_key();

/* Formats address of last section content line todo*/
uint64_t    format_end_line_addr(uint64_t end_addr);

/* Prints section content ( only if --print-section has been passed as parameter ) */
void        dump_section_content(int* sec_code, uint64_t sec_addr, uint64_t sec_size);

/* "xor encryption" algorithm */
void        xor_section(int** sec_code, uint64_t sec_size, struct options_flags_s *opt_flag_t);

/* Modifies original entry point in the elf header */
void        modify_oep(struct new_binary_s* new_bin, struct options_flags_s* opt_f, int next_sec);

/* Adds the section containing the chosen assembly stub */
int         add_section(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f);

/* Encrypts section with the chosen algorithm */
int         encrypt_section(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f);


#endif //ELF64_PACKER_H
