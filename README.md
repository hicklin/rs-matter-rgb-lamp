# (WIP) Matter RGB lamp

This projects builds a Matter enabled RGB lamp.

The current purpose of this project is to understand and demonstrate the user experience of the
Rust implementation of Matter (`rs-matter`) and identify pain points for future 
improvement of `rs-matter`. 
The secondary purpose of this project is to build a Matter enabled floor lamp 
for my bedroom.

This projects uses [nix devenv](https://devenv.sh/) to maintain a reproducible setup.

## Helpful commands

### Building

If using the nix devenv setup:
```
cargo build --bin rgb_lamp_wifi --target riscv32imac-unknown-none-elf --release
```

Different setups might require `+nighly`.

### Flashing

```
espflash flash target/riscv32imac-unknown-none-elf/release/rgb_lamp_wifi --baud 1500000
```

### Monitoring

This is required to get commissioning information from the device.

```
espflash monitor -elf target/riscv32imac-unknown-none-elf/release/rgb_lamp_wifi
```

