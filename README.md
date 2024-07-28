# hwfetch - A fast and accurate Hardware Fetch

hwfetch is designed to quickly and accurately gather and display detailed
information about your system's hardware.

## Installation

### Arch-based Distributions

If you're using an Arch-based distribution, you can easily install **hwfetch**
from the AUR using your favorite helper:

```bash
paru -S hwfetch-git
```

### NixOS

/// TODO

### Manual Installation

First of all make sure you have both rust and just installed.

Then simply clone the repository and run:

```bash
just build && just install
```

## Usage

Once you have **hwfetch** installed, just open your terminal and type:

```bash
hwfetch
```

This will print a detailed and beautifully formatted summary of your system's
hardware information. For more information see the configuring section

## Acknowledgments

A huge thank you to [Penny](https://github.com/pennybelle/) for her invaluable
help with the printing logic. And a shout-out to
[pterror](https://github.com/pterror) for adding Nix support.
