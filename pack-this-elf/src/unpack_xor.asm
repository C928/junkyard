[BITS 64]
section .text

global entry_xor_stub:function
global xor_stub_size:data
global xor_stub_infos_size:data

xor_stub_size           dq      end_xor_stub - entry_xor_stub
xor_stub_infos_size     dq      end_xor_stub - infos_xor_stub

entry_xor_stub:
    push rax
    push rbx
    push rcx
    push rdx
    mov rax, 0
    mov rbx, [rel sec_addr]
    mov rcx, [rel sec_size]
    mov rdx, [rel enc_key]
    loop:
        xor byte [rbx + rax * 8], dl
        ror rdx, 8
        inc rax
        cmp rax, rcx
        jnz loop
        nop
        nop
        jmp 0xcccccccc
    pop rax
    pop rbx
    pop rcx
    pop rdx

infos_xor_stub:
sec_addr:       dq      0xaaaaaaaaaaaaaaaa
sec_size:       dq      0xbbbbbbbbbbbbbbbb
enc_key:        dq      0xa5a5a5a5a5a5a5a5

end_xor_stub:


