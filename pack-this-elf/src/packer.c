#include "../include/packer.h"

int duplicate_file(struct binary_s* bin, struct options_flags_s* opt_f)
{
    struct stat st  = { 0 };
    if(stat(bin->filename, &st) != 0)
        return defined_error(err_stat_file, bin, opt_f);

    int fd = 0;
    if(!(fd = open(bin->filename, O_RDONLY, 0400)))
        return defined_error(err_fd_open, bin, opt_f);

    int fd2 = 0;
    if(!(fd2 = open(bin->filename_packed, O_CREAT | O_RDWR, 0600)))
        return defined_error(err_fd_open, bin, opt_f);

    char* buff = (char*)malloc(st.st_size);
    if(!buff)
        return defined_error(err_dyn_alloc, bin, opt_f);

    if(!(read(fd, buff, st.st_size)))
        return defined_error(err_fd_read, bin, opt_f);

    if(!write(fd2, buff, st.st_size))
        return defined_error(err_fd_write, bin, opt_f);

    if(lseek(fd, 0, SEEK_SET))
        return defined_error(err_fd_seek, bin, opt_f);

    return fd2;
}


char* format_filename_packed(struct binary_s* bin)
{
    char* filename_packed = (char*)malloc(bin->len_filename + 8);
    memcpy(filename_packed, bin->filename, bin->len_filename);
    strncat(filename_packed, ".packed", 8);

    return filename_packed;
}


int parse_option_flags(int argc, char** argv, struct binary_s* bin, struct options_flags_s* opt_f)
{
    bin->len_filename = strlen(argv[1]);
    if(bin->len_filename >= 18)
        return defined_error(err_len_filename, bin, opt_f);

    bin->filename            = argv[1];
    bin->filename_packed     = format_filename_packed(bin);
    opt_f->sec_name          = DEF_SEC_NAME;
    opt_f->new_sec_name      = DEF_NEW_SEC_NAME;
    opt_f->stub_name         = DEF_STUB;
    opt_f->print_sec         = DEF_PRINT_SEC;
    opt_f->enc_key           = 0;    //todo
    char* tmp                = NULL;

    while (true)
    {
        int oi = -1;
        int c = getopt_long(argc, argv, "hsnkrxp", long_options, &oi);
        // TODO: fix segfault for --random_value
        if (c == -1)
            break;

        switch(c)
        {
            case 'h':
                printf("Usage: ./pte <binary name> [options ...]\n\n"
                       "Pack-This-Elf options:\n"
                       "    -h, --help\t\t\tPrint this message.\n"
                       "    -k, --encryption-key\tChoose the 4 bytes key used for encryption\n"
                       "    -s, --section-name\t\tChoose the section that is going to be encrypted\n"
                       "    -n, --new-section-name\tChoose name for the new section\n"
                       "    -x, --xor-encryption\tUse ""xor encryption""\n"
                       "    -p, --print-section\t\tPrint content of original section and also content after"
                       " it has been encrypted\n\n");
                return 2;

            case 's':
                opt_f->sec_name = optarg;
                break;

            case 'n':
                opt_f->new_sec_name = optarg;
                break;

            case 'k':
                if(strlen(optarg) != 20) //todo: allow hex key
                    return defined_error(err_bad_enc_key, bin, opt_f);

                tmp = optarg + strlen(optarg);
                while(optarg < tmp)
                    opt_f->enc_key = (opt_f->enc_key << 1) + (opt_f->enc_key << 3) + *(optarg++) - '0';

                if(opt_f->enc_key <= 0)
                    return defined_error(err_bad_enc_key, bin, opt_f);
                break;

            case 'r':
                if(strlen(optarg) != sizeof(unsigned int))
                    return defined_error(err_bad_rotr_value, bin, opt_f);

                tmp = optarg + strlen(optarg);
                while(optarg < tmp)
                    opt_f->rotr_value = (opt_f->rotr_value << 1) + (opt_f->rotr_value << 3) + *(optarg++) - '0';

                if(opt_f->rotr_value <= 0)
                    return defined_error(err_bad_rotr_value, bin, opt_f);
                break;

            case 'x':
                opt_f->stub_name = "xor_stub";
                break;

            case 'p':
                opt_f->print_sec = true;
                break;

            default:
                return defined_error(err_bad_args, bin, opt_f);
        }
    }

    // if (opt_f->sec_name == NULL)
    //     opt_f->sec_name = ".text";

    // if (opt_f->new_sec_name == NULL)
    //     opt_f->new_sec_name = ".added";

    // if (opt_f->enc_key == 0)
    //     opt_f->enc_key = 0xa5;

    // if (opt_f->stub_name == NULL)
    //     opt_f->stub_name = "xor_stub";

    return 0;
}


