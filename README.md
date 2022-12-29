---
title: The Taleä Computer System
author: Uri Nyx
date: '2022'
mainfont: Palatino, Palatino Linotype, serif
---

# The Taleä Computer System

## *A Fantasy Computer for a Fantasy World*

This is the specification for one of the two families of computers that exist in the fantasy world of *Nar Naellan* (the other one is the [Machine of the Archive of Arkade](about:blank) *not yet even designed*).

It is a fantasy computer inspired in the first PCs, such as the machines from IBM, Apple or Amiga. It is
used in a fantasy world, and was designed to serve every need that may arise
to any successful merchant business across the sea.

It belongs to the family of Taleä architectures, and stands as the most powerful system
for consumers in its range. It shares it's architecture and instruction set with whe less
powerful Taleä Codex.

Taleä Tabula uses a 32-bit processor, clocked at almost 10Mhz, that can adress up to 16Mb of memory. The
system features an 640x480 256 color screen with a VGA-like driver, a printer style teletype, a keyboard, and a
128 Mib disk.

It's main purpose is, opposed to that of the Machine of the Archive of Arkade, accounting and basic data processing, to be used in commerce and banking by merchants and companies of Talandel. It is cheaper and smaller than the one in the Academy, and a net of these standard machines, once tested, has begun spreading over the city and some of its colonies.

You may find here the [User's Manual]()

## About this Emulator

~~The emulator is implemented in C, following the [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming) paradigm with [Literate](https://zyedidia.github.io/literate/index.html), as should be for a literary machine.The only dependencies so far are [SDL2](https://www.libsdl.org/download-2.0.php) and [Inprint](https://github.com/driedfruit/SDL_inprint). It *should* compile and run on all major OSes with support for SDL2.~~~

As of December 2022, the main branch of the emulator is implemented in Rust. The design is based (the general architecture is copied) from [Moa](https://www.github.com/transistorfet/moa). It uses the [Pixels]() crate as its main dependency, but take a look at `Cargo.toml` for a complete list of dependencies. Sadly it is no more a literary program, but it should be cross-platform (even *WebAssembly*, with some important tweaking)

[The documentation for the emulator lays here]()

## Changelog

+ **December 2022**: This project has suffered a lot of reworks and completely new rewrites and restarts over the last four years. I hope this is the last one. This implementation of the emulator, though far from being even decent, is probably the most robust and well written program I've yet managed. Let's see if we get to OS this time. The instruction set has changed, but only by grace of extension, and is a carbon copy of RISC-V I32, with some additions, and a custom ordering of the instructions.
+ **June 2022**: Since the last commit nearly a year ago this project is indeed a completely different one. It has retained only the history and the name.