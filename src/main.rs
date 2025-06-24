mod serial;
mod loader;
mod protocol;
use std::thread;
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;
use clap::{ Parser, Subcommand };
use tracing::{ info, error as einfo };

// ARM Cortex-M Register Indices
pub mod registers {
    pub const R0: u32 = 0x00;
    pub const R1: u32 = 0x01;
    pub const R2: u32 = 0x02;
    pub const R3: u32 = 0x03;
    pub const R4: u32 = 0x04;
    pub const R5: u32 = 0x05;
    pub const R6: u32 = 0x06;
    pub const R7: u32 = 0x07;
    pub const R8: u32 = 0x08;
    pub const R9: u32 = 0x09;
    pub const R10: u32 = 0x0a;
    pub const R11: u32 = 0x0b;
    pub const R12: u32 = 0x0c;
    pub const SP: u32 = 0x0d; // Stack Pointer (R13)
    pub const LR: u32 = 0x0e; // Link Register (R14)
    pub const PC: u32 = 0x0f; // Program Counter (R15)
    pub const XPSR: u32 = 0x10; // Program Status Register
}

#[derive(Parser)]
#[command(name = "swd-debugger")]
#[command(about = "ARM Cortex-M SWD Debugger CLI")]
#[command(version = "1.0")]
struct Cli {
    /// Serial port path (e.g., /dev/tty.usbmodem1234561)
    #[arg(short, long, default_value = "/dev/tty.usbmodem1234561")]
    port: String,

    /// Baud rate for serial communication
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Halt the target processor
    Halt,
    /// Resume the target processor
    Resume,
    /// Read from a memory address
    Read {
        /// Memory address to read from (hex format, e.g., 0x20000000)
        #[arg(value_parser = parse_hex)]
        address: u32,
    },
    /// Write to a memory address
    Write {
        /// Memory address to write to (hex format, e.g., 0x20000000)
        #[arg(value_parser = parse_hex)]
        address: u32,
        /// Value to write (hex format, e.g., 0x12345678)
        #[arg(value_parser = parse_hex)]
        value: u32,
    },
    /// Read a CPU register
    ReadReg {
        /// Register name (r0, r1, ..., r15, sp, lr, pc, xpsr) or index (0-16)
        register: String,
    },
    /// Read the Program Counter (PC) register
    ReadPc,
    /// Read all CPU registers
    ReadAll,
}

fn parse_hex(s: &str) -> Result<u32, std::num::ParseIntError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16)
    } else {
        s.parse::<u32>()
    }
}

fn parse_register_name(reg_name: &str) -> Result<u32, String> {
    match reg_name.to_lowercase().as_str() {
        "r0" => Ok(registers::R0),
        "r1" => Ok(registers::R1),
        "r2" => Ok(registers::R2),
        "r3" => Ok(registers::R3),
        "r4" => Ok(registers::R4),
        "r5" => Ok(registers::R5),
        "r6" => Ok(registers::R6),
        "r7" => Ok(registers::R7),
        "r8" => Ok(registers::R8),
        "r9" => Ok(registers::R9),
        "r10" => Ok(registers::R10),
        "r11" => Ok(registers::R11),
        "r12" => Ok(registers::R12),
        "r13" | "sp" => Ok(registers::SP),
        "r14" | "lr" => Ok(registers::LR),
        "r15" | "pc" => Ok(registers::PC),
        "xpsr" | "psr" => Ok(registers::XPSR),
        _ => {
            // Try parsing as a number
            if let Ok(index) = reg_name.parse::<u32>() {
                if index <= 16 {
                    Ok(index)
                } else {
                    Err(format!("Register index {} out of range (0-16)", index))
                }
            } else {
                Err(format!("Unknown register: {}", reg_name))
            }
        }
    }
}

fn get_register_name(reg_index: u32) -> &'static str {
    match reg_index {
        0x00 => "R0",
        0x01 => "R1",
        0x02 => "R2",
        0x03 => "R3",
        0x04 => "R4",
        0x05 => "R5",
        0x06 => "R6",
        0x07 => "R7",
        0x08 => "R8",
        0x09 => "R9",
        0x0a => "R10",
        0x0b => "R11",
        0x0c => "R12",
        0x0d => "SP",
        0x0e => "LR",
        0x0f => "PC",
        0x10 => "XPSR",
        _ => "UNKNOWN",
    }
}

fn main() {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let subscriber = if cli.verbose {
        FmtSubscriber::builder().with_max_level(tracing::Level::DEBUG).finish()
    } else {
        FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish()
    };

    tracing::subscriber
        ::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    // Create the serial loader
    let mut debug = match loader::SerialLoader::new(&cli.port, cli.baud) {
        Ok(loader) => {
            info!("Connected to {} at {} baud", cli.port, cli.baud);
            loader
        }
        Err(e) => {
            einfo!("Failed to connect to {}: {}", cli.port, e);
            std::process::exit(1);
        }
    };

    // Execute the command
    let result = match cli.command {
        Commands::Halt => {
            info!("Halting target processor...");
            debug.halt()
        }
        Commands::Resume => {
            info!("Resuming target processor...");
            debug.resume()
        }
        Commands::Read { address } => {
            info!("Reading from address 0x{:08X}...", address);
            match debug.read_word(address) {
                Ok(value) => {
                    info!("0x{:08X}: 0x{:08X}", address, value);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Commands::Write { address, value } => {
            info!("Writing 0x{:08X} to address 0x{:08X}...", value, address);
            debug.write_word(address, value)
        }
        Commands::ReadReg { register } => {
            match parse_register_name(&register) {
                Ok(reg_index) => {
                    info!("Reading register {}...", get_register_name(reg_index));
                    match debug.read_register(reg_index) {
                        Ok(value) => {
                            info!("{}: 0x{:08X}", get_register_name(reg_index), value);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => {
                    einfo!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ReadPc => {
            info!("Reading Program Counter...");
            match debug.read_pc_register() {
                Ok(value) => {
                    info!("PC: 0x{:08X}", value);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Commands::ReadAll => {
            info!("Reading all CPU registers...");
            let mut errors = Vec::new();

            // Read all registers R0-R15 and XPSR
            for reg_index in 0..=16 {
                match debug.read_register(reg_index) {
                    Ok(value) => {
                        info!("{}: 0x{:08X}", get_register_name(reg_index), value);
                    }
                    Err(e) => {
                        errors.push(
                            format!("Failed to read {}: {}", get_register_name(reg_index), e)
                        );
                    }
                }
                // Small delay between reads
                thread::sleep(Duration::from_millis(10));
            }

            if !errors.is_empty() {
                for error in errors {
                    einfo!("Error: {}", error);
                }
                Err("Some register reads failed".into())
            } else {
                Ok(())
            }
        }
    };

    match result {
        Ok(_) => {
            if cli.verbose {
                info!("Command completed successfully");
            }
        }
        Err(e) => {
            einfo!("Command failed: {}", e);
            std::process::exit(1);
        }
    }
}
