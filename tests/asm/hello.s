; Hello world bare metal program for the Taleä system
#include "master.asm"
BLIT = 0x06
VIDEO = 32
BUFFER = 0x100

#addr 0

start:
    addi a2, zero, 3
.loop:
    addi a0, zero, 65
    sb a0, BUFFER(a1)
    addi a1, a1, 1
    beq a1, a2, .blit-$
    jal zero, .loop-$
.blit:
    addi a0, zero, BLIT
    shill a0, a0, 24
    ori a0, a0, BUFFER
    swd a0, VIDEO(zero)
    jal zero, start-$      ; infinite printing loop make a trap to halt?
