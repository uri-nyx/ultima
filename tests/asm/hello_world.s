; Hello world bare metal program for the Taleä system
#include "lib/master.asm"
#include "lib/sys.asm"

#addr 0

start:
    jal ra, puts
    j start

puts:
    lbu t1, hello_str(a0)   ; get the next character from the string
    beq t1, zero, .end      ; if it's zero, we're done
    jal t2, .print_char     ; print the character return address is in t2
    addi a0, a0, 1          ; increment the string pointer
    jal zero, puts          ; loop

.print_char:
    sb t1, charbuff(a0)
    jalr zero, 0(t2)        ; return to the puts function

.end:
    la a0, charbuff 
    swd a0, V_COMMAND(zero)
    addi a0, zero, V_blit
    sbd a0, V_COMMAND(zero)
    jalr zero, 0(ra)        ; return to the start function

hello_str:  #d "Hello, world!\0"

charbuff:
    #res 2000