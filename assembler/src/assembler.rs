use std::fs;
use std::fmt;
use std::str::FromStr;
use hash_map::HashMap;
use crate::util::*;

use self::Arg::*;
use ErrorKind::*;

/*
TODO:
- Implement instructions specified in instruction_word_notes.asm
- Figure out how to deal with errors that happen in generated assembly lines
- Add guards against the use of reserved register names
    - Both for aliases and for regular use, where applicable
 */

pub const MAX_REGISTER: usize = 15; // registers 12-15 are reserved (rasp, rara, rat, razero)
pub const REGISTER_PREFIX: &str = "ra"; // (stack pointer, return address, temporary, zero) ^
pub const RAT_ADDR: usize = 14;

// Bit width & max values for immediates
// const IMM_WIDTH: usize = 4;
const REG_ADDR_WIDTH: usize = 4;
const LI_IMM_WIDTH: usize = 8;
const JMPI_IMM_WIDTH: usize = 12;
const MAX_IMM: usize = 15;	  // highest immediate value we can use for most immediate instructions
// (anything > this values becomes a psuedo-instruction and uses li)
const MAX_LI_IMM: usize = 255;	  // highest immediate value we can use with li
const MAX_JMPI_IMM: usize = 4095;
const MAX_16BIT: usize = 65_535;

// Instruction name constants
const LW: &str = "lw";
const SW: &str = "sw";
const JMPR: &str = "jmpr";
const JMPI: &str = "jmpi";
const ADD: &str = "add";
const AND: &str = "and";
const OR: &str = "or";
const SUB: &str = "sub";
const ADDI: &str = "addi";
const ANDI: &str = "andi";
const ORI: &str = "ori";
const SUBI: &str = "subi";
const LI: &str = "li";
const SLL: &str = "sll";

#[derive(Debug)]
/// An enum containing errors that can happen when parsing Assembly code. Each has a String field for an
/// error message.
pub enum AssemblyError {
    InvalidArgument(String),
    SyntaxError(String),
    AliasError(String)
}

/// An enum containing different types of assembly errors. Used for formatting errors based on type.
pub enum ErrorKind {
    InvalidArgument,
    SyntaxError,
    AliasError
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SyntaxError(msg) => write!(f, "{}", msg),
            Self::InvalidArgument(msg) => write!(f, "{}", msg),
            Self::AliasError(msg) => write!(f, "{}", msg),
        }
    }
}

impl AssemblyError {
    /// Return a new AssemblyError of the specified type.
    /// # Arguments
    /// - `err_type` the type of error
    /// - `msg` the error message
    /// # Returns
    /// - `AssemblyError`
    fn new(err_type: ErrorKind, msg: &str) -> Self {
        match err_type {
            InvalidArgument => Self::InvalidArgument{0: msg.to_string()},
            AliasError => Self::AliasError {0: msg.to_string()},
            SyntaxError => Self::SyntaxError{0: msg.to_string()}
        }
    }
}

#[derive(Debug)]
enum Arg {
    Reg,
    Imm,
    BaseAddrReg,
    RTypeInstr,
    ITypeInstr,
    LiInstr,
    PseudoInstr,
    MemInstr,
    RTypeJmp,
    ITypeJmp
}

enum LineType {
    Empty,
    AliasDeclaration,
    Instruction
}

/// A struct to store a line of assembly code and its line number.
#[derive(Clone)]
pub struct AssemblyLine {
    pub num: usize,
    pub contents: String
}

/// A struct to store information about the files being read and their contents.
pub struct Assembler<'a> {
    instructions: Vec<AssemblyLine>,  // assembly code written by the user
    lookup_table: HashMap<&'a str, String>,
    cur_line: AssemblyLine,
    aliases: HashMap<String, String>
}


impl<'a> Assembler<'a> {
    /// Create an assembler instance from a file.
    /// # Arguments
    /// - `input_file` - the full path of the file to read from
    /// # Returns
    /// - `Ok(Assembler)` if the file is parsed successfully
    /// - `Err(std::io::Error)` if the file couldn't be parsed
    pub fn from_file(input_file: &str) -> Result<Self, std::io::Error> {
        let contents = file_to_line_vec(input_file)?;
        Ok(Self::from_vec(contents))
    }

