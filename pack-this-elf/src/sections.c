#include "../include/packer.h"

extern void*            entry_xor_stub();
extern uint64_t         xor_stub_size;
//todo remove xor_stub_infos_size
extern uint64_t         xor_stub_infos_size;

//uint64_t         enc_key;     
//unsigned int     rotr_value;  

int msb_to_lsb(int msb_value)
{
    return (msb_value << 24) |
        ((msb_value <<  8) & 0x00ff0000) |
        ((msb_value >>  8) & 0x0000ff00) |
        ((msb_value >> 24) & 0x000000ff);
}

void int_to_ascii(int int_value, uint8_t* ascii_buff, int index)
{
    uint64_t i = 0;
    uint8_t tmp = 0;
    uint64_t size = 0;
    uint64_t zero_pad = 0;

    snprintf((char*)(ascii_buff + index * 4), 9, "%x", int_value);
    size = strlen((const char*)(ascii_buff + index * 4));
    zero_pad = 8 - size;
    if(size != 8)
    {
        uint8_t tmp_buff[8];
        for(; i < 8; i++)
        {
            if(i < zero_pad)
                tmp_buff[i] = '.';

            else
                tmp_buff[i] = ascii_buff[index * 4 + i - zero_pad];
        }
        strncpy((char*)(ascii_buff + index * 4), (const char*)tmp_buff, 8);
        ascii_buff[index * 4 + 8] = '\0';
    }

    int value1, value2;
    for(i = 0; i < strlen((const char*)(ascii_buff + index * 4)); i += 2)
    {
        if(i >= zero_pad)
        {
            value1 = ascii_buff[index * 4 + i];
            value2 = ascii_buff[index * 4 + i + 1];
            if(value1 > 0x39)
                value1 -= 0x31 - 0xa;

            if(value2 > 0x39)
                value2 -= 0x31 - 0xa;

            tmp = (value1 - '0') * 0x10 + (value2 - '0');
        }
        if(tmp < 0x20 || tmp > 0x7e)
        {
            if(i == 0)
                ascii_buff[index * 4 + i] = '.';
            else
                ascii_buff[index * 4 + i / 2] = '.';
            continue;
        }
        else
        {
            if(i == 0)
                ascii_buff[index * 4 + i] = tmp;
            else
                ascii_buff[index * 4 + i / 2] = tmp;
        }
    }

    ascii_buff[index * 4 + 4] = '\0';
}

uint64_t format_end_line_addr(uint64_t end_addr)
{
    uint64_t end_line_addr = end_addr;
    uint8_t tmp_buff[sizeof(uint64_t)];

    if(end_addr % 0x10 != 0)
    {
        snprintf((char*)tmp_buff, sizeof(uint64_t), "%" PRIx64, end_addr);
        for(uint64_t i = 0; i < strlen((const char*)tmp_buff); i++)
        {
            if(tmp_buff[i + 1] == '\0')
                tmp_buff[i] = 0;
        }
        end_line_addr = strtoull((const char*)tmp_buff, NULL, 16);
        end_line_addr *= 0x10;
    }

    return end_line_addr;
}


void dump_section_content(int* sec_code, uint64_t sec_addr, uint64_t sec_size)
{
    int index = -1;
    uint64_t count = 0;
    uint64_t end_line_addr = format_end_line_addr(sec_addr + sec_size);
    uint8_t ascii_dump[17];

    printf(" ""%" PRIx64 "\t  ", sec_addr);
    if(sec_size > 16)
        count = (uint64_t)(sec_size / 4 + 1);
    else
        count = 4;

    for(uint64_t y = 1; y < count + 1; y++)
    {
        index = (index + 1) % 4;
        int_to_ascii(msb_to_lsb(sec_code[y - 1]), ascii_dump, index);
        printf("%08x ", msb_to_lsb(sec_code[y - 1]));

        if(y % 4 == 0 || sec_addr == end_line_addr)
        {
            sec_addr = sec_addr + 0x10;
            if(sec_addr - 0x10 != end_line_addr)
            {
                printf(" %s\n", ascii_dump);
                printf(" %" PRIx64 "\t  ", sec_addr);
            }
            else
            {
                printf("%08x ", msb_to_lsb(sec_code[y]));
                int_to_ascii(msb_to_lsb(sec_code[y]), ascii_dump, (index + 1) % 4);
                printf("\t\t       %s\n", ascii_dump); //todo: tab + delete extra characters
                break;
            }
        }
        /*if(y % 4 == 0 && y != 0)
            printf(" %s", ascii_dump); */
    }
    printf("\n\n");
}

