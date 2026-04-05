use std::fs;
use std::env;
use std::path::PathBuf;
use crate::assembler::{REGISTER_PREFIX, MAX_REGISTER, AssemblyLine};

/// Remove all whitespace from a string slice, and return the result as a String.
/// # Arguments
/// - `word` - the string from which to remove whitespace
pub fn remove_all_whitespace(word: &str) -> String {
    let mut result = word.to_string();
    result.retain(|c| !c.is_whitespace());
    result
}

/// Trim leading and trailing whitespace from a string slice.
/// # Arguments
/// - `word` - the string slice to trim
/// # Returns
/// - A `String` with leading and trailing whitespace removed
pub fn trim_whitespace(word: &str) -> String {
    word.trim().to_string()
}

/// Count how many digits are in a number.
/// # Arguments
/// - `num` - the number whose digits will be counted
/// # Returns
/// - the number of digits
fn count_digits(num: usize) -> usize {
    num.to_string().chars().count()
}

/// Check whether a word refers to a valid register.
pub fn is_register(word: &str) -> bool {
    let prefix_len = REGISTER_PREFIX.len();
    let min_len = prefix_len + 1;  // register prefix plus 1-digit register number
    let max_len = prefix_len + count_digits(MAX_REGISTER as usize);  // register prefix + digits in max register number
    word.contains(REGISTER_PREFIX) && (min_len <= word.len() && word.len() <= max_len)
}

/// Check whether a word refers to a base address register.
pub fn is_base_addr_register(word: &str) -> bool {
    if is_in_parentheses(word) {
        is_register(&word[1..word.len()-1])
    } else {
        false
    }
}

pub fn is_in_parentheses(word: &str) -> bool {
    word.starts_with('(') && word.ends_with(')')
}

/// Check whether a word is an immediate value.
pub fn is_immediate_value(word: &str) -> bool {
    if word.is_empty() {
        return false
    }
    
    if word.starts_with('-') {  // handle negative numbers
        word.len() > 1 && word[1..].chars().all(|c| c.is_digit(10))
    } else {  // positive numbers
        word.chars().all(|c| c.is_digit(10))
    }
}


/// Format a line of assembly code.
/// # Arguments
/// - `line` - the line of assembly to format
/// # Returns
/// - An all-lowercase `String`, with comments and leading/trailing whitespace removed.
pub fn format_line(line_num: &mut usize, line: &str) -> AssemblyLine {
    let mut result = String::new();
    let number = *line_num;
    *line_num += 1;
    for c in line.chars() {
        // Early return if at the beginning of a comment
        if c == ';' {
            let contents = trim_whitespace(&result);
            return AssemblyLine{num: number, contents}
        }
        result.push(c.to_ascii_lowercase())
    }
    let contents = trim_whitespace(&result);
    AssemblyLine{num: number, contents}
}

/// Parse the contents of a file into a Vec of AssemblyLines.
/// # Arguments
/// - `filename` - the name of the file to parse
/// # Returns
/// - `Ok(Vec<AssemblyLine>)` when the file is parsed successfully
/// - `Err(std::io::Error)` when the file can't be parsed
pub fn file_to_line_vec(filename: &str) -> Result<Vec<AssemblyLine>, std::io::Error> {
    let contents = fs::read_to_string(filename)?;
    Ok(assembly_str_to_line_vec(&contents))
}

/// Parse a &str into a Vector of AssemblyLines, splitting on newline characters.
pub fn assembly_str_to_line_vec(assembly_code: &str) -> Vec<AssemblyLine> {
    let mut line_num: usize = 1;
    // Unpack into string vec for each line
    assembly_code
        .lines()
        .map(|line| {format_line(&mut line_num, line)})
        .filter(|s| !s.contents.is_empty())
    .collect()
}

/// Split a line of assembly code into a Vec of arguments, splitting on whitespace.
/// # Arguments
/// - `line` - the line of assembly to split
/// # Returns
/// - A vector of `String` arguments
pub fn split_args(line: &AssemblyLine) -> Vec<String> {
    line
        .contents
        .split_whitespace()
        .map(|word| word.to_string())
    .collect()

}

/// Remove all parentheses from a string slice.
/// # Arguments
/// - `word` - the string slice from which parentheses shall be removed
/// # Returns
/// - A `String` with the parentheses removed
pub fn remove_parentheses(word: &str) -> String {
    let mut output = String::new();
    for c in word.chars() {
        if c != '(' && c != ')' {
            output.push(c)
        }
    }
    output
}

/// Remove all non-numeric characters from a word, and return the resulting String.
/// # Arguments
/// - `word` - the word to format
/// # Returns
/// - A `String` containing only numeric characters
pub fn remove_non_numeric(word: &str) -> String {
    let mut output = String::new();
    for c in word.chars() {
        if c.is_numeric() {
            output.push(c)
        }
    }
    output
}

/// Determine whether or not the given assembly line is declaring an alias
/// # Arguments
/// - `line` - the line of code to be checked
/// # Returns
/// - `True` if the line of code is declaring an alias, otherwise `False`
pub fn is_alias(line: &str) -> bool {
    line.contains('=')
}

/// Find the directory in which the assembly code is stored.
/// # Arguments
/// - `input_filename` - the name of the file containing the assembly code
/// # Returns
/// - A `String` containing the full path to the file in question, including the filename itself
pub fn find_dir(input_filename: &str) -> String {
    let error_msg_cwd: &str = "Error: failed to find the current working directory.";
    let error_msg_exe: &str = "Error: Failed to find the executable file's path.";

    // First check if file is in cwd
    let cwd = env::current_dir().expect(error_msg_cwd);
    let mut to_return = cwd.clone();
    let full_path = cwd.join(input_filename);

    // If path doesn't exist, check .exe dir
    if ! full_path.exists() {
        let exe_path = env::current_exe().expect(error_msg_exe);
        let cwd = exe_path.parent().expect(error_msg_exe);
        to_return = PathBuf::from(cwd);
        let full_path = cwd.join(input_filename);

        // If the input file isn't here, either, then exit the program
        if ! full_path.exists() {
            eprintln!("{}", error_msg_cwd);
            std::process::exit(1);
        }
    }
    to_return.to_str().expect(error_msg_cwd).to_string()

}

/// Combine a directory and the name of a file within it into one String.
/// # Arguments
/// - `directory` - the directory the file is in
/// - `filename` - the name of the file
/// # Returns
/// - A `String` containing the full path of the file
pub fn get_full_path(directory: &str, filename: &str) -> String {
    let dir = PathBuf::from(directory);
    let result = dir.join(filename).to_str().expect("pls write a better error msg").to_string();
    result
}