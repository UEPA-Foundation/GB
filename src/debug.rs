use crate::gameboy::GameBoy;
use std::io::Write;

#[derive(Clone)]
pub enum Arg {
    Numeric(u16),
    Bool(bool),
    Str(String),
    None,
}

struct WatchPoint {
    // TODO: this does not keep track of what ROM/RAM bank the address belongs to
    addr: u16,
    last_val: u8,
}

pub struct Debugger {
    breakpoints: Vec<u16>,
    watchpoints: Vec<WatchPoint>,
    last_cmd: String,
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
    config: DbgConfig,
}

struct DbgConfig {
    disasm: bool,
    regs: bool,
}

impl Debugger {
    pub fn init() -> Self {
        Self {
            last_cmd: "help".to_string(),
            breakpoints: vec![],
            watchpoints: vec![],
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
            config: DbgConfig { disasm: true, regs: true },
        }
    }

    pub fn prompt(&mut self, gb: &mut GameBoy) {
        print!("> ");
        if !self.stdout.flush().is_ok() {
            println!();
            return;
        }

        let mut user_input = String::new();
        if !self.stdin.read_line(&mut user_input).is_ok() {
            println!();
            return;
        }

        let mut stripped_input = match user_input.strip_suffix("\n") {
            None => user_input.as_str(),
            Some(stripped) => stripped,
        };

        if stripped_input.is_empty() {
            stripped_input = self.last_cmd.as_str();
        } else {
            self.last_cmd = stripped_input.to_string();
        }

        let splitted_input: Vec<&str> = stripped_input.split_whitespace().collect();

        if splitted_input.len() > 3 {
            println!("Invalid number of arguments. They range 0-2.");
            return;
        }

        let cmd_end = stripped_input.find(' ').unwrap_or(stripped_input.len());
        let slash_pos = std::cmp::min(cmd_end, stripped_input.find('/').unwrap_or(stripped_input.len()));

        let cmd_name = &stripped_input[0..slash_pos];
        let mod_str = match stripped_input[slash_pos..cmd_end].to_string().strip_prefix("/") {
            None => stripped_input[slash_pos..cmd_end].to_string(),
            Some(stripped) => stripped.to_string(),
        };

        let mut modif = None;
        if cmd_end != slash_pos {
            match eval_modif(mod_str) {
                Ok(m) => modif = m,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }

        let mut arg1 = Arg::None;
        let mut arg2 = Arg::None;
        if splitted_input.len() > 1 {
            arg1 = match self.eval_arg(gb, splitted_input[1]) {
                Ok(arg) => arg,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }
        if splitted_input.len() > 2 {
            arg2 = match self.eval_arg(gb, splitted_input[2]) {
                Ok(arg) => arg,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }

        match (cmd_name, modif, arg1, arg2) {
            ("c" | "continue", None, Arg::None, Arg::None) => self.continue_cmd(gb),
            ("s" | "step", _, Arg::None, Arg::None) => self.step_cmd(gb, modif),
            ("n" | "next", _, Arg::None, Arg::None) => self.next_cmd(gb, modif),
            ("h" | "help", None, Arg::None, Arg::None) => self.help_cmd("".to_string()),
            ("h" | "help", None, Arg::Str(cmd_name), Arg::None) => self.help_cmd(cmd_name),
            ("b" | "break", None, Arg::Numeric(addr), Arg::None) => self.breakpoint_cmd(addr),
            ("de" | "delete", None, Arg::Numeric(addr), Arg::None) => self.delete_cmd(addr),
            ("w" | "watch", None, Arg::Numeric(addr), Arg::None) => self.watchpoint_cmd(gb, addr),
            ("dw" | "delwatch", None, Arg::Numeric(addr), Arg::None) => self.delwatch_cmd(addr),
            ("l" | "list", None, Arg::None, Arg::None) => self.list_cmd(),
            ("d" | "disassemble", _, Arg::None, Arg::None) => self.disasm_cmd(gb, modif, gb.cpu.pc),
            ("d" | "disassemble", _, Arg::Numeric(addr), Arg::None) => self.disasm_cmd(gb, modif, addr),
            ("x" | "examine", _, Arg::Numeric(addr), Arg::None) => self.examine_cmd(gb, modif, addr),
            ("r" | "regs" | "registers", None, Arg::None, Arg::None) => self.regs_cmd(gb),
            ("set", _, Arg::Str(config), Arg::Bool(state)) => self.set_cmd(config, state),
            ("cl" | "clear", None, Arg::None, Arg::None) => self.clear_cmd(),
            _ => self.help_cmd(cmd_name.to_string()),
        };

        if self.config.regs {
            self.regs_cmd(gb);
        }

        if self.config.disasm {
            self.disasm_cmd(gb, None, gb.cpu.pc);
        }
    }

    fn continue_cmd(&mut self, gb: &mut GameBoy) {
        for i in 0..self.watchpoints.len() {
            self.watchpoints[i].last_val = gb.read(self.watchpoints[i].addr);
        }

        loop {
            gb.cpu_step();

            let mut changes = vec![];
            for i in 0..self.watchpoints.len() {
                let val = gb.read(self.watchpoints[i].addr);
                if val != self.watchpoints[i].last_val {
                    changes.push((i + 1, val, self.watchpoints[i].addr))
                }
            }
            if !changes.is_empty() {
                for i in 0..changes.len() {
                    println!(
                        "Watchpoint {}: value ${:02X} written to address ${:04X}",
                        changes[i].0, changes[i].1, changes[i].2
                    );
                }
                break;
            }

            if self.breakpoints.binary_search(&gb.cpu.pc).is_ok() {
                break;
            }
        }
    }

    fn step_cmd(&mut self, gb: &mut GameBoy, modif: Option<u16>) {
        let steps = match modif {
            None => 1,
            Some(n) => n,
        };
        for _ in 0..steps {
            gb.cpu_step();
        }
    }

    fn next_cmd(&mut self, gb: &mut GameBoy, modif: Option<u16>) {
        let steps = match modif {
            None => 1,
            Some(n) => n,
        };
        for _ in 0..steps {
            let mut count = 0;
            loop {
                match gb.dpc(0) {
                    0xC4 | 0xCC | 0xD4 | 0xDC | 0xCD => count += 1,
                    0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 | 0xD9 => count -= 1,
                    _ => {}
                }
                gb.cpu_step();
                if count == 0 {
                    break;
                }
            }
        }
    }

    fn examine_cmd(&mut self, gb: &mut GameBoy, modif: Option<u16>, addr: u16) {
        let count = match modif {
            None => 32,
            Some(n) => n,
        };
        let mut s = String::new();
        for i in 0..count {
            if i % 16 == 0 {
                s += &format!("${:04X}: ", u16::wrapping_add(addr, i));
            }
            s += &format!("{:02X} ", gb.read(u16::wrapping_add(addr, i)));
            if i % 16 == 15 {
                s += "\n";
            }
        }
        println!("{}", s);
    }

    fn regs_cmd(&mut self, gb: &mut GameBoy) {
        println!("{}", gb.cpu);
        println!();
    }

    fn breakpoint_cmd(&mut self, addr: u16) {
        match self.breakpoints.binary_search(&addr) {
            Ok(_) => {
                println!("Breakpoint already at ${:04X}", addr);
            }
            Err(pos) => {
                self.breakpoints.insert(pos, addr);
                println!("Breakpoint set at ${:04X}", addr);
            }
        }
    }

    fn delete_cmd(&mut self, addr: u16) {
        match self.breakpoints.binary_search(&addr) {
            Ok(pos) => {
                _ = self.breakpoints.remove(pos);
                println!("Deleted breakpoint at ${:04X}", addr);
            }
            Err(_) => {
                println!("No breakpoint at ${:04X}", addr);
            }
        }
    }

    fn watchpoint_cmd(&mut self, gb: &mut GameBoy, addr: u16) {
        match self.watchpoints.binary_search_by(|wp| wp.addr.cmp(&addr)) {
            Ok(_) => {
                println!("Watchpoint already at ${:04X}", addr);
            }
            Err(pos) => {
                self.watchpoints.insert(pos, WatchPoint { addr, last_val: gb.read(addr) });
                println!("Watchpoint set at ${:04X}", addr);
            }
        }
    }

    fn delwatch_cmd(&mut self, addr: u16) {
        match self.watchpoints.binary_search_by(|wp| wp.addr.cmp(&addr)) {
            Ok(pos) => {
                _ = self.watchpoints.remove(pos);
                println!("Deleted watchpoint at ${:04X}", addr);
            }
            Err(_) => {
                println!("No breakpoint at ${:04X}", addr);
            }
        }
    }

    fn list_cmd(&mut self) {
        println!("BREAKPOINTS");
        for i in 0..self.breakpoints.len() {
            println!("Breakpoint {}: ${:X}", i + 1, self.breakpoints[i]);
        }
        println!();

        println!("WATCHPOINTS");
        for i in 0..self.watchpoints.len() {
            println!("Watchpoint {}: ${:X}", i + 1, self.watchpoints[i].addr);
        }
        println!();
    }

    fn set_cmd(&mut self, config: String, state: bool) {
        match config.as_str() {
            "disasm" => self.config.disasm = state,
            "regs" => self.config.regs = state,
            _ => println!("Invalid configuration flag: {}", config),
        }
    }

    fn disasm_cmd(&mut self, gb: &mut GameBoy, modif: Option<u16>, mut addr: u16) {
        let count = match modif {
            None => 5,
            Some(n) => n,
        };

        for _ in 1..=count {
            let (dis, len) = self.disassemble(gb, addr);
            let mut padding = "    ";
            if addr == gb.cpu.pc {
                padding = " -> ";
            }
            println!("{}{:04X}: {}", padding, addr, dis);
            addr = u16::wrapping_add(addr, len as u16);
        }
    }

    fn help_cmd(&mut self, cmd_name: String) {
        match cmd_name.as_str() {
            "" => {
                println!("List of commands:\n");
                println!("{}h{}elp -- displays this message", ULINE, RESET);
                println!("{}c{}ontinue -- continues execution without stopping", ULINE, RESET);
                println!("{}s{}tep -- executes the next instruction, stepping into function calls", ULINE, RESET);
                println!("{}n{}ext -- executes the next instruction, stepping over function calls", ULINE, RESET);
                println!("e{}x{}amine -- displays a range of values from memory", ULINE, RESET);
                println!("{}r{}egisters -- displays value of cpu registers", ULINE, RESET);
                println!("{}d{}isassemble -- disassembles instructions at PC or at a specified address", ULINE, RESET);
                println!("{}b{}reak -- creates a breakpoint at a specified address", ULINE, RESET);
                println!("{}de{}lete -- deletes a breakpoint at a specified address", ULINE, RESET);
                println!("{}w{}atch -- creates a watchpoint at a specified address", ULINE, RESET);
                println!("{}d{}el{}w{}atch -- deletes a watchpoint at a specified address", ULINE, RESET, ULINE, RESET);
                println!("{}l{}ist -- lists live breakpoints and watchpoints", ULINE, RESET);
                println!("{}s{}et -- sets a configuration flag", ULINE, RESET);
                println!("{}cl{}ear -- clears terminal", ULINE, RESET);
                println!();
            }
            "h" | "help" => {
                println!("{}h{}elp -- displays list of commands or explanation of a command", ULINE, RESET);
                println!("usage: help [command]");
                println!();
            }
            "c" | "continue" => {
                println!("{}c{}ontinue -- continues execution without stopping", ULINE, RESET);
                println!("usage: continue");
                println!();
            }
            "s" | "step" => {
                println!("{}s{}tep[/count] -- executes [count] next instruction(s)", ULINE, RESET);
                println!("usage: step");
                println!();
            }
            "n" | "next" => {
                println!("{}s{}tep[/count] -- executes [count] next instruction(s)", ULINE, RESET);
                println!("usage: step");
                println!();
            }
            "x" | "examine" => {
                println!("e{}x{}amine -- displays a range of values from memory", ULINE, RESET);
                println!("usage: examine[/count] address");
                println!();
            }
            "r" | "regs" | "registers" => {
                println!("{}r{}egisters -- displays value of cpu registers", ULINE, RESET);
                println!("usage: registers");
                println!();
            }
            "d" | "disassemble" => {
                println!("{}d{}isassemble -- disassembles instructions at a specified address", ULINE, RESET);
                println!("usage: disassemble[/count] address");
                println!();
            }
            "b" | "break" => {
                println!("{}b{}reak -- creates a breakpoint at a specified address", ULINE, RESET);
                println!("             program execution will stop when arriving at a breakpoint");
                println!("usage: break address");
                println!();
            }
            "de" | "delete" => {
                println!("{}de{}lete -- deletes a breakpoint at a specified address", ULINE, RESET);
                println!("usage: delete address");
                println!();
            }
            "w" | "watch" => {
                println!("{}w{}atch -- creates a watchpoint at a specified address", ULINE, RESET);
                println!("             program execution will stop when value at a watchpoint changes");
                println!("usage: watch address");
                println!();
            }
            "dw" | "delwatch" => {
                println!("{}de{}l{}w{}atch -- deletes a watchpoint at a specified address", ULINE, RESET, ULINE, RESET);
                println!("usage: delwatch address");
                println!();
            }
            "l" | "list" => {
                println!("{}l{}ist -- lists live breakpoints and watchpoints", ULINE, RESET);
                println!("usage: list");
                println!();
            }
            "set" => {
                println!("{}set{} -- sets a configuration flag", ULINE, RESET);
                println!("usage: set config on/off\n");
                println!("Configuration flags:");
                println!("disasm -- automatic printing of disassembly at current address");
                println!("regs -- automatic printing of cpu regs");
                println!();
            }
            "cl" | "clear" => {
                println!("{}cl{}ear -- clears terminal", ULINE, RESET);
                println!("usage: clear\n");
                println!();
            }
            _ => {
                println!("{}", format!("Invalid command: {}", cmd_name));
                println!();
            }
        }
    }

    fn clear_cmd(&self) {
        print!("\x1b[2J\x1b[1;1H");
    }

    fn eval_arg(&self, gb: &mut GameBoy, arg_str: &str) -> Result<Arg, String> {
        match arg_str {
            "on" => {
                return Ok(Arg::Bool(true));
            }
            "off" => {
                return Ok(Arg::Bool(false));
            }
            "help" | "continue" | "step" | "disassemble" | "break" | "delete" | "watch" | "delwatch" | "list"
            | "examine" | "registers" | "set" | "clear" => {
                return Ok(Arg::Str(arg_str.to_string()));
            }
            "disasm" | "regs" => {
                return Ok(Arg::Str(arg_str.to_string()));
            }
            "af" => return Ok(Arg::Numeric(gb.cpu.rd_af())),
            "bc" => return Ok(Arg::Numeric(gb.cpu.rd_bc())),
            "de" => return Ok(Arg::Numeric(gb.cpu.rd_de())),
            "hl" => return Ok(Arg::Numeric(gb.cpu.rd_hl())),
            "pc" => return Ok(Arg::Numeric(gb.cpu.pc)),
            "sp" => return Ok(Arg::Numeric(gb.cpu.sp)),
            _ => {}
        };

        let arg;
        if arg_str.starts_with('$') {
            arg = match u16::from_str_radix(&arg_str[1..], 16) {
                Ok(n) => Ok(Arg::Numeric(n)),
                Err(_e) => Err(format!("Invalid argument: {}", arg_str)),
            }
        } else {
            arg = match arg_str.parse::<u16>() {
                Ok(n) => Ok(Arg::Numeric(n)),
                Err(_e) => Err(format!("Invalid argument: {}", arg_str)),
            }
        };

        arg
    }

    pub fn disassemble(&self, gb: &mut GameBoy, addr: u16) -> (String, u8) {
        let opcode = gb.read(addr);
        let mut mnemonic = OPCODES_STR[opcode as usize].to_string();

        if mnemonic == "CB" {
            let param = gb.read(u16::wrapping_add(addr, 1));
            return (OPCODES_CB_STR[param as usize].to_string(), 2);
        }

        if mnemonic.contains("U8") {
            let param = gb.read(u16::wrapping_add(addr, 1));
            mnemonic = mnemonic.replace("U8", &format!("${:02X}", param));
            return (mnemonic, 2);
        }

        if mnemonic.contains("U16") {
            let param1 = gb.read(u16::wrapping_add(addr, 1));
            let param2 = gb.read(u16::wrapping_add(addr, 2));
            mnemonic = mnemonic.replace("U16", &format!("${:04X}", (((param2 as u16) << 8) + param1 as u16)));
            return (mnemonic, 3);
        }

        if mnemonic.contains("I8") {
            let param = gb.read(u16::wrapping_add(addr, 1));
            mnemonic = mnemonic.replace("I8", &format!("${:02X}", param));
            mnemonic += " (";
            if param as i8 > 0 {
                mnemonic += "+";
            }
            mnemonic += &format!("{})", param as i8);
            return (mnemonic, 2);
        }

        (mnemonic, 1)
    }
}

fn eval_modif(mod_str: String) -> Result<Option<u16>, String> {
    if mod_str == "" {
        return Ok(None);
    }
    if mod_str.starts_with('$') {
        Ok(Some(u16::from_str_radix(&mod_str[1..], 16).or_else(|_| Err(format!("Invalid modifier: {}", mod_str)))?))
    } else {
        Ok(Some(mod_str.parse::<u16>().or_else(|_| Err(format!("Invalid modifier: {}", mod_str)))?))
    }
}

#[rustfmt::skip]
pub const OPCODES_STR: [&str; 256] = [
/*             X0              X1              X2              X3              X4              X5              X6              X7              */
/*             X8              X9              XA              XB              XC              XD              XE              XF              */
/* 0X */      "NOP",           "LD BC, U16",   "LD (BC), A",   "INC BC",       "INC B",        "DEC B",        "LD B, U8",     "RLCA",
              "LD (U16), SP",  "ADD HL, BC",   "LD A, (BC)",   "DEC BC",       "INC C",        "DEC C",        "LD C, U8",     "RRCA",
/* 1X */      "STOP",          "LD DE, U16",   "LD (DE), A",   "INC DE",       "INC D",        "DEC D",        "LD D, U8",     "RLA",
              "JR I8",         "ADD HL, DE",   "LD A, (DE)",   "DEC DE",       "INC E",        "DEC E",        "LD E, U8",     "RRA",
/* 2X */      "JR NZ, I8",     "LD HL, U16",   "LD (HLI), A",  "INC HL",       "INC H",        "DEC H",        "LD H, U8",     "DAA",
              "JR Z, I8",      "ADD HL, HL",   "LD A, (HLI)",  "DEC HL",       "INC L",        "DEC L",        "LD L, U8",     "CPL",
/* 3X */      "JR NC, I8",     "LD SP, U16",   "LD (HLD), A",  "INC SP",       "INC (HL)",     "DEC (HL)",     "LD (HL), U8",  "SCF",
              "JR C, I8",      "ADD HL, SP",   "LD A, (HLD)",  "DEC SP",       "INC A",        "DEC A",        "LD A, U8",     "CCF",
/* 4X */      "LD B, B",       "LD B, C",      "LD B, D",      "LD B, E",      "LD B, H",      "LD B, L",      "LD B, (HL)",   "LD B, A",
              "LD C, B",       "LD C, C",      "LD C, D",      "LD C, E",      "LD C, H",      "LD C, L",      "LD C, (HL)",   "LD C, A",
/* 5X */      "LD D, B",       "LD D, C",      "LD D, D",      "LD D, E",      "LD D, H",      "LD D, L",      "LD D, (HL)",   "LD D, A",
              "LD E, B",       "LD E, C",      "LD E, D",      "LD E, E",      "LD E, H",      "LD E, L",      "LD E, (HL)",   "LD E, A",
/* 6X */      "LD H, B",       "LD H, C",      "LD H, D",      "LD H, E",      "LD H, H",      "LD H, L",      "LD H, (HL)",   "LD H, A",
              "LD L, B",       "LD L, C",      "LD L, D",      "LD L, E",      "LD L, H",      "LD L, L",      "LD L, (HL)",   "LD L, A",
/* 7X */      "LD (HL), B",    "LD (HL), C",   "LD (HL), D",   "LD (HL), E",   "LD (HL), H",   "LD (HL), L",   "HALT",         "LD (HL), A",
              "LD A, B",       "LD A, C",      "LD A, D",      "LD A, E",      "LD A, H",      "LD A, L",      "LD A, (HL)",   "LD A, A",
/* 8X */      "ADD A, B",      "ADD A, C",     "ADD A, D",     "ADD A, E",     "ADD A, H",     "ADD A, L",     "ADD A, (HL)",  "ADD A, A",
              "ADC A, B",      "ADC A, C",     "ADC A, D",     "ADC A, E",     "ADC A, H",     "ADC A, L",     "ADC A, (HL)",  "ADC A, A",
/* 9X */      "SUB B",         "SUB C",        "SUB D",        "SUB E",        "SUB H",        "SUB L",        "SUB (HL)",     "SUB A",
              "SBC A, B",      "SBC A, C",     "SBC A, D",     "SBC A, E",     "SBC A, H",     "SBC A, L",     "SBC A, (HL)",  "SBC A, A",
/* AX */      "AND B",         "AND C",        "AND D",        "AND E",        "AND H",        "AND L",        "AND (HL)",     "AND A",
              "XOR B",         "XOR C",        "XOR D",        "XOR E",        "XOR H",        "XOR L",        "XOR (HL)",     "XOR A",
/* BX */      "OR B",          "OR C",         "OR D",         "OR E",         "OR H",         "OR L",         "OR (HL)",      "OR A",
              "CP B",          "CP C",         "CP D",         "CP E",         "CP H",         "CP L",         "CP (HL)",      "CP A",
/* CX */      "RET NZ",        "POP BC",       "JP NZ, U16",   "JP U16",       "CALL NZ, U16", "PUSH BC",      "ADD A, U8",    "RST $00",
              "RET Z",         "RET",          "JP Z, U16",    "CB",           "CALL Z, U16",  "CALL U16",     "ADC A, U8",    "RST $08",
/* DX */      "RET NC",        "POP DE",       "JP NC, U16",   "UNDEFINED",    "CALL NC, U16", "PUSH DE",      "SUB U8",       "RST $10",
              "RET C",         "RETI",         "JP C, U16",    "UNDEFINED",    "CALL C, U16",  "UNDEFINED",    "SBC A, U8",    "RST $18",
/* EX */      "LDH (U8), A",   "POP HL",       "LDH (C), A",   "UNDEFINED",    "UNDEFINED",    "PUSH HL",      "AND U8",       "RST $20",
              "ADD SP, I8",    "JP (HL)",      "LD (U16), A",  "UNDEFINED",    "UNDEFINED",    "UNDEFINED",    "XOR U8",       "RST $28",
/* FX */      "LDH A, (U8)",   "POP AF",       "LDH A, (C)",   "DI",           "UNDEFINED",    "PUSH AF",      "OR U8",        "RST $30",
              "LD HL, SP+I8",  "LD SP, HL",    "LD A, (U16)",  "EI",           "UNDEFINED",    "UNDEFINED",    "CP U8",        "RST $38",
];

#[rustfmt::skip]
pub const OPCODES_CB_STR: [&str; 256] = [
/*            X0             X1             X2             X3             X4             X5             X6             X7             */
/*            X8             X9             XA             XB             XC             XD             XE             XF             */
/* 0X */      "RLC B",       "RLC C",       "RLC D",       "RLC E",       "RLC H",       "RLC L",       "RLC (HL)",    "RLC A",
              "RRC B",       "RRC C",       "RRC D",       "RRC E",       "RRC H",       "RRC L",       "RRC (HL)",    "RRC A",
/* 1X */      "RL B",        "RL C",        "RL D",        "RL E",        "RL H",        "RL L",        "RL (HL)",     "RL A",
              "RR B",        "RR C",        "RR D",        "RR E",        "RR H",        "RR L",        "RR (HL)",     "RR A",
/* 2X */      "SLA B",       "SLA C",       "SLA D",       "SLA E",       "SLA H",       "SLA L",       "SLA (HL)",    "SLA A",
              "SRA B",       "SRA C",       "SRA D",       "SRA E",       "SRA H",       "SRA L",       "SRA (HL)",    "SRA A",
/* 3X */      "SWAP B",      "SWAP C",      "SWAP D",      "SWAP E",      "SWAP H",      "SWAP L",      "SWAP (HL)",   "SWAP A",
              "SRL B",       "SRL C",       "SRL D",       "SRL E",       "SRL H",       "SRL L",       "SRL (HL)",    "SRL A",
/* 4X */      "BIT 0, B",    "BIT 0, C",    "BIT 0, D",    "BIT 0, E",    "BIT 0, H",    "BIT 0, L",    "BIT 0, (HL)", "BIT 0, A",
              "BIT 1, B",    "BIT 1, C",    "BIT 1, D",    "BIT 1, E",    "BIT 1, H",    "BIT 1, L",    "BIT 1, (HL)", "BIT 1, A",
/* 5X */      "BIT 2, B",    "BIT 2, C",    "BIT 2, D",    "BIT 2, E",    "BIT 2, H",    "BIT 2, L",    "BIT 2, (HL)", "BIT 2, A",
              "BIT 3, B",    "BIT 3, C",    "BIT 3, D",    "BIT 3, E",    "BIT 3, H",    "BIT 3, L",    "BIT 3, (HL)", "BIT 3, A",
/* 6X */      "BIT 4, B",    "BIT 4, C",    "BIT 4, D",    "BIT 4, E",    "BIT 4, H",    "BIT 4, L",    "BIT 4, (HL)", "BIT 4, A",
              "BIT 5, B",    "BIT 5, C",    "BIT 5, D",    "BIT 5, E",    "BIT 5, H",    "BIT 5, L",    "BIT 5, (HL)", "BIT 5, A",
/* 7X */      "BIT 6, B",    "BIT 6, C",    "BIT 6, D",    "BIT 6, E",    "BIT 6, H",    "BIT 6, L",    "BIT 6, (HL)", "BIT 6, A",
              "BIT 7, B",    "BIT 7, C",    "BIT 7, D",    "BIT 7, E",    "BIT 7, H",    "BIT 7, L",    "BIT 7, (HL)", "BIT 7, A",
/* 8X */      "RES 0, B",    "RES 0, C",    "RES 0, D",    "RES 0, E",    "RES 0, H",    "RES 0, L",    "RES 0, (HL)", "RES 0, A",
              "RES 1, B",    "RES 1, C",    "RES 1, D",    "RES 1, E",    "RES 1, H",    "RES 1, L",    "RES 1, (HL)", "RES 1, A",
/* 9X */      "RES 2, B",    "RES 2, C",    "RES 2, D",    "RES 2, E",    "RES 2, H",    "RES 2, L",    "RES 2, (HL)", "RES 2, A",
              "RES 3, B",    "RES 3, C",    "RES 3, D",    "RES 3, E",    "RES 3, H",    "RES 3, L",    "RES 3, (HL)", "RES 3, A",
/* AX */      "RES 4, B",    "RES 4, C",    "RES 4, D",    "RES 4, E",    "RES 4, H",    "RES 4, L",    "RES 4, (HL)", "RES 4, A",
              "RES 5, B",    "RES 5, C",    "RES 5, D",    "RES 5, E",    "RES 5, H",    "RES 5, L",    "RES 5, (HL)", "RES 5, A",
/* BX */      "RES 6, B",    "RES 6, C",    "RES 6, D",    "RES 6, E",    "RES 6, H",    "RES 6, L",    "RES 6, (HL)", "RES 6, A",
              "RES 7, B",    "RES 7, C",    "RES 7, D",    "RES 7, E",    "RES 7, H",    "RES 7, L",    "RES 7, (HL)", "RES 7, A",
/* CX */      "SET 0, B",    "SET 0, C",    "SET 0, D",    "SET 0, E",    "SET 0, H",    "SET 0, L",    "SET 0, (HL)", "SET 0, A",
              "SET 1, B",    "SET 1, C",    "SET 1, D",    "SET 1, E",    "SET 1, H",    "SET 1, L",    "SET 1, (HL)", "SET 1, A",
/* DX */      "SET 2, B",    "SET 2, C",    "SET 2, D",    "SET 2, E",    "SET 2, H",    "SET 2, L",    "SET 2, (HL)", "SET 2, A",
              "SET 3, B",    "SET 3, C",    "SET 3, D",    "SET 3, E",    "SET 3, H",    "SET 3, L",    "SET 3, (HL)", "SET 3, A",
/* EX */      "SET 4, B",    "SET 4, C",    "SET 4, D",    "SET 4, E",    "SET 4, H",    "SET 4, L",    "SET 4, (HL)", "SET 4, A",
              "SET 5, B",    "SET 5, C",    "SET 5, D",    "SET 5, E",    "SET 5, H",    "SET 5, L",    "SET 5, (HL)", "SET 5, A",
/* FX */      "SET 6, B",    "SET 6, C",    "SET 6, D",    "SET 6, E",    "SET 6, H",    "SET 6, L",    "SET 6, (HL)", "SET 6, A",
              "SET 7, B",    "SET 7, C",    "SET 7, D",    "SET 7, E",    "SET 7, H",    "SET 7, L",    "SET 7, (HL)", "SET 7, A",
];

// Terminal utilities

const RESET: &str = "\x1b[0m";
const ULINE: &str = "\x1b[4m";
