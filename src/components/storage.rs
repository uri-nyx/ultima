/*
use arch::*;

enum Commands {
    Nop,
    StoreSector,
    LoadSector,
    Query
}

struct Sector {
    d: [Byte; 512]
}

struct  Disk {
    filename: String,
    descriptor: i32, //File,
    sectors: u16
}

struct Drive {
    disks: [Disk; 16],
    current: Disk, //<- pointer!!

    iif: InterruptInterface
}
}

mod tps {
use arch::*;

struct Tps {
    filename: String,
    descriptor: i32, //FILE
}

struct Drive {
    a: Tps,
    b: Tps,
    current: Tps, //<- Pointer!!

    iif: InterruptInterface
}

enum Commands {
    Nop,
    Query(Query),
    Open,
    Close,
    StoreSector,
    LoadSector
}

enum Query {
    Bootable,
    Present
}
}
*/