uint64_t random_key()
{
    uint64_t key = 0;

    srand((unsigned int)time(NULL));
    for(int i = 0; i < 22; i++)
        key = 10 * key + rand()%10;

    return key;
}

uint32_t random_rotr()
{
    unsigned int rotr_value = 0; 
    //rotr_value = 0; 

    srand((unsigned int)time(NULL));
    for(int i = 0; i < 10; i++)
        rotr_value = 10 * rotr_value + rand()%10;

    return rotr_value;
}

void xor_section(int** sec_code, uint64_t sec_size, struct options_flags_s *opt_f)
{
    if(opt_f->enc_key == 0)
        opt_f->enc_key = random_key();

    if(opt_f->rotr_value == 0)
        opt_f->rotr_value = random_rotr();

    //unsigned int n = opt_f->rotr_value;
    uint64_t tmp_key = opt_f->enc_key;
    uint64_t count = (uint64_t)(sec_size / 4 + 1);
    // TODO: random rotr
    for(uint64_t i = 0; i < count; i++)
    {
        tmp_key = (((tmp_key & 0xFF) << 56) | (tmp_key >> 8));
        (*sec_code)[i] = (*sec_code)[i] ^ (int)tmp_key;
    }
}

int encrypt_section(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f)
{
    char* buff          = NULL;
    char* sh_str        = NULL;
    char* sh_name       = NULL;
    uint64_t sec_addr   = 0;
    uint64_t sec_size   = 0;

    buff = (char*)malloc(new_bin->elf64_shdr[new_bin->elf64_ehdr->e_shstrndx].sh_size);
    if(!buff)
        return defined_error(err_dyn_alloc, bin, opt_f);

    int fd = open(bin->filename_packed, O_RDONLY, 0400);
    if(!fd)
        return defined_error(err_fd_open, bin, opt_f);

    if(!(lseek(fd, (long)new_bin->elf64_shdr[new_bin->elf64_ehdr->e_shstrndx].sh_offset, SEEK_SET)))
        return defined_error(err_fd_seek, bin, opt_f);

    if(!(read(fd, buff, (long)new_bin->elf64_shdr[new_bin->elf64_ehdr->e_shstrndx].sh_size)))
        return defined_error(err_fd_read, bin, opt_f);
    sh_str = buff;

    int i = 0;
    for(; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        if(new_bin->elf64_shdr[i].sh_name == 0)
            continue;

        sh_name = sh_str + new_bin->elf64_shdr[i].sh_name;
        if(strcmp(opt_f->sec_name, sh_name) == 0)
        {
            sec_addr = new_bin->elf64_shdr[i].sh_addr;
            sec_size = new_bin->elf64_shdr[i].sh_size;
            printf("Section to encrypt\t: %s \n", sh_name);
            printf("Section address (in memory): 0x""%" PRIx64 "\n", sec_addr);
            // TODO : review that -->  printf("offset: 0x""%" PRIx64"\n", elf64_shdr[i].sh_offset);
            printf("Section size (bytes)\t: 0x""%" PRIx64 "\n\n", sec_size);
            break;
        }
    }

    if(i < new_bin->elf64_ehdr->e_shnum)
    {
        if(opt_f->print_sec)
        {
            printf("Hex dump of section '%s':\n", opt_f->sec_name);
            dump_section_content(new_bin->section_content[i], sec_addr, sec_size);
        }
    }
    else
        defined_error(err_sec_not_found, bin, opt_f);

    if(strncmp(opt_f->stub_name, "xor_stub", 8) == 0)
    {
        xor_section(&new_bin->section_content[i], sec_size, opt_f);
        if(opt_f->print_sec)
        {
            printf("Hex dump of xored section '%s':\n", opt_f->sec_name);
            dump_section_content(new_bin->section_content[i], sec_addr, sec_size);
        }
    }

    // printf("Hex dump of original bin '%s': ", sec_name);
    // char* original_sec = xor_section(xored_sec, (unsigned int)sh_table[i].sh_size, enc_key);
    // dump_section_content(original_sec, (unsigned int)sh_table[i].sh_size);

    free(buff);
    return 0;
}


