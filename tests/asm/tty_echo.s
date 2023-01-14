; Write Hello world to the tty
#include "lib/master.asm"
#include "lib/sys.asm"

RX_LEN = T_RX + 1
IVT_TTY_EOT = 0xf
#addr 0
start:
    li  a0, le(0b1_1_0_010_111110_11111111_000000000000)
    ;          0b1_1_0_101_111110_11111111_000000000000 
    ssreg a0
    la a0, tty_transmit_handler
    li t1, _IVT
    swd a0, IVT_TTY_TRANSMIT(t1)
    la sp, supervisor_stack
    addi t1, zero, 1
    .receive:
        j .receive


tty_transmit_handler:
    pushb zero, sp      ; NULL terminator
    
    .transmit:
        lbud a0, T_RX(zero)
        pushb a0, sp
        lbud a0, RX_LEN(zero)
        beq zero, a0, .print
        j .transmit

    .print:
        popb a0, sp
        beq a0, zero, .end
        sbd a0, T_TX(zero)
        j .print

    .end:
        addi a0, zero, 10 ; NEWLINE
        sbd a0, T_TX(zero)
        sysret


#res 1024
supervisor_stack: