use std::fs;
use std::fmt;
use crate::util::*;

const FILE_HEADER: &str = "v2.0 raw\n";

#[derive(Debug)]
pub enum HexEncodingError {
    FileReadError(String),
    ParseError(String)
}

impl fmt::Display for HexEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileReadError(msg) => write!(f, "\nFile read error: {msg}"),
            Self::ParseError(msg) => write!(f, "Error while parsing machine code: {msg}"),
        }
    }
}

/// Convert a file containing instructions in binary into hex
/// # Arguments:
/// - `binary_filename` - the name of the input binary file
/// - `hex_filename` - the name to give the output file
/// # Returns:
/// - `Ok(())` if the operation was sucessful
/// - `Err(HexEncodingError)` if there was an error
pub fn convert_hex(binary_filename: &str, hex_filename: &str) -> Result<(), HexEncodingError> {
    let contents_result = file_to_line_vec(binary_filename);
    let mut output = String::from(FILE_HEADER);
    let to_return = match contents_result {
        Ok(contents) => {
            let len = contents.len();
            let mut i: usize = 0;
            for line in contents {
                let converted_line_result = convert_line(&line.contents);
                match converted_line_result {
                    Ok(converted_line) => output.push_str(&converted_line),
                    Err(e) => return Err(e)
                }
                i += 1;
                if i < len {
                    output.push(' ');
                }
                
            }
            Ok(())
        },
        Err(e) => Err(HexEncodingError::FileReadError(e.to_string()))
    };
    fs::write(hex_filename, output).expect("Failed to write the file.");
    to_return
}

fn convert_line(line: &str) -> Result<String, HexEncodingError> {
    let decimal_value_result = <u32>::from_str_radix(line, 2);
    match decimal_value_result {
        Ok(value) => {
            let result = &format!("{:0>6x}", value);
            Ok(result.to_string())
        },
        Err(e) => {
            let error_msg = format!("{e}\nError encountered on this line:\n\t{line}\n");
            Err(HexEncodingError::ParseError(error_msg))
        }
    }
}