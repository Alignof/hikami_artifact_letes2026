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

## Build Guide

### 1. Guest Image Preparation

#### Guest Kernel

The guest Linux kernel used in this artifact is based on the **Rockos Kernel (v6.6.87)**.

- **Source**: [rockos-riscv/rockos-kernel (v6.6.87)](https://github.com/rockos-riscv/rockos-kernel/releases/tag/rockos-v6.6.87)
- Build the kernel according to its instructions and place the resulting `vmlinux` or binary in the appropriate directory if you wish to run a full Linux guest.
- This kernel is used to verify OS compatibility on the proposed system and to run benchmarks using QEMU (as a baseline comparison system) on the device.

#### Evaluation Binaries

For the specific evaluations described in the paper (e.g., emulation overhead), we use specialized bare-metal evaluation binaries. These are automatically built by the top-level build script.

### 2. Building the Hypervisor

We provide a root `Makefile.toml` to automate the build process, including building the evaluation binaries, linking them as guest images, and compiling the hypervisor.

To build the hypervisor with the evaluation binaries embedded:

```bash
cargo make build-hikami
```

This command executes the following steps:

1. Enters the `evaluation/` environment via Nix and builds the `emulation_overhead` binaries for both QEMU and Megrez targets.
2. Creates symbolic links in `hikami/guest_image/` to these binaries.
3. Enters the `hikami/` environment via Nix and builds the hypervisor with the `identity_map` feature enabled.

**Output Binary**: `hikami/target/riscv64imac-unknown-none-elf/release/hikami`

### 3. Building Bootloaders (OpenSBI & U-Boot)

```
/* Instructions for building `rockos-opensbi` and `rockos-u-boot` will be added here. */
```

---

## Deployment and Hardware Setup

```
/* Instructions for writing the hypervisor and bootloaders to an SD card or flash memory will be added here. */
```

## Running Evaluations

```
/* Instructions for executing the benchmarks and reproducing the results from the paper will be added here. */
```

---

## Cleaning the Workspace

To remove all generated binaries and symbolic links:

```bash
cargo make clean
```