    /// Create an assembler instance from a Vector of assembly lines.
    /// # Arguments
    /// - `lines` - the Vector containing the assembly code
    pub fn from_vec(lines: Vec<AssemblyLine>) -> Self {
        let instructions = lines;
        let lookup_table = Self::initialize_lookup_table();
        let aliases: HashMap<String, String> = HashMap::new();
        let cur_line: AssemblyLine = AssemblyLine{num: 0, contents: String::new()};
        Assembler{instructions, lookup_table, cur_line, aliases}
    }

    /// Create an assembler instance from a &str containing assembly code, splitting instructions on whitespace.
    /// # Arguments
    /// - `assembly_code` - the assembly code to be parsed
    #[allow(dead_code)]
    pub fn from_str(assembly_code: &str) -> Self {
        let contents = assembly_str_to_line_vec(assembly_code);
        Self::from_vec(contents)
    }

    /// Map an I-type instruction to its corresponding R-type instruction.
    fn map_i_to_r_type(&self, op: &str) -> Result<&str, AssemblyError> {
        match op {
            ADDI => Ok(ADD),
            ANDI => Ok(AND),
            ORI => Ok(OR),
            SUBI => Ok(SUB),
            JMPI => Ok(JMPR),
            _ => Err(self.format_err(InvalidArgument, Some(0), Some(&format!(
                "{op} doesn't have a corresponding R-type instruction"))))
        }
    }
    