void modify_oep(struct new_binary_s* new_bin, struct options_flags_s* opt_f, int next_sec)
{
    unsigned int new_oep = 0;           // New original entry point is the one to jump to after xor stub was completed
    uint64_t old_oep =
            new_bin->elf64_ehdr->e_entry;
    ///uint64_t xor_stub_size = 10;         
    ///uint64_t xor_stub_infos_size = 3;     
    printf("Old entry point: %lu", old_oep);

    new_bin->elf64_ehdr->e_entry = new_bin->elf64_shdr[next_sec].sh_addr;
    if(strncmp(opt_f->stub_name, "xor_stub", 8) == 0)
    {
        new_oep = old_oep - new_bin->elf64_ehdr->e_entry + xor_stub_size - xor_stub_infos_size;
        //memcpy(new_bin->section_content[next_sec] + xor_stub_size / 4 - xor_stub_infos_size / 4 + 4, &new_oep, 4);
        memcpy((new_bin->section_content[next_sec] + xor_stub_size / 4) - 8, &new_oep, sizeof(unsigned int));
    }
    printf("new entry point: %u\n", new_oep);
    ///printf("old entry point ""%" PRIx64 "\n", old_oep);
    ///printf("xor stub size ""%" PRIx64 "\n", xor_stub_size);
    ///printf("xor stub infos size ""%" PRIx64 "\n", xor_stub_infos_size);
    ///printf("next sec.sh_addr ""%" PRIx64 "\n", new_bin->elf64_shdr[next_sec].sh_addr);
}


