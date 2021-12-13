; personal learning exercise

; part one design choices:
; maximise use of registers to avoid callouts to memory
; support input up to 12-character lines and ~10k lines

; part two (incomplete):
; 1. create a buffer of binary numbers from the original stdin list 
; 2. then perform a second pass through the buffer, measuring the position of deviation from ideal and retaining the smallest distance

global _main
extern _printf

section .text

_main:
sub rsp, 8
; initialise two vars to track line length
push 0
push 0
mov rbx, buffer
xor r12, r12
xor r13, r13
xor r14, r14
xor r15, r15

%macro m_readline 1
    mov rdx, rbx ; buffer position
    mov rdi, %1 ; net counts register
    mov rsi, rbp ; quarter
    call read_and_update
    mov rbx, rdx
    mov %1, rdi
    cmp r15, 1 ; EOL
    je .eol
    pop r9 ; get line length
    inc r9
    pop rsi
    call check_end
    push rsi ; store line length
    push r9
    inc rbp
    cmp rbp, 3
%endmacro

; rbx: buffer position
; r15: eof marker
; r12-r14: contain 12 16-bit signed ints, representing net counts
readline:
    mov rbp, 0
    .loop12:
    m_readline r12
    jle .loop12

    mov rbp, 0
    .loop13:
    m_readline r13
    jle .loop13

    mov rbp, 0
    .loop14:
    m_readline r14
    jle .loop14

    jmp      readline ; loop
.eol:
    pop r8 ; scratch
    pop r9 
    mov r9, r8
    push r9
    push 0
    jmp      readline ; loop

; inputs:
; rdi: net counts register
; rsi: quarter of register
; rdx: buffer position
; r15: eof/eol marker
; outputs:
; rdi: net counts register
; rdx: buffer position
; r15: eof/eol marker
read_and_update:
    push rbp
    mov rbp, rsp
    sub rsp, 8
    push r12
    push r13
    push r14
    mov r12, rdi ; net counts register
    mov r13, rsi ; quarter
    mov r14, rdx ; buffer pos

    ; get existing net count register
    ;get16si
    ;rdi net counts register
    mov rdi, r12
    ;rsi quarter
    mov rsi, r13
    call     get16si
    ; update net count based on next char
    ;readchar
    ;rsi net counts value 
    mov rsi, rdi
    ;rdi buffer pos
    mov rdi, r14
    call     readchar
    mov r14, rdi
    ; store inside 64-bit register
    ; put16si
    ;rdx: net count value
    mov rdx, rsi ; readchar net count value output
    ;rsi: quarter
    mov rsi, r13
    ;rdi: net count register
    mov rdi, r12
    call     put16si
    mov rdx, r14
    pop r14
    pop r13
    pop r12
    leave
    ret

; problem: line length gets reset to 0 on eol directly before eof; out of registers?
; inputs:
; r15: EOL/EOF marker
; rsi: line length
; r12-14: storage registers
; outputs:
; r15: EOL/EOF marker
; rsi: line length
; r12-14: storage registers
check_end:
    push rbp
    mov rbp, rsp
    push r12
    push r13
    mov rdi, 12 ;max 12-bit inputs supported
    sub rdi, rsi
    mov r12, rsi
    mov r13, rdi
    mov rsi, r12
    mov rdi, r13
    pop r13
    pop r12
    cmp      r15, 2
    je       calculate_summary_vars ; finish if stream ends
.check:
    cmp      rsi, buffersize ; TODO support chunking stdin instead of ending after buffer
    je       calculate_summary_vars ; finish if buffer ends
    leave
    ret

; read one byte from stdin, alter net count, increase mem pos
; inputs:
; rdi: memory position
; si: net count
; r15: end of line/stream
; outputs:
; rdi: memory position
; si: net count
; r15: end of line/stream
readchar:
    push    rbp
    mov rbp, rsp
    push    r12
    push    r13
    push    r14
    push    r15
    mov r12, rdi
    mov r13, rsi
    mov r14, r15 ; eol/eof marker
    mov     rax, 0x2000003 ; read
    mov     rsi, r12
    mov     rdi, 0 ; stdin
    mov     rdx, 1
    syscall
    cmp     rax, 0
    je      .endstream ; read returned EOF
    mov     r15, [r12]
    cmp     r15, `\n`
    je      .endline ; read returned \n
    mov     r14, 0
    cmp     r15, "0"
    je      .inc0
    cmp     r15, "1"
    je      .dec0
    jmp     .end
.inc0:
    inc     r13w
    jmp     .end
.dec0:
    dec     r13w
    jmp     .end
.endline:
    mov     r14, 1
    jmp     .end
