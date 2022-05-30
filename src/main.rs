use std::io::{self, Read};
use std::error::Error;

mod hack {
    use std::iter::Iterator;

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
        A(&'a str),
        C(&'a str, &'a str, &'a str)
    }

    impl Instruction<'_> {
        fn parse_comp_and_jump(s: &str) -> (&str, &str) {
            let comp_and_jump_parts: Vec<&str> = s.split(";").collect();

            if comp_and_jump_parts.len() == 1 {
                (comp_and_jump_parts[0].trim(), "")
            } else {
                (comp_and_jump_parts[0].trim(), comp_and_jump_parts[1].trim())
            }
        }

        fn c_from_string(s: &str) -> Instruction {
            let mut dest = "";
            let comp;
            let jump;

            let dest_parts: Vec<&str> = s.split("=").collect();

            if dest_parts.len() == 1 {
                (comp, jump) = Self::parse_comp_and_jump(dest_parts[0]);
            } else {
                dest = dest_parts[0];
                (comp, jump) = Self::parse_comp_and_jump(dest_parts[1]);
            }

            Instruction::C(dest, comp, jump)
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
    let output: Vec<hack::Instruction> = output.collect(); //::<Vec<String>>().join("\n");

    println!("{:#?}", output);

    Ok(())
}