int add_section(struct binary_s* bin, struct new_binary_s* new_bin, struct options_flags_s* opt_f)
{
    Elf64_Shdr* new_shdr    = NULL;
    uint64_t new_phdr_size  = 0;
    int last_pt_load        = -1;
    int next_sec            = -1;
    int* xor_stub           = NULL;
    int* new_sec            = NULL;
    char* tmp               = NULL;
    uint64_t sec_addr       = 0;
    uint64_t sec_size       = 0;

    ///uint64_t xor_stub_size = 100;      
    ///uint64_t entry_xor_stub = 10;     

    int i = 0;
    for(; i < new_bin->elf64_ehdr->e_phnum; i++)
    {
        if(new_bin->elf64_phdr[i].p_type == PT_LOAD)
            last_pt_load = i;
    }
    if(last_pt_load == -1)
        return defined_error(err_no_pt_load, bin, opt_f);

    Elf64_Phdr* tmp_phdr = new_bin->elf64_phdr + last_pt_load;
    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        Elf64_Shdr* tmp_shdr = new_bin->elf64_shdr + i;
        if((*tmp_shdr).sh_addr + (*tmp_shdr).sh_size >= (*tmp_phdr).p_vaddr + (*tmp_phdr).p_memsz)
            next_sec = i + 1;
    }
    if(next_sec == -1)
        return defined_error(err_no_suitable_sec, bin, opt_f);

    new_bin->elf64_shdr = (Elf64_Shdr*)realloc((void*)(new_bin->elf64_shdr),
                                               (new_bin->elf64_ehdr->e_shnum + 1) * sizeof(Elf64_Shdr));
    if(!new_bin->elf64_shdr)
        return defined_error(err_dyn_alloc, bin, opt_f);

    new_bin->section_content = (int**)realloc((void*)new_bin->section_content,
                                              (new_bin->elf64_ehdr->e_shnum + 1) * sizeof(int*));
    if(!new_bin->section_content)
        return defined_error(err_dyn_alloc, bin, opt_f);

    new_shdr = (Elf64_Shdr*)calloc(1, sizeof(Elf64_Shdr));
    if(!new_shdr)
        return defined_error(err_dyn_alloc, bin, opt_f);

    sec_addr = -1;
    sec_size = -1;
    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        char* sec_name = (char*)new_bin->section_content[new_bin->elf64_ehdr->e_shstrndx] + new_bin->elf64_shdr[i].sh_name;

        if (strcmp(opt_f->sec_name, (const char*)sec_name) == 0)
        {
            sec_addr = new_bin->elf64_shdr[i].sh_addr;
            sec_size = new_bin->elf64_shdr[i].sh_size;
            break;
        }
    }
    if(sec_addr == (uint64_t)-1 || sec_size == (uint64_t)-1)
        return defined_error(err_sec_not_found, bin, opt_f);

    // todo: modify tmp method
    tmp = opt_f->new_sec_name + strlen(opt_f->new_sec_name);
    while(opt_f->new_sec_name < tmp)
        new_shdr->sh_name = (new_shdr->sh_name << 1) + (new_shdr->sh_name << 3) + *(opt_f->new_sec_name++) - '0';

    new_shdr->sh_type       = (Elf64_Word)SHT_PROGBITS;
    new_shdr->sh_flags      = (Elf64_Word)(SHF_EXECINSTR | SHF_ALLOC);
    new_shdr->sh_addr       = (Elf64_Addr)(new_bin->elf64_phdr[last_pt_load].p_vaddr + new_bin->elf64_phdr[last_pt_load].p_memsz);
    new_shdr->sh_offset     = (Elf64_Off)(new_bin->elf64_phdr[last_pt_load].p_offset + new_bin->elf64_phdr[last_pt_load].p_memsz);
    new_shdr->sh_link       = (Elf64_Word)0;
    new_shdr->sh_info       = (Elf64_Word)0;
    new_shdr->sh_addralign  = (Elf64_Xword)0x10;
    new_shdr->sh_entsize    = (Elf64_Xword)0;

    if(strncmp(opt_f->stub_name, "xor_stub", 8) == 0)
    {
        new_shdr->sh_size = (Elf64_Xword)xor_stub_size;
        new_sec = (int*)malloc(xor_stub_size);
        if(!new_sec)
            return defined_error(err_dyn_alloc, bin, opt_f);

        xor_stub = (int*)malloc(xor_stub_size);
        if(!xor_stub)
            return defined_error(err_dyn_alloc, bin, opt_f);


        ///printf("%" PRIx64 " xor stub size\n", xor_stub_size);
        ///printf("%" PRIx64 " infos size\n", xor_stub_infos_size);
        ///printf("%" PRIx64 " enc key\n", opt_f->enc_key);
        ///printf("%p @xor stub\n", xor_stub);

        memcpy((void*)xor_stub, (const void*)entry_xor_stub, (size_t)xor_stub_size);
        //memcpy((void*)(xor_stub + xor_stub_size - 4), (const void*)&opt_f->rotr_value, sizeof(uint32_t));
        //todo -6 or -8
        memcpy((void*)(xor_stub + xor_stub_size / 4) - 6, &opt_f->enc_key, sizeof(uint64_t));
        memcpy((void*)(xor_stub + xor_stub_size / 4) - 16, &sec_size, sizeof(uint64_t));
        memcpy((void*)(xor_stub + xor_stub_size / 4) - 24, &sec_addr, sizeof(uint64_t));

        ///printf("%" PRIx64 " sec addr\n", sec_addr);
        ///printf("%" PRIx64 " sec size\n", sec_size);
        ///printf("dump %u sec: \n", new_shdr->sh_name);
        ///dump_section_content(xor_stub, new_shdr->sh_addr, new_shdr->sh_size);

        memmove(new_bin->elf64_shdr + next_sec + 1, new_bin->elf64_shdr + next_sec,
                (new_bin->elf64_ehdr->e_shnum - next_sec - 3) * sizeof(Elf64_Shdr));

        memmove(new_bin->section_content + next_sec + 1, new_bin->section_content + next_sec,
                (new_bin->elf64_ehdr->e_shnum - next_sec - 3) * sizeof(int));

        memcpy(new_bin->elf64_shdr + next_sec, new_shdr, sizeof(Elf64_Shdr));
        new_bin->section_content[next_sec] = xor_stub;

        new_phdr_size = new_bin->elf64_phdr[last_pt_load].p_memsz + xor_stub_size;
        new_bin->elf64_phdr[last_pt_load].p_memsz = new_phdr_size;
        new_bin->elf64_phdr[last_pt_load].p_filesz = new_phdr_size;
        //todo probably good instead of for loop
        //new_bin->elf64_phdr[last_pt_load].p_flags = (Elf64_Word)(PF_X | PF_R | PF_W);

        for(i = 0; i < new_bin->elf64_ehdr->e_phnum; i++)
        {
            if(new_bin->elf64_phdr[i].p_type == PT_LOAD)
                new_bin->elf64_phdr[i].p_flags = (Elf64_Word)(PF_X | PF_R | PF_W);
        }

        modify_oep(new_bin, opt_f, next_sec);
    }
    new_bin->elf64_ehdr->e_shnum += 1;
    new_bin->elf64_ehdr->e_shoff =
            new_bin->elf64_shdr[new_bin->elf64_ehdr->e_shnum - 1].sh_offset
            + new_bin->elf64_shdr[new_bin->elf64_ehdr->e_shnum - 1].sh_size;

    ///printf("##################################################################\n");

    for(i = 0; i < new_bin->elf64_ehdr->e_shnum; i++)
    {
        if( i == 26 && new_bin->section_content[i])
        {
            if(i != next_sec)
            {
                char* name = (char*)new_bin->section_content[new_bin->elf64_ehdr->e_shstrndx]
                             + new_bin->elf64_shdr[i].sh_name;
                printf("sec name : %s num: %d\n", name, i);
            }
            else
                printf("sec name: %s num: %d\n", opt_f->new_sec_name, i);

            sec_addr = new_bin->elf64_shdr[i].sh_addr;
            sec_size = new_bin->elf64_shdr[i].sh_size;
            dump_section_content(new_bin->section_content[i], sec_addr, sec_size);
            printf("\n");
        }
    }

    return 0;
}
