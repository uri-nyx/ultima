; Write Hello world to the tty
#include "lib/master.asm"
#include "lib/sys.asm"

T_LEN = T_TX + 1
#addr 0
start:
    li  a0, le(0b1_1_0_010_111110_11111111_000000000000)
    ssreg a0
    la a0, tty_transmit_handler
    li t1, _IVT
    swd a0, IVT_TTY_TRANSMIT(t1)
    la sp, supervisor_stack
    
.receive:
    j .receive
    j .receive


tty_transmit_handler:
    addi t1, t1, .msg.len
    mv t2, zero
.transmit:
    lbu a0, .msg(t2)
    sbd t1, T_LEN(zero)
    sbd a0, T_TX(zero)
    sbd zero, T_TX(zero)
    subi t1, t1, 1
    addi t2, t2, 1
    beq t1, zero, .end
    j .transmit

.end:
    sysret

.msg:
    #d "Hello, World!\0"
    ..len = $ - .msg

#align 32
#res 1024
supervisor_stack: