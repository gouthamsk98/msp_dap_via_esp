# ARM Cortex-M SWD Debugger CLI

A command-line interface for debugging ARM Cortex-M processors via SWD (Serial Wire Debug) protocol over serial communication.

## Features

- Halt and resume target processor
- Read/write memory addresses
- Read CPU registers (R0-R15, SP, LR, PC, XPSR)
- Support for hexadecimal and decimal input formats
- Verbose logging support

## Installation

```bash
cargo build --release
```

## Usage

### Basic Commands

```bash
# Show help
./target/release/msp_dap_link_via_serial --help

# Halt the target processor
./target/release/msp_dap_link_via_serial halt

# Resume the target processor
./target/release/msp_dap_link_via_serial resume

# Read from memory address
./target/release/msp_dap_link_via_serial read 0x20000000

# Write to memory address
./target/release/msp_dap_link_via_serial write 0x20000000 0x12345678

# Read a specific register
./target/release/msp_dap_link_via_serial read-reg pc
./target/release/msp_dap_link_via_serial read-reg r0
./target/release/msp_dap_link_via_serial read-reg sp

# Read Program Counter
./target/release/msp_dap_link_via_serial read-pc

# Read all registers
./target/release/msp_dap_link_via_serial read-all
```

### Options

- `--port, -p`: Serial port path (default: `/dev/tty.usbmodem1234561`)
- `--baud, -b`: Baud rate (default: `115200`)
- `--verbose, -v`: Enable verbose output

### Examples

```bash
# Use different serial port
./target/release/msp_dap_link_via_serial -p /dev/ttyUSB0 halt

# Use different baud rate with verbose output
./target/release/msp_dap_link_via_serial -b 9600 -v read-all

# Read memory with custom port
./target/release/msp_dap_link_via_serial -p /dev/ttyACM0 read 0x08000000
```

### Register Names

The following register names are supported:

- `r0` - `r15`: General purpose registers
- `sp` or `r13`: Stack Pointer
- `lr` or `r14`: Link Register
- `pc` or `r15`: Program Counter
- `xpsr` or `psr`: Program Status Register

You can also use numeric indices (0-16).

### Number Formats

Both hexadecimal (with `0x` prefix) and decimal numbers are supported:

```bash
# Hexadecimal
./target/release/msp_dap_link_via_serial read 0x20000000
./target/release/msp_dap_link_via_serial write 0x20000000 0x12345678

# Decimal
./target/release/msp_dap_link_via_serial read 536870912
./target/release/msp_dap_link_via_serial write 536870912 305419896
```

## Error Handling

The CLI provides clear error messages for:

- Serial port connection failures
- Invalid register names
- Communication timeouts
- Protocol errors

Use the `--verbose` flag for detailed debugging information.

## Architecture

The project consists of several modules:

- `main.rs`: CLI interface and command handling
- `loader.rs`: Serial communication and SWD protocol implementation
- `protocol.rs`: Low-level protocol frame handling
- `serial.rs`: Serial port utilities

## Dependencies

- `clap`: Command-line argument parsing
- `serialport`: Serial communication
- `tracing`: Logging framework
- `crc`: CRC calculation for protocol integrity
