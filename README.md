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

### 1. Guest Image Preparation

The Hikami hypervisor embeds guest images (kernel, initrd, and device trees) into its binary. You must prepare these files before building the hypervisor.

#### Guest Kernel (Rockos Kernel)

- **Source**: [rockos-riscv/rockos-kernel (v6.6.87)](https://github.com/rockos-riscv/rockos-kernel/releases/tag/rockos-v6.6.87)
- This kernel is used to verify OS compatibility and run baseline benchmarks. Build it and place the resulting `vmlinux` in the appropriate directory if you wish to run a full Linux guest.

#### Initrd

The hypervisor requires an `initrd` to be present at `hikami/guest_image/initrd`.

1. Download the official Milk-V Megrez system image.
2. Mount the image and extract the `initrd` file (e.g., `initrd.img-6.6.87-win2030`).
3. Place or symlink this file to `hikami/guest_image/initrd`.

#### Evaluation Binaries

Specialized bare-metal evaluation binaries for measuring emulation overhead are automatically built and linked by our script.

### 2. Building the Hypervisor

Once the guest images are prepared:

```bash
cargo make build-hikami
```

This command:

1. Builds `emulation_overhead` binaries for QEMU and Megrez targets.
2. Links them to `hikami/guest_image/megrez/emulation_eval` and `hikami/guest_image/qemu/emulation_eval`.
3. Compiles the `hikami` hypervisor, embedding the prepared images.

**Output Binary**: `hikami/target/riscv64imac-unknown-none-elf/release/hikami`

### 3. Building Bootloaders (OpenSBI & U-Boot)

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

Replace the bootloader and add the hypervisor to the first partition (FAT32) of your SD card (e.g., `/dev/sdX1`).

```bash
sudo mount /dev/sdX1 /mnt
# Update the bootloader
sudo cp rockos-opensbi/sign/preload/bootloader_secboot_ddr5.bin /mnt/
# Add the hypervisor
sudo cp hikami/target/riscv64imac-unknown-none-elf/release/hikami /mnt/hikami.elf
sudo umount /mnt
```

---

## Running Evaluations

### Reproduction of Results

The hypervisor built with `cargo make build-hikami` has the emulation benchmarks already embedded.

1. Insert the prepared SD card into the Milk-V Megrez.
2. Connect to the serial console (115200 baud).
3. Power on the device.
4. The hypervisor will boot, start the guest, and output the benchmark results (latency measurement, etc.) to the console.

---

## Cleaning the Workspace

To remove all generated binaries and symbolic links:

```bash
cargo make clean
```
