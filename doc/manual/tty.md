# The Teletype System

The *teletype system* is the simplest way to interface with the *Taleä Computer System*. It is composed of a printer and a keyboard that send and receive text data over a serial connection.

## PORTS

Note that the transmit (TX) and receive (RX) ports are labeled here from the CPU's point of view (if we took the serial controler side, they would be reversed).

    ╭──────┬──────────┬─────╮
    │RX    │ halfword │ 0x00│
    ├──────┼──────────┼─────┤
    │TX    │ halfword │ 0x02│
    ├──────┼──────────┼─────┤
    │STATUS│ byte     │ 0x04│
    ├──────┼──────────┼─────┤
    │CTRL  │ byte     │ 0x05│
    ╰──────┴──────────┴─────╯

## Sending a byte to the TTY

Sending a byte to the tty is easy, one just needs to write the character code to the TX port and spcify the number of bytes remaining from the message (in the low order byte of the port).

## Receiving data from the TTY

An interrupt will be fired when there is data incoming from the TTY at *priority level* 3. It shall be read acknowleding the transmision by writing to the `ACK` flag in the `STATUS` register (least significant byte, active high), and reading from the `RX` port.

## Controlling the tty

WORK IN PROGGRESS
