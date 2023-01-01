; Bare metal graphic hello world program for the Taleä system
; This program is a more efficient aproach to the previous example
; It uses the BLIT video module command and an in memory buffer
#include "lib/master.asm"
#include "lib/sys.asm"

    #addr 0
start:
    li t1, (640 * 480)      ; load end of buffer in t1
    addi a0, zero, V_setmode
    sbd a0, V_COMMAND(zero)
    addi a0, zero, 2        ; grphic mode
    sbd a0, V_DATAH(zero)
    li t3, 100_000_000         ; implement better timers than this one, and make the video more responsive, a queue of comands would be neat
.wait:
    addi t4, t4, 1
    add zero, zero, zero
    bne t4, t3, .wait

.color:
    addi a0, a0, 65
    la t3, buffer
    fill t3, t1, a0
.end:
    la a0, buffer 
    swd a0, V_COMMAND(zero)
    addi a0, zero, V_blit
    sbd a0, V_COMMAND(zero)
halt:
    j halt


buffer:
    #res (640 * 480)
