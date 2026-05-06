# Hikami Artifact (LCTES 2026)

This repository contains the artifacts for the paper "Hikami: A Lightweight Hypervisor for RISC-V with Extension Emulation" (LCTES 2026).

## Repository Structure

- `hikami`: The core implementation of the proposed hypervisor.
- `hikami_zbs`: Emulation module for the Zbs extension.
- `evaluation/`: Benchmark and evaluation code.
- `rockos-u-boot/`: Second-stage bootloader.
- `rockos-opensbi/`: Third-stage bootloader (modified for Hikami).
- `lctes26-paper45.pdf`: The accepted paper.

## Prerequisites

- [Nix](https://nixos.org/download.html) (with flake support enabled)
- [cargo-make](https://github.com/sagiegurari/cargo-make)

---

## Build Guide

We provide a root `Makefile.toml` to automate the entire build process.

To build everything (Hypervisor, Evaluation Binaries, U-Boot, and OpenSBI):

```bash
cargo make build-all
```

### 1. Building the Hypervisor & Evaluation Binaries

```bash
cargo make build-hikami
```

This command:

1. Builds `emulation_overhead` binaries for QEMU and Megrez targets.
2. Links them to `hikami/guest_image/`.
3. Compiles the `hikami` hypervisor.

**Output Binary**: `hikami/target/riscv64imac-unknown-none-elf/release/hikami`

### 2. Building Bootloaders (OpenSBI & U-Boot)

```bash
cargo make build-bootloaders
```

This command:

1. Builds U-Boot for the Milk-V Megrez.
2. Builds OpenSBI using U-Boot as a payload.
3. Signs the resulting binary using `nsign`.

**Output Binary**: `rockos-opensbi/sign/preload/bootloader_secboot_ddr5.bin`

---

## Deployment and Hardware Setup

### 1. Prepare the Base Disk Image

Download the base system image for Milk-V Megrez from the official releases:
[Milk-V Megrez Build Releases](https://github.com/milkv-megrez/megrez-build/releases/)

Flash the downloaded image to an SD card (e.g., using `dd` or BalenaEtcher).

### 2. Write Artifacts to the SD Card

After flashing the base image, you need to replace the bootloader and add the hypervisor.

Assuming your SD card is identified as `/dev/sdX`:

#### Write the Bootloader (Partition 1)

Mount the first partition (FAT32) and copy the signed bootloader:

```bash
sudo mount /dev/sdX1 /mnt
sudo cp rockos-opensbi/sign/preload/bootloader_secboot_ddr5.bin /mnt/
sudo umount /mnt
```

#### Write the Hypervisor (Partition 1)

Copy the built `hikami` binary (as `hikami.elf`) to the same partition. U-Boot is configured to load this file:

```bash
sudo mount /dev/sdX1 /mnt
sudo cp hikami/target/riscv64imac-unknown-none-elf/release/hikami /mnt/hikami.elf
sudo umount /mnt
```

#### (Optional) Write the Guest Kernel (Rockos Kernel)

If you wish to run the full Rockos Linux guest, build the [Rockos Kernel (v6.6.87)](https://github.com/rockos-riscv/rockos-kernel/releases/tag/rockos-v6.6.87) and place its `vmlinux` in the boot partition or rootfs as required by your guest configuration.

---

## Running Evaluations

### Reproduction of Results

The hypervisor built with `cargo make build-hikami` has the emulation benchmarks already embedded. When you boot the Milk-V Megrez with this hypervisor, it will automatically execute the embedded benchmarks.

1. Insert the prepared SD card into the Milk-V Megrez.
2. Connect to the serial console (115200 baud).
3. Power on the device.
4. The hypervisor will boot, start the guest, and output the benchmark results to the console.

---

## Cleaning the Workspace

To remove all generated binaries and symbolic links:

```bash
cargo make clean
```
