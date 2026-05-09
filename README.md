# Artifact Evaluation: Hikami (LCTES 2026)

This artifact supports the paper: **"Hikami: A Lightweight Hypervisor for Emulating RISC-V Extension Semantics with Sail-Driven Auto-Generation"** (Paper #45).

## 1. Getting Started Guide

### Prerequisites

#### Hardware
- **Milk-V Megrez**: This is the primary evaluation platform (RISC-V 64-bit with Hypervisor extension support).
- **Serial Console**: USB-to-TTL cable or a Type-C to Type-A cable (115200 baud) for interacting with the board. You can use tools like `picocom` (e.g., `picocom -b 115200 /dev/ttyUSB0`).
- **SD Card**: At least 8GB, used for booting the hypervisor and guest OS.

#### Software
- **Nix**: Used for managing the build environment and dependencies.
  ```bash
  curl -L https://nixos.org/nix/install | sh
  ```
- **Linux Host**: For building the artifacts and flashing the SD card.
- **tokei**: For SLoC measurement (can be installed via `cargo install tokei` or your package manager).
- **binutils**: `readelf` is required for binary analysis.

### Basic Test: Building the Hypervisor

To verify that the environment is set up correctly, build the entire project:

1.  **Clone the repository and submodules:**
    ```bash
    git submodule update --init --recursive
    ```

2.  **Build everything:**
    This command will build the hypervisor, evaluation binaries, U-Boot, and OpenSBI.
    ```bash
    cargo make build-all
    ```
    *Note: The first run may take some time as Nix downloads dependencies.*

3.  **Verify Binaries:**
    Check that the following key binaries were generated:
    - `hikami/target/riscv64imac-unknown-none-elf/release/hikami`
    - `rockos-opensbi/sign/preload/bootloader_secboot_ddr5.bin`

### Using Pre-built Binaries (Optional)

If you wish to skip the build process, we provide pre-built binaries in the `binaries/` directory:
- `binaries/hikami.elf`: The core hypervisor.
- `binaries/bootloader_secboot_ddr5.bin`: The signed bootloader (OpenSBI + U-Boot).
- `binaries/megrez/*.eval`: Evaluation binaries for performance measurement.

---

## 2. Step-by-Step Instructions

### Experiment 1: SLoC and Binary Size Measurement (Section 4.2, Table 5)

This experiment verifies the resource efficiency of the hypervisor as described in the paper.

1.  **Source Lines of Code (SLoC):**
    Run `tokei` on the hypervisor components:
    ```bash
    tokei hikami hikami_zbs
    ```
    *Note: Table 5 reports ~7,000 lines of Rust code total (Core Library + Hypervisor Binary + Zbs Module).*

2.  **Binary Size and Memory Footprint:**
    Use `readelf` to inspect the hypervisor binary (either your built one or the pre-built one):
    ```bash
    readelf -S binaries/hikami.elf
    ```
    Observe the size of the `.text` section. As mentioned in Section 4.2, the `.text` section occupies approximately 84 KiB.

### Experiment 2: Measuring Emulation and Interrupt Latency (Table 7, Table 8, Section 4.3.2)

This experiment reproduces the performance results on the **Milk-V Megrez** hardware.

1.  **Flash the SD Card:**
    Download a base Megrez image, flash it to an SD card, and then install the artifacts. 

    **Option A: Automated Install (After build)**
    ```bash
    DEVICE=/dev/sdX cargo make install  # Replace /dev/sdX with your SD card device
    ```

    **Option B: Manual Install (Using pre-built binaries)**
    ```bash
    sudo mount /dev/sdX1 /mnt
    sudo cp binaries/bootloader_secboot_ddr5.bin /mnt/
    sudo cp binaries/hikami.elf /mnt/hikami.elf
    sudo umount /mnt
    ```

2.  **Run the Benchmark:**
    - Insert the SD card into the Megrez and connect the serial console.
    - Power on the board.
    - The hypervisor will boot and automatically start the embedded evaluation guest.
    - Observe the console output for "Ratio Average" and "Interrupt Latency" results.

3.  **Switching Evaluation Binaries:**
    The hypervisor embeds the guest binary at compile-time. To switch between different evaluation binaries (e.g., for different experiments), you must modify `hikami/src/target_board/megrez.rs` and rebuild.
    Example of switching to a specific binary:
    ```rust
    // In hikami/src/target_board/megrez.rs
    pub static GUEST_KERNEL: [u8; include_bytes!("../../guest_image/megrez/emulation_eval").len()] =
        *include_bytes!("../../guest_image/megrez/emulation_eval");
    ```

4.  **Expected Output:**
    - **Interrupt Latency**: Should be around 1.0 $\mu$s for timer and 0.5 $\mu$s for external interrupts.
    - **Emulation Overhead**: CoreMark scores should show Hikami at >99% of native performance.

### Experiment 3: Full Linux Boot (Table 6)

1.  **Prepare Linux Image:**
    Place the `vmlinux` and `initrd` in `hikami/guest_image/` as described in the root `README.md`.

2.  **Build and Run:**
    Rebuild the hypervisor with `cargo make build-hikami` and flash it.
    The console will show the Linux boot process. Compare the timestamps with native boot (without the hypervisor).

### Experiment 4: Extension Emulation Code Generation (Section 3.3, 4.6)

This experiment verifies the **Ozora** auto-generation framework.

1.  **Generate Code for Zbs Extension:**
    ```bash
    cd ozora
    nix develop . --command cargo r riscv_insts_zbs.sail target/zbs.rs --ext-name Zbs
    ```

2.  **Verify Correctness:**
    Compare the generated code with the manually integrated version in the hypervisor:
    ```bash
    # Check emulation logic
    diff -u ../hikami_zbs/src/lib.rs target/zbs.rs
    ```
    *Note: Minor formatting differences may exist; use `rustfmt` on both files for a cleaner comparison.*

### Experiment 5: QEMU Evaluation (Partial Reproduction)

If physical hardware (Milk-V Megrez) is not available, you can still verify the emulation logic and the hypervisor's core mechanisms using QEMU.

1.  **Run Emulation Overhead Benchmark on QEMU:**
    ```bash
    cd evaluation/emulation_overhead
    nix develop .. --command cargo make qemu
    ```
    This will run the bare-metal evaluation binary on QEMU's `virt` machine.

2.  **Verify Tracing (Optional):**
    If you have a QEMU build with TCG plugin support, you can generate a trace log:
    ```bash
    nix develop .. --command cargo make qemu_log
    ```
    This generates `qemu.log`, which can be analyzed using `cargo make inspect`.

---

## 3. Claims Supported by the Artifact

| Claim | Evidence | Paper Reference |
| :--- | :--- | :--- |
| **Lightweight Implementation** | SLoC count (~7k lines total) and binary size (< 100 KiB) | Section 4.2, Table 5 |
| **Sub-microsecond Interrupt Latency** | Direct measurement on Megrez hardware (~1.0 $\mu$s) | Section 4.3.2 |
| **Near-native Performance** | CoreMark results showing 99.8% of native speed | Section 4.5, Table 7 |
| **Outperforms QEMU in Realistic Workloads** | Faster execution when emulated instructions account for $\le$ 0.1% | Section 4.5, Table 8 |
| **Reliable Code Generation** | Sail-to-Rust conversion using Ozora (81/109 lines auto-generated) | Section 4.6 |

---

## 4. Claims NOT Supported by the Artifact

- **Hardware-Specific Performance on Non-Megrez Platforms**: While the hypervisor is designed for RISC-V generally, the exact latency figures (0.5 $\mu$s - 1.0 $\mu$s) are specific to the EIC7700X SoC (Milk-V Megrez).
- **IOMMU/AIA Support on Physical Hardware**: As noted in Section 5 (Conclusion), validation of AIA and IOMMU support on physical hardware is identified as future work, as the current target SoC (Milk-V Megrez) does not natively support these features.

---

## 5. Directory Structure Reference

- `hikami/`: Hypervisor core (Rust).
- `hikami_zbs/`: Emulation module for Bitmanip extension.
- `evaluation/`: Bare-metal test cases for latency measurement.
- `ozora/`: Sail-driven code generation tool.
- `rockos-u-boot/` & `rockos-opensbi/`: Modified bootloader stack.
- `lctes26-paper45.pdf`: Submitted version of the paper.
