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

This repo has a Nix flake.

<details>
<summary>Detailed Nix instructions</summary>

```nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    hwfetch.url = "github:morr0ne/hwfetch";
    hwfetch.inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

It defines the `hwfetch` package, and `default` as an alias to `hwfetch`. It
also defines a `default` shell.

It can be used in your system installation:

```nix
environment.systemPackages = [
  inputs.hwfetch.packages.${pkgs.system}.hwfetch
  # or
  inputs.hwfetch.packages.${pkgs.system}.default
];
```

</details>

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

## License

This project is licensed under the
[Apache-2.0 License](http://www.apache.org/licenses/LICENSE-2.0). For more
information, please see the [LICENSE](LICENSE) file.
