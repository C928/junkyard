#include "../include/packer.h"

int defined_error(int err_flag, struct binary_s* exec, struct options_flags_s* opt_f)
{
    switch(err_flag)
    {
        case err_bad_args:
            fprintf(stderr, "Error: Usage: ./pack-this-elf <binary name> [options ...]\n"
                            "Use -h or --help option for more\n\n");
            break;

        case err_bad_enc_key:
            fprintf(stderr, "Error: Encryption key: length must be 8 bytes,"
                            "( 20 digits in base 10 ) and also be greater than 0. Do not forget to pad with 0s. "
                            "Input was ""%" PRIx64 "\n\n", opt_f->enc_key);
            break;

        case  err_bad_rotr_value:
            fprintf(stderr, "Error: Rotate right value: length must be of size sizeof(unsigned int) which "
                            "is 4 bytes for most compilers. The value must also be greater than 0. "
                            "Do not forget to pad with 0s. Input was ""%u""\n\n", opt_f->rotr_value);
            break;

        case err_dupl_file:
            fprintf(stderr, "Error: File: ""%s"" could not be duplicated\n\n", exec->filename);
            break;

        case err_not_elf:
            fprintf(stderr, "Error: File: ""%s"" is not an ELF file\n\n", exec->filename);
            break;

        case err_bad_arch:
            fprintf(stderr, "Error: Architecture: ""%s"" has %hu architecture, this packer currently "
                            "only support x86_64\n\n", exec->filename, exec->cpu_arch);
            break;

        case err_len_filename:
            fprintf(stderr, "Error: Filename: length of binary name must be less than 18. ""%s"" is %d\n\n",
                            exec->filename, exec->len_filename);
            break;

        case err_stat_file:
            fprintf(stderr, "Error: File: could not load the file ""%s""\n\n", exec->filename);
            break;

        case err_fd_open:
            fprintf(stderr, "Error: File Descriptor: ""%s"" no such file or directory\n\n", exec->filename);
            break;

        case err_mem_mapping:
            fprintf(stderr, "Error: Mapping: mmap could not map into memory\n\n");
            break;

        case err_fd_read:
            fprintf(stderr, "Error: File Descriptor: ""%s"" cannot be read\n\n", exec->filename);
            break;

        case err_fd_write:
            fprintf(stderr, "Error: File Descriptor: ""%s"" cannot be written\n\n", exec->filename_packed);
            break;

        case err_fd_seek:
            fprintf(stderr, "Error: File Descriptor: cannot seek offset in ""%s""\n\n", exec->filename);
            break;

        case err_sec_not_found:
            fprintf(stderr, "Error: Section: %s not found\n\n", opt_f->sec_name);
            break;

        case err_enc:
            fprintf(stderr, "Error: Section: %s could not be encrypted\n\n", opt_f->sec_name);
            break;

        case err_dyn_alloc:
            fprintf(stderr, "Error: Malloc, Calloc, Realloc: dynamic allocation failed\n\n");
            break;

        case err_mem_copy:
            fprintf(stderr, "Error: Memcpy: memory content could not be copied, memcpy() failed\n\n");
            break;

        case err_add_sec:
            fprintf(stderr, "Error: Section: ""%s"" could not be added\n\n", opt_f->new_sec_name);
            break;

        case err_no_pt_load:
            fprintf(stderr, "Error: Segment: ""%s"" has no pt_load segment\n\n", exec->filename);
            break;

        case err_no_suitable_sec:
            fprintf(stderr, "Error: Stub: %s has no suitable section for insertion\n\n", opt_f->stub_name);
            break;

        case err_chmod:
            fprintf(stderr, "Error: Privilege: could not chmod of %s to 0775\n\n", exec->filename_packed);
            break;

        default:
            fprintf(stderr, "Error: Unidentified\n\n");
    }

    return 1;
}
