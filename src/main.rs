#![feature(test)]

#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#[macro_use]
#[macro_use(assert_diff)]

extern crate log;
extern crate colog;
extern crate regex;
extern crate difference;
extern crate time;
extern crate test;

use std::io::{self, stdout, BufRead, Write};
use regex::Regex;
use std::process::exit;

mod cpu;
mod tools;

fn main() {

    drop(colog::init());

    let mut cpu = cpu::CPU::new();

    let bios = tools::read_binary("../dos-software-decoding/ibm-pc/ibm5550/ipl5550.rom");
    cpu.load_bios(&bios);

    let stdin = io::stdin();

    loop {
        print!("{:04X}:{:04X}> ", cpu.sreg16[cpu::CS], cpu.ip);
        let _ = stdout().flush();

        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();

        let parts: Vec<String> = line.split(' ')
            .map(|s| s.trim_right().to_string())
            .collect();
        match parts[0].as_ref() {
            "load" => {
                if parts.len() < 2 {
                    error!("Filename not provided.");
                } else {
                    let data = tools::read_binary(parts[1].as_ref());
                    cpu.load_com(&data);
                }
            }
            "flat" => {
                let offset = cpu.get_offset();
                let rom_offset = offset - cpu.get_rom_base() + 0x100;
                info!("{:04X}:{:04X} is {:06X}.  rom offset is 0000:0100, or {:06X}",
                      cpu.sreg16[cpu::CS],
                      cpu.ip,
                      offset,
                      rom_offset);
            }
            "reset" => {
                info!("Resetting CPU");
                cpu.reset();
            }
            "r" | "reg" | "regs" => {
                cpu.print_registers();
            }
            "d" | "disasm" => {
                let op = cpu.disasm_instruction();
                info!("{:?}", op);
                info!("{}", op.pretty_string());
            }
            "v" => {
                info!("Executed {} instructions", cpu.instruction_count);
            }
            "e" => {
                let n = if parts.len() < 2 {
                    1
                } else {
                    parts[1].parse::<usize>().unwrap()
                };

                info!("Executing {} instructions", n);
                for _ in 0..n {
                    let op = cpu.disasm_instruction();
                    info!("{}", op.pretty_string());
                    cpu.execute_instruction();
                }
            }
            "bp" | "breakpoint" => {
                // breakpoints - all values are flat offsets
                // XXX: "bp remove 0x123"
                // XXX allow to enter bp in format "segment:offset"
                if parts.len() < 2 {
                    error!("breakpoint: not enough arguments");
                } else {
                    match parts[1].as_ref() {
                        "help" => {
                            info!("Available breakpoint commands:");
                            info!("  bp add 0x123     adds a breakpoint");
                            info!("  bp clear         clears all breakpoints");
                            info!("  bp list          list all breakpoints");
                        }
                        "add" | "set" => {
                            let bp = parse_number_string(&parts[2]);
                            cpu.add_breakpoint(bp);
                            info!("Breakpoint added: {:04X}", bp);
                        }
                        "clear" => {
                            cpu.clear_breakpoints();
                        }
                        "list" => {
                            let list = cpu.get_breakpoints(); // .sort();
                            // XXXX sort list

                            let strs: Vec<String> =
                                list.iter().map(|b| format!("{:04X}", b)).collect();
                            let formatted_list = strs.join(" ");
                            warn!("breakpoints: {}", formatted_list);
                        }
                        _ => error!("unknown breakpoint subcommand: {}", parts[1]),
                    }
                }
            }
            "run" => {
                let list = cpu.get_breakpoints();
                warn!("Executing until we hit a breakpoint");

                loop {
                    cpu.execute_instruction();
                    if cpu.fatal_error {
                        error!("Failed to execute instruction, breaking.");
                        break;
                    }
                    let offset = cpu.get_offset();

                    // break if we hit a breakpoint
                    let mut list_iter = list.iter();
                    if let Some(n) = list_iter.find(|&&x| x == offset) {
                        warn!("Breakpoint reached {:04X}", n);
                        break;
                    }
                }
            }
            "exit" | "quit" | "q" => {
                info!("Exiting ... {} instructions was executed",
                      cpu.instruction_count);
                exit(0);
            }
            "" => {}
            _ => {
                println!("Unknown command: {}", parts[0]);
            }
        }
    }
}

fn parse_number_string(s: &str) -> usize {
    // XXX return Option, none = failed to parse
    if s[0..2] == *"0x" {
        usize::from_str_radix(&s[2..], 16).unwrap()
    } else {
        // decimal
        s.parse::<usize>().unwrap()
    }
}