int memory_mapping(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f)
{
    void* file_mapping      = NULL;
    void* start_address      = NULL;
    __off_t start_offset    = 0;
    size_t file_size        = 0;
    void* ret               = NULL;
    int fd                  = 0;

    fd = open(bin->filename_packed, O_RDONLY, 0400);
    if(!fd)
        return defined_error(err_fd_open, bin, opt_f);

    file_size = lseek(fd, start_offset, SEEK_END);
    if(!file_size)
        return defined_error(err_fd_seek, bin, opt_f);

    file_mapping = mmap(start_address, file_size, PROT_READ | PROT_WRITE, MAP_PRIVATE, fd, start_offset);
    if(file_mapping == MAP_FAILED)
        return -1;

    new_bin->elf64_ehdr = (Elf64_Ehdr*)calloc(1, sizeof(Elf64_Ehdr));
    if(!new_bin->elf64_ehdr || file_size < sizeof(Elf64_Ehdr))
        return defined_error(err_dyn_alloc, bin, opt_f);

    memcpy(new_bin->elf64_ehdr, file_mapping, sizeof(Elf64_Ehdr));

    new_bin->elf64_phdr = (Elf64_Phdr*)calloc(new_bin->elf64_ehdr->e_phnum, sizeof(Elf64_Phdr));
    if(!new_bin->elf64_phdr)
        return defined_error(err_dyn_alloc, bin, opt_f);

    int i = 0;
    for(; i < (new_bin->elf64_ehdr)->e_phnum; i++)
    {
        ret = memcpy(&new_bin->elf64_phdr[i], file_mapping + new_bin->elf64_ehdr->e_phoff + i * sizeof(Elf64_Phdr),
                     sizeof(Elf64_Phdr));
        if(!ret)
            return defined_error(err_mem_copy, bin, opt_f);
    }

    new_bin->elf64_shdr = (Elf64_Shdr*)calloc(new_bin->elf64_ehdr->e_shnum, sizeof(Elf64_Shdr));
    if(!new_bin->elf64_shdr)
        return defined_error(err_dyn_alloc, bin, opt_f);

    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        ret = memcpy(&new_bin->elf64_shdr[i], file_mapping + new_bin->elf64_ehdr->e_shoff + i * sizeof(Elf64_Shdr),
                     sizeof(Elf64_Shdr));
        if(!ret)
            return defined_error(err_mem_copy, bin, opt_f);
    }

    new_bin->section_content = (int**)calloc(new_bin->elf64_ehdr->e_shnum, sizeof(int*));
    if(!new_bin->section_content)
        return defined_error(err_dyn_alloc, bin, opt_f);

    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        if(new_bin->elf64_shdr[i].sh_type == SHT_NULL || new_bin->elf64_shdr[i].sh_type == SHT_NOBITS)
            new_bin->section_content[i] = 0;

        else
        {
            new_bin->section_content[i] = (int*)calloc(1, new_bin->elf64_shdr[i].sh_size);
            if(!new_bin->section_content[i])
                return defined_error(err_dyn_alloc, bin, opt_f);

            ret = memcpy(new_bin->section_content[i], file_mapping + new_bin->elf64_shdr[i].sh_offset,
                         (size_t)new_bin->elf64_shdr[i].sh_size);
            if(!ret)
                return defined_error(err_mem_copy, bin, opt_f);
        }
    }

    munmap(file_mapping, sizeof(file_mapping));
    close(fd);
    return 0;
}