    /// Load a large (> 8-bit) immediate value into the assembly-reserved register
    /// # Arguments
    /// - `imm` - the immediate value to be loaded
    /// # Returns
    /// - `Ok(Vec<AssemblyLine>)` containing assembly code that stores the immediate value in the rat register
    /// - `Err(AssemblyError)` if there was an error parsing the instruction
    fn load_large_imm(&self, imm: usize)
            -> Result<Vec<AssemblyLine>, AssemblyError> {
        let mut expanded_instructions: Vec<AssemblyLine> = Vec::new();

        // String name of rat register
        let rat: String = format!("{REGISTER_PREFIX}{RAT_ADDR}");

        // Can load into rat with only one instruction if value is 8 bits or less
        if imm <= 255 {
            expanded_instructions.push(AssemblyLine{num: 0, contents: format!(
                "{LI} {rat} {imm}")});
            return Ok(expanded_instructions)
        }

        // Load bits 15-8
        let li_mask: usize = 0x00FF;
        let mut shifted: usize = (imm >> 8) & li_mask;
        let mut line_num: usize = 0;
        expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
            "{LI} {rat} {shifted}")});
            line_num += 1;

        // Load bits 7-4
        let addi_mask: usize = 0x000F;
        shifted = (imm >> 4) & addi_mask;
        if shifted != 0 {  // skip adding 0
            // sll to put the bits in the proper position
            expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
                "{SLL} {rat} {rat} 4")});
                line_num += 1;
            expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
                "{ADDI} {rat} {rat} {shifted}")});
                line_num += 1;
            expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
                "{SLL} {rat} {rat} 4")});
                line_num += 1;
        } else {
            // if bits 7-4 were all 0, we can just shift by 8 instead
            expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
                "{SLL} {rat} {rat} 8")});
                line_num += 1;
        }
        
        // Load bits 3-0
        shifted = imm & addi_mask;
        // skip adding 0
        if shifted != 0 {
            expanded_instructions.push(AssemblyLine{num: line_num, contents: format!(
                "{ADDI} {rat} {rat} {shifted}")});
        }
        Ok(expanded_instructions)
    }

    /// Save the assembled program to a new file in binary format.
    /// # Arguments
    /// - `output_file` - the full path of the output file (will be created if it doesn't exist)
    /// # Returns
    /// - `Ok(())` if the file is saved successfully
    /// - `Err(AssemblyError)` if there was an error assembling the program
    pub fn save_as_binary(&mut self, output_file: &str) -> Result<(), AssemblyError> {
        let output = self.as_string()?.0;
        fs::write(output_file, output).expect("Failed to write the file.");
        Ok(())
    }

    pub fn save_as_asm(&mut self, output_file: &str) -> Result<(), AssemblyError> {
        let output = self.as_string()?.1;
        fs::write(output_file, output).expect("Failed to write the file.");
        Ok(())
    }

    /// Parse the program into machine code and expanded assembly code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` if there was an error assembling the program
    pub fn as_string(&mut self) -> Result<(String, String), AssemblyError> {
        let mut machine_code = String::new();
        let mut asm_code = String::new();

        for line in &self.instructions {
            self.cur_line = self.substitute_aliases(line.clone());
            match self.get_line_type() {
                LineType::Empty => (),
                LineType::AliasDeclaration => {
                    let (name, value) = self.parse_alias()?;
                    self.aliases.insert(name, value);
                },
                LineType::Instruction => {
                    let parsed_line_result = self.parse_instruction(line);
                    match parsed_line_result {
                        Ok(parsed_line) => {
                            let binary_line = &parsed_line.0;
                            let asm_line = &parsed_line.1;
                            if ! binary_line.is_empty() {
                                machine_code.push_str(binary_line);
                                machine_code.push('\n');
                            }
                            if ! asm_line.is_empty() {
                                asm_code.push_str(asm_line);
                                asm_code.push('\n');
                            }
                        },

                        Err(e) => return Err(e)
                    }
                }
            }
        }
        println!("asm code:\n{asm_code}");
        Ok((machine_code, asm_code))
    }

    /// Parse a line containing an alias, and return a tuple containing the alias's name and associated value.
    /// # Arguments
    /// # Returns
    /// - `Ok((String, String))` the name and associated value
    /// - `Err(AssemblyError)` if there was a parsing error
    fn parse_alias(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let args = split_args(line);
        if args.len() != 3 || args[1] != "=" {
            return Err(self.format_err(AliasError, None, None))
        };

        let name = args[0].clone();
        let value = args[2].clone();
        match self.find_arg_type(&value, 2) {
            Err(_) => return Err(self.format_err(AliasError, None, Some("Invalid value for the alias"))),
            _ => ()
        }

        if name.contains('(') || value.contains('(') {
            return Err(self.format_err(AliasError, None, Some("Aliases can't have parentheses")))
        }

        if self.aliases.contains_key(&value) {
            return Err(self.format_err(AliasError, None, Some("Aliases can't store other aliases")))
        }

        Ok((name, value))
    }

    /// Get the type of line of code.
    /// # Returns
    /// - `LineType::Empty` if the line is empty (or consists entirely of comments)
    /// - `LineType::Alias` if the line is defining a Alias
    /// - `LineType::Instruction` if neither of the above are true
    fn get_line_type(&self) -> LineType {
        let line: &AssemblyLine = &self.cur_line;
        match line {
            line if line.contents.is_empty() => LineType::Empty,
            line if is_alias(&line.contents) => LineType::AliasDeclaration,
            _ => LineType::Instruction
        }
    }

    /// Create, format, and return an assembly error.
    /// # Arguments
    /// - `pos_opt` - an optional position number for the invalid argument in question
    /// - `msg_opt` - an optional explanation message to include
    /// # Returns
    /// - The formatted `AssemblyError`
    fn format_err(&self, err: ErrorKind,
        pos_opt: Option<usize>, msg_opt: Option<&str>) -> AssemblyError {
        let line: &AssemblyLine = &self.cur_line;
        let line_num = line.num;
        let contents = &line.contents;

        let pos = match pos_opt {
            Some(pos) => format!(", position {}", pos),
            None => "".to_string()
        };

        let explanation = match msg_opt {
            Some(msg) => format!("Explanation: {msg}"),
            None => "".to_string()
        };

        let mut msg: String = match err {
            InvalidArgument => "\nInvalid argument".to_string(),
            AliasError => "\nInvalid alias".to_string(),
            SyntaxError => "\nSyntax (or unknown) error".to_string()
        };
        msg.push_str(&format!(" on line {line_num}{pos}:\n\t{contents}\n{explanation}"));
        AssemblyError::new(err, &msg)
    }

    /// Take a line of assembly, and return a Vec containing its instruction types.
    /// # Returns
    /// - `Ok(Vec<Arg>)` if the line is parsed properly
    /// - `Err(AssemblyError)` if the line fails to be parsed properly
    fn parse_instr_arg_types(&self) -> Result<Vec<Arg>, AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let instruction_words: Vec<String> = split_args(line);
        instruction_words
        .iter()
        .enumerate()
        .map(|(index, word)| self.find_arg_type(word, index))
        .collect()
    }

    /// Find out what type of argument the given assembly word is.
    /// # Arguments
    /// - `word` - the assembly word to check
    /// # Returns
    /// - `Ok(Arg)` the type of assembly word, if it's a valid word
    /// - `Err(AssemblyError)` if the assembly word is invalid
    fn find_arg_type(&self, word: &str, arg_pos: usize) -> Result<Arg, AssemblyError> {

        // Match arg type
        match word {
            ADD | AND | OR | SUB => Ok(RTypeInstr),
            ADDI | SLL => Ok(ITypeInstr),  // some other I-types are now pseudo-instructions, see below
            ANDI | ORI | SUBI => Ok(PseudoInstr),
            LI => Ok(LiInstr),
            LW | SW => Ok(MemInstr),
            JMPR => Ok(RTypeJmp),
            JMPI => Ok(ITypeJmp),
            word if is_base_addr_register(word) => Ok(BaseAddrReg),
            word if is_register(word) => Ok(Reg),
            word if is_immediate_value(word)  => Ok(Imm),
            _ => Err(self.format_err(InvalidArgument, Some(arg_pos + 1), None))
        }
    }


    /// Substitute all aliases found in an assembly line.
    /// # Arguments
    /// - `line` - the assembly line to read
    /// # Returns
    /// - the assembly line, with all encountered aliases substituted for their values
    fn substitute_aliases(&self, mut line: AssemblyLine) -> AssemblyLine {
        // Note: The implementation for this method is pretty terrible. I kind of just threw something together
        // because I wanted it to work. I'll probably make it better if and when I come back to the project.
        // TODO: Refactor this
        let aliases = &self.aliases;

        let mut subbed_arg: String = String::new();
        // Iterate over the arguments and substitute all the ones that are in the aliases HashMap with their values
        line.contents = {
            split_args(&line)
            .into_iter()
            .map(|arg| String::from({
                // If the arg is in the aliases HashMap
                if let Some(alias_value) = aliases.get(&arg) {
                    alias_value
                } else {
                    // If the arg is in parentheses, remove them and check the aliases HashMap, then add them back
                    let stripped = remove_parentheses(&arg);
                    if let Some(alias_value) = aliases.get(&stripped) {
                        subbed_arg = add_parentheses(&alias_value);
                        &subbed_arg
                    } else {
                        &arg
                    }
                }
            }))
            .collect::<Vec<_>>()
            .join(" ")
        };
       line
    }


    /// Parse one assembly instruction (a full line) into its corresponding machine code.
    /// # Arguments
    /// - `original_line` - the original line of assembly code as written, before aliases were substituted
    /// # Returns
    /// - `Ok(String)` - the machine code as a String, if the assembly was parsed successfully
    /// - `Err(AssemblyError)` - if the line fails to be parsed properly
    fn parse_instruction(&self, original_line: &AssemblyLine) -> Result<(String, String), AssemblyError> {
        let arg_types = self.parse_instr_arg_types()?;

        match arg_types.as_slice() {
            // Valid inputs
            [RTypeInstr, Reg, Reg, Reg] => self.parse_r_type(),
            [ITypeInstr, Reg, Reg, Imm] => self.parse_i_type(),
            [LiInstr, Reg, Imm] => self.parse_li_instr(),
            [MemInstr, Reg, Imm, BaseAddrReg] => self.parse_mem_type(),
            [RTypeJmp, Reg] => self.parse_jmp_r_type(),
            [ITypeJmp, Imm] => self.parse_jmp_i_type(),

            // Invalid inputs
            [RTypeInstr, _, _, Imm] => Err(self.format_err(InvalidArgument, Some(4), Some(
                "R-type instructions don't take an immediate value as an argument."))),

            [ITypeInstr, _, _, Reg] => Err(self.format_err(InvalidArgument, Some(4), Some(
                "I-type instructions take an immediate value, not a register."))),

            [MemInstr, _, _, Reg] => Err(self.format_err(InvalidArgument, Some(4), Some(&format!(
                "Base address register needs parentheses, e.g. ({REGISTER_PREFIX}0) instead of {REGISTER_PREFIX}0")))),

            [MemInstr, _, _, _] => Err(self.format_err(InvalidArgument, Some(4), Some(
                "Memory instructions only take 3 arguments"))),

            [Reg | Imm | BaseAddrReg, ..] => Err(self.format_err(InvalidArgument, Some(1), Some(
                "The first argument must be an operation."))),

            [RTypeJmp, Imm] => Err(self.format_err(InvalidArgument, Some(2), Some(
                "This argument must be a register, not an immediate value."))),

            [ITypeJmp, Reg] => Err(self.format_err(InvalidArgument, Some(2), Some(
                "This argument must be an immediate value, not a register."))),

            [RTypeJmp, _, ..] => Err(self.format_err(InvalidArgument, Some(2), Some(&format!(
                "Too many arguments: {JMPR} only takes the destination register as an argument.")))),

            [ITypeJmp, _, ..] => Err(self.format_err(InvalidArgument, Some(2), Some(&format!(
                "Too many arguments: {JMPI} only takes an immediate value as an argument.")))),

            _ => Err(AssemblyError::new(SyntaxError, &format!(
                "Syntax error on line {}: \n\t{}", original_line.num, original_line.contents)))
        }
    }

    /// Parse a jump (R-type) instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_jmp_r_type(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);
        let op_pos = 0;
        let reg_pos = 1;

        // Get op (theoretically ALWAYS == JMP here, but I'd rather not hardcode it)
        let op = &args[op_pos];
        let reg = &args[reg_pos];
        output.push_str(&self.get_opcode(op)?);

        // Get register address
        output.push_str(&self.get_register_addr(reg, reg_pos)?);

        Ok((output, line.contents.to_string()))
    }

    /// Parse a jump (I-type) instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_jmp_i_type(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);
        let op_pos = 0;
        let imm_pos = 1;

        // Get op (theoretically ALWAYS == "jmpi" here, but I'd rather not hardcode it)
        let op = &args[op_pos];
        let imm_str = &args[imm_pos];
        let imm = <usize>::from_str(imm_str).unwrap();

        // If the value is too large to handle in just one instruction
        if imm > MAX_JMPI_IMM {
            // If the value is too large for the CPU altogether, return an error
            if imm > MAX_16BIT {
                return Err(self.format_err(InvalidArgument, Some(imm_pos + 1), Some(&format!(
                    "Immediate value is above the maximum of {MAX_16BIT}"))));
            }

            // Assemble the instruction using the assembler-reserved rat register in place of an immediate value
            let rat: String = format!("{REGISTER_PREFIX}{RAT_ADDR}");
            let mut expanded_instructions: Vec<AssemblyLine> = self.load_large_imm(imm)?;

            // Create the final line that uses the value now stored in rat
            let new_op: &str = self.map_i_to_r_type(op)?;
            let final_line = AssemblyLine {
                num: expanded_instructions.len() + 1,
                contents: format!("{new_op} {rat}")
            };
            expanded_instructions.push(final_line);

            // Create a new assembler instance to parse the generated instructions
            let mut temp_assembler = Self::from_vec(expanded_instructions);
            return temp_assembler.as_string()
        }

        output.push_str(&self.get_opcode(op)?);

        // Get memory address from immediate
        output.push_str(&self.get_imm_value(imm_str, imm_pos, JMPI_IMM_WIDTH)?);

        Ok((output, line.contents.to_string()))
    }



    /// Parse an li instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_li_instr(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);
        let op_pos = 0;
        let imm_pos = 2;

        // This function is only called when there are 3 args, so access by index is safe
        // Get op (theoretically ALWAYS == "li" here, but I'd rather not hardcode it)
        let op = &args[op_pos];
        let imm_str = &args[imm_pos];
        let imm = <usize>::from_str(imm_str).unwrap();

        // If the value is too large to handle in just one instruction
        if imm > MAX_LI_IMM {
            // If the value is too large for the CPU altogether, return an error
            if imm > MAX_16BIT {
                return Err(self.format_err(InvalidArgument, Some(imm_pos + 1), Some(&format!(
                    "Immediate value is above the maximum of {MAX_16BIT}"))));
            }

            // Assemble the instruction using the assembler-reserved rat register in place of an immediate value
            let rat: String = format!("{REGISTER_PREFIX}{RAT_ADDR}");
            let mut expanded_instructions: Vec<AssemblyLine> = self.load_large_imm(imm)?;

            // Create the final line that uses the value now stored in rat
            let final_line = AssemblyLine {
                num: expanded_instructions.len() + 1,
                contents: {
                    if op == LI {
                        format!("{} {} {} {}", ADDI, &args[1], rat, 0)
                    } else {
                        let new_op: &str = self.map_i_to_r_type(op)?;
                        format!("{} {} {}", new_op, &args[1], rat)
                    }
                }
            };
            expanded_instructions.push(final_line);

            // Create a new assembler instance to parse the generated instructions
            let mut temp_assembler = Self::from_vec(expanded_instructions);
            return temp_assembler.as_string()
        }

        output.push_str(&self.get_opcode(op)?);

        output.push_str(&self.get_register_addr(&args[1], 1)?);

        // Get value
        output.push_str(&self.get_imm_value(imm_str, imm_pos, LI_IMM_WIDTH)?);

        Ok((output, line.contents.to_string()))
        
    }
    
    /// Parse an R-type instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_r_type(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);
        let op_pos = 0;

        // This function is only called when there are 4 args, so access by index is safe
        // Get op
        let op = &args[op_pos];
        output.push_str(&self.get_opcode(op)?);

        // Get register addreeses
        for i in 1..=3 {
            let reg = &args[i];
            output.push_str(&self.get_register_addr(reg, i)?)
        }

        Ok((output, line.contents.to_string()))
    }

    /// Parse an I-type instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_i_type(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);
        let imm_pos = 3;
        let op = &args[0];
        let imm_str = &args[imm_pos];
        let imm = <usize>::from_str(imm_str).unwrap();

        // If the value is too large to be used as a single immediate (>=16)
        if imm > MAX_IMM {
            // If the value is too large for the CPU altogether, return an error
            if imm > MAX_16BIT {
                return Err(self.format_err(InvalidArgument, Some(imm_pos + 1), Some(&format!(
                    "Immediate value is above the maximum of {MAX_16BIT}"))));
            }

            // Assemble the instruction using the assembler-reserved rat register in place of an immediate value
            let rat: String = format!("{REGISTER_PREFIX}{RAT_ADDR}");
            let mut expanded_instructions: Vec<AssemblyLine> = self.load_large_imm(imm)?;

            // Create the final line that uses the value now stored in rat
            let new_op: &str = self.map_i_to_r_type(op)?;
            let final_line = AssemblyLine {
                num: expanded_instructions.len() + 1,
                contents: format!("{new_op} {} {} {rat}", &args[1], &args[2])
            };
            expanded_instructions.push(final_line);

            // Create a new assembler instance to parse the generated instructions
            let mut temp_assembler = Self::from_vec(expanded_instructions);
            return temp_assembler.as_string()
        }

        // Get op
        output.push_str(&self.get_opcode(op)?);

        // Get registers
        for i in 1..=2 {
            let reg = &args[i];
            output.push_str(&self.get_register_addr(reg, i)?)
        }

        // Get immediate value
        output.push_str(&self.get_imm_value(imm_str, imm_pos, 4)?);

        Ok((output, line.contents.to_string()))
    }

    /// Parse a memory-type (SW or LW) instruction into its corresponding machine code.
    /// # Returns
    /// - `Ok(String, String)` - the machine code and assembly code, respectively
    /// - `Err(AssemblyError)` - if there was an error while attempting to parse the line
    fn parse_mem_type(&self) -> Result<(String, String), AssemblyError> {
        let line: &AssemblyLine = &self.cur_line;
        let mut output = String::new();
        let args = split_args(line);

        // Positions of each argument in the instruction
        let op_pos = 0;
        let reg_pos = 1;
        let imm_pos = 2;
        let base_addr_pos = 3;

        // Get the operation being performed
        let op = &args[op_pos];

        // Get the immediate value
        let imm_str = &args[imm_pos];
        let imm = <usize>::from_str(imm_str).unwrap();

        if imm > MAX_IMM {
            // If the value is too large for the CPU altogether, return an error
            if imm > MAX_16BIT {
                return Err(self.format_err(InvalidArgument, Some(imm_pos + 1), Some(&format!(
                    "Immediate value is above the maximum of {MAX_16BIT}"))));
            }

            // Assemble the instruction using the assembler-reserved rat register in place of an immediate value
            let rat: String = format!("{REGISTER_PREFIX}{RAT_ADDR}");
            let mut expanded_instructions = self.load_large_imm(imm)?;

            // First, add previous base address to rat to create new base address, and use an offset of 0
            let orig_base_addr_reg = remove_parentheses(&args[base_addr_pos]);
            let penultimate_line = AssemblyLine {
                num: expanded_instructions.len() + 1,
                contents: format!("{ADD} {rat} {rat} {}", orig_base_addr_reg)
            };
            // The final instruction, using an offset of 0 with the correct base address
            let final_line = AssemblyLine {
                num: expanded_instructions.len() + 2,
                contents: format!("{op} {} 0 ({rat})", &args[reg_pos])
            };
            expanded_instructions.push(penultimate_line);
            expanded_instructions.push(final_line);

            // Create a new assembler instance to parse the generated instructions
            let mut temp_assembler = Self::from_vec(expanded_instructions);
            return temp_assembler.as_string()
        }

        // Append opcode
        output.push_str(&self.get_opcode(op)?);

        // Append src/dest register
        let reg = &args[reg_pos];
        output.push_str(&self.get_register_addr(reg, reg_pos)?);

        // Get immediate offset value (don't append yet, needs to be swapped for machine code)
        let offset_result = self.get_imm_value(&args[imm_pos], imm_pos, 4);

        // Get base address
        let base_addr_result = self.get_register_addr(&args[base_addr_pos], base_addr_pos);

        // Now append offset & base addr in reverse order
        match (offset_result, base_addr_result) {
            (Ok(offset), Ok(base_addr)) => {
                output.push_str(&base_addr);
                output.push_str(&offset);
            }
            (Err(e), _) | (_, Err(e)) => return Err(e),
        }

        Ok((output, line.contents.to_string()))
    }

    /// Get the opcode from the lookup table.
    /// # Arguments
    /// - `op` the name of the operation to look up, e.g. ADD
    /// # Returns
    /// - `Ok(String)` - the machine code for the operation, e.g. 0000
    /// - `Err(AssemblyError)` if the operation doesn't exist
    fn get_opcode(&self, op: &str) -> Result<String, AssemblyError> {
        let opcode_option = self.lookup_table.get(&op);
        match opcode_option {
            Some(opcode) => Ok(opcode.to_string()),
            None => Err(self.format_err(InvalidArgument, Some(1), Some(
                "Invalid operation (not yet implemented)")))
        }
    }

    /// Take a decimal number, and return a String with its value in binary, padded to the specified number of digits.
    /// # Arguments
    /// - `num` - the number to convert
    /// - `max_value` - the max value the number can be
    /// - `digits` - number of digits to pad the String to
    /// # Returns
    /// - `Ok(String)` if the number is parsed successfully and <= the maximum value
    /// - `Err(AssemblyError)` if the number is above the maximum value, or can't be parsed
    fn decimal_to_binary_str(&self, num: usize, max_value: usize, digits: usize) -> Result<String, AssemblyError> {
        if num <= max_value {
            Ok(format!("{:0>digits$b}", num))
        } else {
            Err(self.format_err(InvalidArgument, Some(3), Some(&format!(
                "Number out of range (must be between 0 and {max_value}, inclusive)"))))
        }
    }

    /// Take the name of a register, and return its address as a binary string.
    /// # Arguments
    /// - `arg` - the register argument to parse
    /// - `arg_num` - the index of the argument (used for error messages)
    /// # Returns
    /// - `Ok(String)` if the register address is parsed successfully and is a valid address
    /// - `Err(AssemblyError)` if there is an error while attempting to parse the register
    fn get_register_addr(&self, arg: &str, arg_num: usize) -> Result<String, AssemblyError> {
        let width = REG_ADDR_WIDTH;
        // Remove register prefix (and "()", if applicable) from the arg
        let output = remove_non_numeric(arg);

        match <usize>::from_str(&output) {
            Ok(dec_addr) => Ok(self.decimal_to_binary_str(dec_addr, MAX_REGISTER, width)?),
            Err(e) => Err(self.format_err(InvalidArgument, Some(arg_num), Some(&format!(
                "Failed to parse register address: {e}"))))
        }
    }

    /// Get an immediate value from a string slice argument, and return it as a binary string.
    /// # Arguments
    /// - `arg` - the argument to parse
    /// - `arg_num` - the index of the argument (used for error messages)
    /// - `pad_to` - the number of digits to pad to
    /// # Returns
    /// - `Ok(String)` if the argument is successfully parsed into a valid value
    /// - `Err(AssemblyError)` if the argument can't be parsed, or its value is invalid
    fn get_imm_value(&self, arg: &str, arg_num: usize, pad_to: usize) -> Result<String, AssemblyError> {
        let max_value: usize = 2_usize.pow(pad_to as u32) - 1;
        // Parse the value as a number, then convert it to a padded binary string (with bounds checking)
        match <usize>::from_str(arg) {
            Ok(dec_addr) => Ok(self.decimal_to_binary_str(dec_addr, max_value, pad_to)?),
            Err(e) => Err(self.format_err(InvalidArgument, Some(arg_num), Some(
                &format!("Error while parsing the argument: {}", e))))
        }
    }

    /// Initialize the opcode lookup table with hardcoded values.
    fn initialize_lookup_table() -> HashMap<&'a str, String> {
        let mut table: HashMap<&str, String> = HashMap::new();
        // TODO: Make this *not* disgusting (perhaps implement FromIterator for HashMap?)
        // R-type instructions
        table.insert(ADD, "0000".to_string());
        table.insert(AND, "0001".to_string());
        table.insert(OR, "0010".to_string());
        table.insert(SUB, "0011".to_string());

        // I-type instructions
        table.insert(ADDI, "1000".to_string());
        
        // LI instruction
        table.insert(LI, "1111".to_string());
        
        // SLL instruction
        table.insert(SLL, "1011".to_string());

        // Memory instructions
        table.insert(SW, "1100".to_string());
        table.insert(LW, "1101".to_string());

        // JMP instructions
        table.insert(JMPR, "011000000000".to_string());
        table.insert(JMPI, "1110".to_string());
        

        // Pseudo-instructions:
        // ori and subi use rat with their R-type variants
        // also implement mov, cp (?), inc, dec

        table
    }
}
