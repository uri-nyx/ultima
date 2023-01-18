#include "../asm/lib/master.asm"
#include "../asm/lib/sys.asm"
	mv a0, zero
	push a0, sp
	li a0, 8
	push a0, sp
	li a0, 4
	pop a1, sp
	add a0, a0, a1
	push a0, sp
	li a0, 3
	push a0, sp
	li a0, 4
	pop a1, sp
	add a0, a0, a1
	push a0, sp
	li a0, 5
	push a0, sp
	li a0, 6
	pop a1, sp
	sub a0, a1, a0
	pop a1, sp
	add a0, a0, a1
	pop a1, sp
	idiv a0, zero, a1, a0
	pop a1, sp
	sub a0, a1, a0

    halt:
        j halt