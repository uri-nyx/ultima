# The Teletype System

The *teletype system* is the simplest way to interface with the *Taleä Computer System*. It is composed of a printer and a keyboard that send and receive text data over a serial connection.

## PORTS

Note that the transmit (TX) and receive (RX) ports are labeled here from the CPU's point of view (if we took the serial controler side, they would be reversed).

    ╭──────┬──────────┬─────╮
    │RX    │ halfword │ 0x00│
    ├──────┼──────────┼─────┤
    │TX    │ halfword │ 0x02│ <- Notice, only a byte is used
    ├──────┼──────────┼─────┤
    │STATUS│ byte     │ 0x04│
    ├──────┼──────────┼─────┤
    │CTRL  │ byte     │ 0x05│
    ╰──────┴──────────┴─────╯

## Sending a byte to the TTY

Sending a byte to the tty is easy, one just needs to write it to the TX port.

## Receiving data from the TTY

An interrupt will be fired when there is data incoming from the TTY at *priority level* 4. Reading from `RX[0]` will transfer a byte to the system, whereas reading from `RX[1]` will return the number of remaining incoming bytes. Notice that subsequent reads from `RX[0]` will return the bytes of the input in a LIFO fashion, until the buffer is exhausted, when it will read `0x00`.

## Controlling the tty

WORK IN PROGGRESS
