# Sirius instruction set

## Math

### Arithmetic 0x1

OOOO_FFF_RRRRR_RRRRR_0000_0000_0000_0000_0 -> Register Register
OOOO_FFF_RRRRR_IIII_IIII_IIII_IIII_IIII    -> Register Immediate(signed)

+ Add: add, addi    0x1 0x5
+ Sub: ~~~, ~~~~    0x2 0x6
+ Div: idiv, idivi  0x3 0x7
+ Mul: mul, muli    0x4 0x8
  
### Logic 0x2

OOOO_FFF_RRRRR_RRRRR_0000_0000_0000_0000_0 -> Register Register
OOOO_FFF_RRRRR_IIII_IIII_IIII_IIII_IIII    -> Register Immediate(signed)

+ Or:  or, ori      0x1 0x5
+ And: and, andi    0x2 0x6
+ Xor: xor, xori    0x3 0x7
+ Not: not, twos    0x4 0x8

### Shifts 0x3

OOOO_FFF_RRRRR_RRRRR_0000_0000_0000_0000_0 -> Register Register
OOOO_FFF_RRRRR_IIII_IIII_IIII_IIII_IIII    -> Register Immediate(signed)


+ Arithmetic Right: sh.ra, shi.ra,  0x1 0x5
+ Logical Right:    sh.la, shi.la,  0x2 0x6
+ Logical Left:     sh.ll, shi.ll,  0x3 0x7
+ Rotate:           ror, rol,       0x4 0x8
+ ffs                               0x9
+ popcount                          0xa

## Memory 

All memory instructio accept *one* level of indirection as [ arg ]

### Loads 0x4
OOOO_FFF_RRRRR_RRRRR_0000_0000_0000_0000_0 -> Register Register
OOOO_FFF_RRRRR_IIII_IIII_IIII_IIII_IIII    -> Register Immediate(signed)

+ Bytes: lb, lbu, lbd lbud      0x1 0x4 0x7 0x9
+ 16bit: lh, lhu, lhd, lhud     0x2 0x5 0x8 0xa
+ Word:  lw, lwd                0x3 0x6
+ lui, auipc                    0xb 0xc
  
### Stores 0x5

+ Bytes: sb, sbd                0x1 0x4
+ 16bit: sh, shd                0x2 0x5
+ Word:  sw, swd                0x3 0x6

### Block 0x6

+ Copy: copy                    0x1
+ Swap: swap                    0x2
+ Literal: fill                 0x3
+ Move: mov, movi               0x4 0x5

### Stack 0x7

+ Push: pushb, pushh, push      0x1 0x3 0x5
+ Pop: popb, poph, pop          0x2 0x4 0x6

### Registers 0x8

+ Save: save                    0x1
+ Restore: restore              0x2
+ Exchange: exch                0x3

## Flow control

### Branches 0x9

+ == : beq                      0x1
+ != : bne                      0x2
+ <  : blt                      0x3
+ >=  : bge                     0x4
+ < (unsigned) : bltu           0x5
+ >= (unsigned): bgeu           0x6
+ Jumps: jal, jalr              0x7 0x9
  
## System 0xa

+ Trap: trap, syscall,                      0x1 0x4
+ SREG: gsr, ssr(SUPERVISOR)                0x2 0x5
+ Interrupt(SUPERVISOR): rti, rsyscall      0x3 0x6