//todo: implement write function to avoid redundant code
int write_new_bin(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f)
{
    uint64_t offset = 0;
    const char zero = '0';

    int fd = 0;
    if(!(fd = open(bin->filename_packed, O_WRONLY, 0700)))
        return defined_error(err_fd_open, bin, opt_f);

    if(!write(fd, new_bin->elf64_ehdr, sizeof(Elf64_Ehdr)))
        return defined_error(err_fd_write, bin, opt_f);

    offset += sizeof(Elf64_Ehdr);

    for(; offset < new_bin->elf64_ehdr->e_phoff; offset++)
    {
        if(!write(fd, (const void*)&zero, sizeof(char)))
            return defined_error(err_fd_write, bin, opt_f);
    }

    if(!write(fd, new_bin->elf64_phdr, new_bin->elf64_ehdr->e_phnum * sizeof(Elf64_Phdr)))
        return defined_error(err_fd_write, bin, opt_f);

    offset += new_bin->elf64_ehdr->e_phnum * sizeof(Elf64_Phdr);

    uint64_t y = 0, i = 0;
    for(; y < new_bin->elf64_ehdr->e_shnum; y++)
    {
        if(new_bin->elf64_shdr[y].sh_type != SHT_NOBITS)
        {
            for(; offset < new_bin->elf64_shdr[y].sh_offset; offset++)
            {
                if(!write(fd, (const void*)&zero, sizeof(char)))
                    return defined_error(err_fd_write, bin, opt_f);
            }
            offset += i;

            //printf("%08x\n", section_content[y]);
            //todo ?wtf
            if(new_bin->section_content[y] != NULL)
            {
                if(!write(fd, new_bin->section_content[y], new_bin->elf64_shdr[y].sh_size))
                    return defined_error(err_fd_write, bin, opt_f);
            }
            offset += new_bin->elf64_shdr[y].sh_size;
        }
    }
    for(; offset < new_bin->elf64_ehdr->e_shoff; offset++)
    {
        if(!write(fd, (const void*)&zero, new_bin->elf64_ehdr->e_shoff))
            return defined_error(err_fd_write, bin, opt_f);
    }

    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        if(!write(fd, &new_bin->elf64_shdr[i], sizeof(Elf64_Shdr)))
            return defined_error(err_fd_write, bin, opt_f);
    }

    if((fd = chmod(bin->filename_packed, 0744)))
        return defined_error(err_chmod, bin, opt_f);

    close(fd);
    return 0;
}

void free_memory(struct binary_s* bin, struct options_flags_s* opt_f, struct new_binary_s* new_bin)
{
    free(bin);
    free(opt_f);
    free(new_bin->elf64_ehdr);
    free(new_bin->elf64_shdr);
    free(new_bin->elf64_phdr);
    free(new_bin->section_content);
    free(new_bin);
}

int main(int argc, char** argv)
{
    bin_t* bin              = (bin_t*)calloc(1, sizeof(bin_t));
    opt_flag_t* opt_f       = (opt_flag_t*)calloc(1, sizeof(opt_flag_t));
    new_bin_t*  new_bin     = (new_bin_t*)calloc(1, sizeof(new_bin_t));
    Elf64_Ehdr elf64_ehdr   = { 0 };

    if(argc < 2)
        return defined_error(err_bad_args, bin, opt_f);

    int ret = parse_option_flags(argc, argv, bin, opt_f);
    if(ret == 2)
        return 0;

    if(ret == 1)
        return 1;

    int fd = duplicate_file(bin, opt_f);
    if(fd == 0 || fd == 1)
        return defined_error(err_fd_open, bin, opt_f);

    if(lseek(fd, 0, SEEK_SET))
        return defined_error(err_fd_seek, bin, opt_f);

    if(!read(fd, &elf64_ehdr, sizeof(Elf64_Ehdr)))
        return defined_error(err_fd_read, bin, opt_f);

    close(fd);
    char ident[4] = { 0 }, magic[4] = { 0 };
    for(int i = 0; i < 4; i++)
        ident[i] = (char)elf64_ehdr.e_ident[i];

    snprintf(magic, sizeof(magic), "%s", ident+1);
    if(strcmp(magic,"ELF") != 0)
        return defined_error(err_not_elf, bin, opt_f);

    bin->cpu_arch = elf64_ehdr.e_machine;
    if(bin->cpu_arch != EM_X86_64)
        return defined_error(err_bad_arch, bin, opt_f);
    printf("%s%s\n", magic, "x86-64");

    ret = memory_mapping(bin, new_bin, opt_f);
    if(ret == -1)
        return defined_error(err_mem_mapping, bin, opt_f);

    if(ret == 1)
        return 1;

    ret = encrypt_section(bin, new_bin, opt_f);
    if(ret == 1)
        return 1;

    if(add_section(bin, new_bin, opt_f))
        return defined_error(err_add_sec, bin, opt_f);

    ret = write_new_bin(bin, new_bin, opt_f);
    if(ret == 1)
        return 1;

    /*for(int i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        if(new_bin->section_content[i] != NULL)
        {
            char* name = (char*)new_bin->section_content[new_bin->elf64_ehdr->e_shstrndx]
                         + new_bin->elf64_shdr[i].sh_name;
            printf("sec name :%s num: %d\n", name, i);
            uint64_t sec_addr = new_bin->elf64_shdr[i].sh_addr;
            uint64_t sec_size = new_bin->elf64_shdr[i].sh_size;
            dump_section_content(new_bin->section_content[i], sec_addr, sec_size);
        }
        printf("\n");
    }*/

    free_memory(bin, opt_f, new_bin);
    return 0;
}
