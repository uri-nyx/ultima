; Write Hello world to the tty
#include "lib/master.asm"
#include "lib/sys.asm"
T_LEN = T_TX + 1
#addr 0
start:
    addi t1, t1, msg.len
.transmit:
    lbu a0, msg(t2)
    sbd t1, T_LEN(zero)
    sbd a0, T_TX(zero)
    sbd zero, T_TX(zero)
    subi t1, t1, 1
    addi t2, t2, 1
    beq t1, zero, halt
    j .transmit

halt:
    j halt

msg:
    #d "Hello, World!\0"
    .len = $ - msg