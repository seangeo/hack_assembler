use std::io::{self, Read};
use std::error::Error;

mod hack {
    use phf::phf_map;

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

    pub fn strip_comment<'a>(line: &'a str) -> Option<&'a str> {
        let line_parts: Vec<&str>= line.split("//").take(1).collect();
        let line = line_parts[0].trim();

        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    pub fn to_instruction(line: &str) -> Instruction {
        match line.strip_prefix("@") {
            Some(address) => Instruction::A(address),
            None => Instruction::c_from_string(line)
        }
    }

    #[derive(Debug)]
    pub enum Instruction<'a> {
        // A - address
        A(&'a str),
        // C - dest, comp, jump
        C(&'a str, &'a str, &'a str)
    }

    impl Instruction<'_> {
        fn c_from_string(s: &str) -> Instruction {
            let dest_parts: Vec<&str> = s.split("=").collect();

            if dest_parts.len() == 1 {
                let (comp, jump) = Self::parse_comp_and_jump(dest_parts[0]);
                Instruction::C("", comp, jump)
            } else {
                let (comp, jump) = Self::parse_comp_and_jump(dest_parts[1]);
                Instruction::C(dest_parts[0], comp, jump)
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
    let mut input_string = String::new();
    io::stdin().lock().read_to_string(&mut input_string)?;

    let output = input_string.lines();
    let output = output.filter_map(|line| hack::strip_comment(line));
    let output = output.map(|line| hack::to_instruction(line));
    let output = output.map(|instruction| instruction.to_binary());
    let output = output.collect::<Vec<String>>().join("\n");

    println!("{}", output);

    Ok(())
}
