# Custom Devices

Custom devices can be added easily to the system with certain constraints (aside from emulating them and integrating them in the emulator):

1. They must be either memory mapped or implement some sort of port device that is. There is no native port i/o for Taleä.
2. They may access memory and have their own clock frequencies, but the interaction with the cpu may only be via interrupts.

The area in data memory from 0x002c to 0x00ff shall be reserved for installing custom devices, though they can be installed in other region at will.

