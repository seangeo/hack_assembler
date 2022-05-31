use std::io::{self, Read};
use std::error::Error;

mod hack {
    use phf::phf_map;
    use std::collections::HashMap;

    static COMP_MAP: phf::Map<&'static str, &'static str> = phf_map! {
        "0"   => "0101010",
        "1"   => "0111111",
        "-1"  => "0111010",
        "D"   => "0001100",
        "A"   => "0110000",
        "!D"  => "0110001",
        "!A"  => "0110011",
        "-D"  => "0001111",
        "-A"  => "0110011",
        "D+1" => "0011111",
        "A+1" => "0110111",
        "D-1" => "0001110",
        "A-1" => "0110010",
        "D+A" => "0000010",
        "D-A" => "0010011",
        "A-D" => "0000111",
        "D&A" => "0000000",
        "D|A" => "0010101",
        "M"   => "1110000",
        "!M"  => "1110001",
        "-M"  => "1110011",
        "M+1" => "1110111",
        "M-1" => "1110010",
        "D+M" => "1000010",
        "D-M" => "1010011",
        "M-D" => "1000111",
        "D&M" => "1000000",
        "D|M" => "1010101"
    };

    static DEST_MAP: phf::Map<&'static str, &'static str> = phf_map! {
        "M" => "001",
        "D" => "010",
        "MD" => "011",
        "A" => "100",
        "AM" => "101",
        "AD" => "110",
        "AMD" => "111"
    };

    static JUMP_MAP: phf::Map<&'static str, &'static str> = phf_map! {
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111"
    };

    static VARIABLES: phf::Map<&'static str, i16> = phf_map! {
        "R0" => 0,
        "R1" => 1,
        "R2" => 2,
        "R3" => 3,
        "R4" => 4,
        "R5" => 5,
        "R6" => 6,
        "R7" => 7,
        "R8" => 8,
        "R9" => 9,
        "R10" => 10,
        "R11" => 11,
        "R12" => 12,
        "R13" => 13,
        "R14" => 14,
        "R15" => 15,
        "SP" => 0,
        "LCL" => 1,
        "ARG" => 2,
        "THIS" => 3,
        "THAT" => 4,
        "SCREEN" => 16384,
        "KDB" => 24576
    };

    pub fn strip_comments<'a>(lines: std::str::Lines<'a>) -> Vec<&'a str> {
        lines.filter_map(|line| {
            let line_parts: Vec<&str> = line.split("//").collect();
            let line = line_parts[0].trim();

            if line.is_empty() {
                None
            } else {
                Some(line)
            }
        }).collect()
    }

    pub fn filter_labels<'a>(lines: Vec<&'a str>, symbol_table: & mut SymbolTable) -> Vec<&'a str> {
        let mut line_count = 0;

        lines.into_iter().filter_map(|line|
            if line.starts_with("(") && line.ends_with(")") {
                let symbol = line.strip_prefix("(").unwrap().strip_suffix(")").unwrap();
                symbol_table.insert_label(symbol, line_count);
                None
            } else {
                line_count = line_count + 1;
                Some(line)
            }
        ).collect()
    }

    pub fn parse_instructions(lines: Vec<&str>) -> Vec<Instruction> {
        lines.iter().map(|line|
            match line.strip_prefix("@") {
                Some(address) => Instruction::A(address.to_string()),
                None => Instruction::c_from_string(line)
            }
        ).collect()
    }

    pub fn resolve_symbols<'a>(instructions: Vec<Instruction>, symbol_table: &'a mut SymbolTable) -> Vec<Instruction> {
        let mut result = Vec::new();

        for instruction in instructions {
            let i = match instruction {
                Instruction::A(address) => Instruction::A(symbol_table.apply_to_address(&address)),
                Instruction::C(d, c, j) => Instruction::C(d, c, j)
            };

            result.push(i);
        }

        result
    }

    pub struct SymbolTable {
        table: HashMap<String, i16>,
        variable_count: i16
    }

    pub fn new_symbol_table() -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            variable_count: 16
        }
    }

    impl SymbolTable {
        pub fn insert_label(&mut self, symbol: &str, value: i16) {
            self.table.insert(symbol.to_string(), value);
        }

        pub fn apply_to_address<'a>(&'a mut self, address: &'a str) -> String {
            // check if we just have a number, which is a literal address
            if address.parse::<i16>().is_ok() {
                address.to_string()
            } else {
                // First try fixed symbols
                match VARIABLES.get(address) {
                    Some(value) => value.to_string(),
                    // Then try variables we've found in the file
                    None => match self.table.get(address) {
                        Some(value) => value.to_string(),
                        None => self.create_new_variable(address)
                    }
                }
            }
        }

        fn create_new_variable<'a>(&mut self, address: &'a str) -> String {
            let new_variable = self.variable_count;
            self.variable_count = self.variable_count + 1;
            self.table.insert(address.to_string(), new_variable);

            new_variable.to_string()
        }
    }

    #[derive(Debug)]
    pub enum Instruction {
        // A - address
        A(String),
        // C - dest, comp, jump
        C(String, String, String)
    }

    impl Instruction {
        fn c_from_string(s: &str) -> Instruction {
            let dest_parts: Vec<&str> = s.split("=").collect();

            if dest_parts.len() == 1 {
                let (comp, jump) = Self::parse_comp_and_jump(dest_parts[0]);
                Instruction::C("".to_string(), comp.to_string(), jump.to_string())
            } else {
                let (comp, jump) = Self::parse_comp_and_jump(dest_parts[1]);
                Instruction::C(dest_parts[0].to_string(), comp.to_string(), jump.to_string())
            }
        }

        fn parse_comp_and_jump(s: &str) -> (&str, &str) {
            let comp_and_jump_parts: Vec<&str> = s.split(";").collect();

            if comp_and_jump_parts.len() == 1 {
                (comp_and_jump_parts[0].trim(), "")
            } else {
                (comp_and_jump_parts[0].trim(), comp_and_jump_parts[1].trim())
            }
        }

        pub fn to_binary(&self) -> String {
            match self {
                Self::A(address) => format!("0{:0>15b}", address.parse::<i16>().unwrap()),
                Self::C(dest, comp, jump) => {
                    format!("111{}{}{}",
                        COMP_MAP.get(comp).unwrap(),
                        DEST_MAP.get(dest).unwrap_or(&"000"),
                        JUMP_MAP.get(jump).unwrap_or(&"000"))
                }
            }
        }
    }
}

/* Assembles a Hack Assembly program into a .hack "binary" file.
 *
 * Asm should be provided on STDIN.
 * Hack Machine language will be output on STDOUT.
 */
fn main() -> Result<(), Box<dyn Error>> {
    let mut symbol_table = hack::new_symbol_table();
    let mut input_string = String::new();
    io::stdin().lock().read_to_string(&mut input_string)?;

    let lines = hack::strip_comments(input_string.lines());
    let lines = hack::filter_labels(lines, &mut symbol_table);
    let instructions = hack::parse_instructions(lines);
    let instructions = hack::resolve_symbols(instructions, &mut symbol_table);
    let output: Vec<String> = instructions.iter().map(|instruction| instruction.to_binary()).collect();

    println!("{}", output.join("\n"));

    Ok(())
}
