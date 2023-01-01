#include "lib/master.asm"
#include "lib/sys.asm"

comm =  (TPS_load << 24 | 1)

start:
    addi a0, a0, TPS_open
    sbd a0, TPS_COMMAND(zero)
    li a0, comm
    swd a0, TPS_COMMAND(zero)
    j halt

halt:
    j halt