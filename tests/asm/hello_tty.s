; Write Hello world to the tty
#include "lib/master.asm"
#include "lib/sys.asm"
T_LEN = T_TX + 1
#addr 0
start:
    addi t1, t1, msg.len
.transmit:
    lbu a0, msg(t2)
    beq a0, zero, halt
    sbd a0, T_TX(zero)
    addi t2, t2, 1
    j .transmit

halt:
    j halt

msg:
    #d "Hello, World!\0"
    .len = $ - msg