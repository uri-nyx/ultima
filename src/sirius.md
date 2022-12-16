# Sirius instruction set

## Math

### Arithmetic

+ Add: add, addi
+ Sub: ~~~, ~~~~
+ Div: idiv, idivi
+ Mul: mul, muli
  
### Logic

+ Or:  or, ori
+ And: and, andi
+ Xor: xor, xori
+ Not: not, twos

### Shifts

+ Arithmetic Right: sh.ra, shi.ra,
+ Logical Right:    sh.la, shi.la,
+ Logical Left:     sh.ll, shi.ll,
+ Rotate:           ror, rol,
+ ffs
+ popcount

## Memory

All memory instructio accept *one* level of indirection as [ arg ]

### Loads

+ Bytes: lb, lbu, lbd lbud
+ 16bit: lh, lhu, lhd, lhud
+ Word:  lw, lwd
+ lui, auipc
  
### Stores

+ Bytes: sb, sbd
+ 16bit: sh, shd
+ Word:  sw, swd

### Block

+ Copy: copy
+ Swap: swap
+ Literal: fill
+ Move: mov

### Stack

+ Push: pushb, pushh, push
+ Pop: popb, poph, pop

### Registers

+ Save: save
+ Restore: restore
+ Exchange: exch

## Flow control

### Branches

+ == : beq
+ != : bne
+ <  : blt
+ >=  : bge
+ < (unsigned) : bltu
+ >= (unsigned): bgeu
+ Jumps: jal, jalr
  
## System

+ Trap: trap, syscall,
+ SREG: gsr, ssr(SUPERVISOR)
+ Interrupt(SUPERVISOR): rti, rsyscall
