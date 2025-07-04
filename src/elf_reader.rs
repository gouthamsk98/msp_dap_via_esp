use std::fs::File;
use std::io::{ Read, Seek, SeekFrom };
use tracing::info;
use goblin::elf::Elf;
use crc::{ Crc, Algorithm::CRC_32_IEEE };
use std::collections::HashMap;
use (#[derive(Debug)] pub);
struct ByteMismatch {
    pub address: u32,
    pub expected: u8,
    pub actual: u8,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub success: bool,
    pub total_sections: usize,
    pub verified_sections: Vec<u32>,
    pub mismatched_sections: HashMap<u32, Vec<ByteMismatch>>,
    pub errors: Vec<String>,
}

impl VerificationResult {
    fn new() -> Self {
        VerificationResult {
            success: true,
            total_sections: 0,
            verified_sections: Vec::new(),
            mismatched_sections: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn print_report(&self) {
        println!("\n=== Flash Verification Report ===");
        println!("Total sections: {}", self.total_sections);
        println!("Verified sections: {}", self.verified_sections.len());
        println!("Failed sections: {}", self.mismatched_sections.len());

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  • {}", error);
            }
        }

        if !self.mismatched_sections.is_empty() {
            println!("\nMismatched Sections:");
            for (addr, mismatches) in &self.mismatched_sections {
                println!("  Section 0x{:08X}: {} mismatches", addr, mismatches.len());

                // Show first few mismatches
                for (i, mismatch) in mismatches.iter().take(5).enumerate() {
                    println!(
                        "    0x{:08X}: expected 0x{:02X}, got 0x{:02X}",
                        mismatch.address,
                        mismatch.expected,
                        mismatch.actual
                    );
                }

                if mismatches.len() > 5 {
                    println!("    ... and {} more", mismatches.len() - 5);
                }
            }
        }

        println!("\nOverall result: {}", if self.success { "PASS" } else { "FAIL" });
    }
}

#[derive(Debug, Clone)]
pub struct FlashSection {
    pub address: u32,
    pub size: u32,
    pub data: Vec<u8>,
}

pub struct ElfFlashVerifier {
    pub sections: Vec<FlashSection>,
    pub entry_point: u32,
}
impl ElfFlashVerifier {
    pub fn from_elf_file(elf_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(elf_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let elf = Elf::parse(&buffer)?;
        let mut sections = Vec::new();

        // Extract programmable sections (those that should be in flash)
        for section_header in &elf.section_headers {
            // Check if section should be programmed to flash
            if Self::is_programmable_section(&elf, section_header, &buffer) {
                let section_data = Self::extract_section_data(&buffer, section_header)?;

                sections.push(FlashSection {
                    address: section_header.sh_addr as u32,
                    size: section_header.sh_size as u32,
                    data: section_data,
                });
            }
        }

        // Sort sections by address
        sections.sort_by_key(|s| s.address);

        Ok(ElfFlashVerifier {
            sections,
            entry_point: elf.entry as u32,
        })
    }
    fn is_programmable_section(
        elf: &Elf,
        section_header: &goblin::elf::SectionHeader,
        buffer: &[u8]
    ) -> bool {
        use goblin::elf::section_header::*;

        // Section must be allocated and have content
        if (section_header.sh_flags & (SHF_ALLOC as u64)) == 0 {
            return false;
        }

        // Must have data (not BSS)
        if section_header.sh_type == SHT_NOBITS {
            return false;
        }

        // Must be in flash address range (0x00000000 - 0x0001FFFF for MSPM0G3507)
        let addr = section_header.sh_addr as u32;
        if addr >= 0x00000000 && addr < 0x00020000 {
            return true;
        }

        // Also check for info flash (0x41C00000 range)
        if addr >= 0x41c00000 && addr < 0x41c00400 {
            return true;
        }

        false
    }
    fn extract_section_data(
        buffer: &[u8],
        section_header: &goblin::elf::SectionHeader
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = section_header.sh_offset as usize;
        let size = section_header.sh_size as usize;

        if start + size > buffer.len() {
            return Err("Section data extends beyond file".into());
        }

        Ok(buffer[start..start + size].to_vec())
    }
    pub fn verify_flash<F>(
        &self,
        mut read_flash: F
    ) -> Result<VerificationResult, Box<dyn std::error::Error>>
        where F: FnMut(u32, u32) -> Result<Vec<u8>, String>
    {
        let mut result = VerificationResult::new();

        for section in &self.sections {
            println!(
                "Verifying section at 0x{:08X}, size: {} bytes",
                section.address,
                section.size
            );

            // Read flash data in chunks (your protocol has 4-byte max)
            let mut flash_data = Vec::new();
            let mut current_addr = section.address;
            let mut remaining = section.size;

            while remaining > 0 {
                let chunk_size = std::cmp::min(remaining, 4);
                match read_flash(current_addr, chunk_size) {
                    Ok(mut chunk) => {
                        // Pad chunk if necessary
                        chunk.resize(chunk_size as usize, 0xff);
                        flash_data.extend_from_slice(&chunk);
                    }
                    Err(e) => {
                        result.errors.push(
                            format!("Failed to read flash at 0x{:08X}: {}", current_addr, e)
                        );
                        return Ok(result);
                    }
                }
                current_addr += chunk_size;
                remaining -= chunk_size;
            }

            // Compare data
            if flash_data.len() != section.data.len() {
                result.errors.push(
                    format!(
                        "Size mismatch in section at 0x{:08X}: expected {} bytes, got {} bytes",
                        section.address,
                        section.data.len(),
                        flash_data.len()
                    )
                );
                continue;
            }

            // Byte-by-byte comparison
            let mut mismatches = Vec::new();
            for (i, (expected, actual)) in section.data.iter().zip(flash_data.iter()).enumerate() {
                if expected != actual {
                    mismatches.push(ByteMismatch {
                        address: section.address + (i as u32),
                        expected: *expected,
                        actual: *actual,
                    });
                }
            }

            if mismatches.is_empty() {
                result.verified_sections.push(section.address);
                println!("✓ Section at 0x{:08X} verified successfully", section.address);
            } else {
                result.mismatched_sections.insert(section.address, mismatches);
                // println!(
                //     "✗ Section at 0x{:08X} has {} mismatches",
                //     section.address,
                //     mismatches.len()
                // );
            }
        }

        // Calculate overall statistics
        result.total_sections = self.sections.len();
        result.success = result.errors.is_empty() && result.mismatched_sections.is_empty();

        Ok(result)
    }
    pub fn get_memory_map(&self) -> Vec<(u32, u32)> {
        self.sections
            .iter()
            .map(|s| (s.address, s.address + s.size - 1))
            .collect()
    }

    pub fn calculate_checksum(&self) -> u32 {
        let crc = Crc::<u32>::new(&CRC_32_IEEE);
        let mut digest = crc.digest();

        for section in &self.sections {
            digest.update(&section.address.to_le_bytes());
            digest.update(&section.data);
        }

        digest.finalize()
    }
}