.endstream:
    mov     r14, 2
    jmp     .end
.end:
    inc r12
    mov rdi, r12
    mov rsi, r13
    pop     r15
    mov r15, r14
    pop     r14
    pop     r13
    pop     r12
    leave
    ret

; inputs:
; r12-r14: 16-bit net counts of 0s
; rdi: number of bits to truncate
; outputs:
; r8: gamma
; r10: epsilon
; r11: oxygen generator rating bit criteria
; rdx: co2 scrubber rating bit criteria
calculate_summary_vars:
    push r12
    push r13
    push r14
    sub rsp, 8
    push rdi
    push r12
    sub rsp, 8
    push r13
    sub rsp, 8
    push r14
    sub rsp, 8
    mov r8, 0 ; gamma
    mov r9, 0b1 ; 1-mask
    mov r10, 0 ; epsilon
    mov r11, 0 ; oxygen
    mov rdx, 0 ; co2
    mov rbp, 2
.loop1:
    add rsp, 8
    pop r15
    mov rbx, 3
.loop2:
    mov rdi, r15
    mov rsi, rbx
    push r8
    push r9
    push r10
    call get16si
    pop r10
    pop r9
    pop r8
    mov rsi, rdi
    ; si > 0: 1 is more common; flip epsilon bit; flip co2 bit
    ; si < 0; 0 is more common; flip gamma bit; flip oxygen bit
    ; si == 0; same number of 0s and 1s; flip oxygen bit
    cmp     si, 0
    jg      .positive
    je      .equal
    or      r8, r9 
    or      r11, r9
    jmp     .continue
.equal:
    or      r11, r9
    jmp     .continue
.positive:
    or      r10, r9 
    or      rdx, r9
.continue:
    shl r9, 1
    dec rbx
    cmp rbx, 0
    jge .loop2
    dec rbp
    cmp rbp, 0
    jge .loop1
    pop rcx ; bits to truncate
    shr r8, cl
    shr r10, cl
    shr r11, cl
    shr rdx, cl

; prints the power consumption as a decimal value
; inputs:
; r8: gamma
; r10: epsilon
power_consumption:
    mov rax, r10
    mul r8
    mov r12, r8
    mov r13, r10
    mov r14, rax
    ;mov rsi, r12
    ;call mark3
    ;mov rsi, r13
    ;call mark3
    mov rsi, r14
    call mark
    mov rsi, r12
    mov rdi, r13
    mov rdx, r14
    add rsp, 8
    pop r14
    pop r13
    pop r12
    ;jmp exit

exit:
    mov     rax, 0x2000001 ; exit
    mov     rdi, 0
    syscall

; get a 16-bit signed int from a 64-bit register
; move the wanted 16 bits to the right and pad the rest with 0s
; inputs:
; rdi: register to use
; rsi: quarter to use, 0-3
; outputs:
; rdi: 16 bits of data with 48 left-padded zeroes
get16si:
    push rbp
    mov rbp, rsp
    mov     rcx, rdi
    mov     rdx, rsi
.quarter:
    cmp rdx, 0
    je .continue
    shr     rcx, 16; shift right
    dec rdx
    jmp .quarter
.continue:
    and     rcx, 0xffff ; apply mask
    mov     rdi, rcx
    leave
    ret

; place a 16-bit signed int into a 64-bit register without disturbing the other bits
; create a mask, use it to zero out the target bits, shift the int left as needed, and with target
; inputs:
; rdi: register to change
; rsi: quarter to use
; rdx: 16-bit int
; outputs:
; rdi: changed register
put16si:
    push rbp
    mov rbp, rsp
    sub rsp, 8
    mov rcx, 0x000000000000ffff ; mask
.quarter:
    cmp rsi, 0
    je .continue
    shl rdx, 16; shift left
    shl rcx, 16; shift mask left
    dec rsi
    jmp .quarter
.continue:
    and     rdx, rcx ; apply mask
    not     rcx ; reverse mask
    and     rdi, rcx ; apply reverse mask to zero out target bits
    add     rdi, rdx
    leave
    ret

; printf rsi
; inputs:
; rsi: var to print
; outputs: none
mark:
    push rbp
    mov rbp, rsp
    mov     rdi, var1dump
    mov     rax, 1
    call    _printf
    leave
    ret

; inputs:
; outputs:
mark3:
    push rbp
    mov rbp, rsp
    mov rdi, var2dump
    mov rax, 1
    call _printf
    leave
    ret

section .data

fmt: db "net 0s: %hi, most common: %d", 10, 0

var1dump:   db      "%d", 10, 0
var2dump:   db      "0x%lx", 10, 0

buffersize: equ 1

section .bss
buffer: resw buffersize
