# (WIP) Matter RGB lamp

**NOTE** This project is still very much a playground.

This projects builds a Matter enabled RGB lamp.

The current purpose of this project is to understand the user experience of the 
Rust implementation of Matter (`rs-matter`) and identify pain points for future 
improvement of `rs-matter`. 
The secondary purpose of this project is to build a Matter enabled floor lamp 
for my bedroom.

This projects uses [nix devenv](https://devenv.sh/) to maintain a reproducible setup.

## Helpful commands

### Building

If using the nix devenv setup:
```
cargo build --bin rgb_lamp_wifi --target riscv32imac-unknown-none-elf --no-default-features --features esp32c6
```

Different setups might require `+nighly`.

### Flashing

```
espflash flash target/riscv32imac-unknown-none-elf/debug/rgb_lamp_wifi --baud 1500000
```

### Monitoring

This is required to get commissioning information from the device.

```
espflash monitor -elf target/riscv32imac-unknown-none-elf/debug/rgb_lamp_wifi 
```