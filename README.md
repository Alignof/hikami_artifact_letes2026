# Hikami Artifact (LCTES 2026)

This repository contains the artifacts for the paper "Hikami: A Lightweight Hypervisor for RISC-V with Extension Emulation" (LCTES 2026).

## Repository Structure

- `hikami`: The core implementation of the proposed hypervisor.
- `hikami_zbs`: Emulation module for the Zbs extension.
- `evaluation/`: Benchmark and evaluation code.
- `rockos-u-boot/`: Second-stage bootloader.
- `rockos-opensbi/`: Third-stage bootloader (modified for Hikami).
- `lctes26-paper45.pdf`: The accepted paper.
- `ozora`: Tools for automatic generation of hypervisor modules from Sail specifications.
- `sail-riscv`: Formal specification of the RISC-V ISA in Sail.

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

**Output Binary**: `hikami/target/riscv64imac-unknown-none-elf/release/hikami`

### 3. Building Bootloaders (OpenSBI & U-Boot)

```bash
cargo make build-bootloaders
```

**Output Binary**: `rockos-opensbi/sign/preload/bootloader_secboot_ddr5.bin`

---

## Deployment and Hardware Setup

### 1. Prepare the Base Disk Image

Download the base system image for Milk-V Megrez from the official releases:
[Milk-V Megrez Build Releases](https://github.com/milkv-megrez/megrez-build/releases/)

Flash the downloaded image to an SD card (e.g., using `dd` or BalenaEtcher).

### 2. Install Artifacts to the SD Card

You can use the `install` task to copy the built bootloader and hypervisor to the SD card.

Set the `DEVICE` environment variable to your SD card device (e.g., `/dev/sdX`). The script will mount the first partition (`${DEVICE}1`) and copy the artifacts.

```bash
# Example: If your SD card is /dev/sdb
DEVICE=/dev/sdb cargo make install
```

This command will:

1. Mount `${DEVICE}1` to `/mnt`.
2. Copy the signed bootloader (`bootloader_secboot_ddr5.bin`).
3. Copy the hypervisor as `hikami.elf`.
4. Unmount the partition.

---

## Running Evaluations

### Reproduction of Results

The hypervisor built with `cargo make build-hikami` has the emulation benchmarks already embedded.

1. Insert the prepared SD card into the Milk-V Megrez.
2. Connect to the serial console (115200 baud).
3. Power on the device.
4. The hypervisor will boot, start the guest, and output the benchmark results (latency measurement, etc.) to the console.

---

## Extension Emulation Code Generation

Hikami uses **Ozora** to automatically generate emulation code from formal RISC-V specifications written in Sail. This ensures the correctness of the emulation logic.

To verify the code generation for the Zbs extension:

1. Navigate to the `ozora` directory.
2. Ensure you have the `sail` compiler and its dependencies installed (see `ozora/README.md` for details).
3. Run the following command:

```bash
cd ozora
cargo r riscv_insts_zbs.sail target/zbs.rs --ext-name Zbs
```

This command processes the Zbs specification (`riscv_insts_zbs.sail`) and generates the corresponding Rust code in `ozora/target/`. The generated structure is as follows:

- `ozora/target/zbs.rs`: The core emulation module.
- `ozora/target/instruction/zbs_extension.rs`: Instruction definitions used for the decoder.
- `ozora/target/decode/zbs_extension.rs`: The implementation of the decoder itself.

### Verification of Generated Code

To ensure the generated code is correct, you can compare it with the code currently used in Hikami and Raki. First, format the generated code to match the project's style:

```bash
rustfmt ozora/target/zbs.rs
rustfmt ozora/target/instruction/zbs_extension.rs
rustfmt ozora/target/decode/zbs_extension.rs
```

Then, use `delta` (or your preferred diff tool) to verify that the generated logic matches the manually integrated code:

```bash
# Compare emulation logic
delta hikami_zbs/src/lib.rs ozora/target/zbs.rs

# Compare decoder implementation
delta raki/src/decode/zbs_extension.rs ozora/target/decode/zbs_extension.rs

# Compare instruction definitions
delta raki/src/instruction/zbs_extension.rs ozora/target/instruction/zbs_extension.rs
```

---

## Cleaning the Workspace

To remove all generated binaries and symbolic links:

```bash
cargo make clean
```
