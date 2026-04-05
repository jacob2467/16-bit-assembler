#[cfg(test)]
pub mod tests {
	use crate::{util, assembler::Assembler};

	/// Format an instruction to be printed
	fn format_instr(instruction: &str) -> String {
		let mut i: usize = 0;
		let mut result = String::new();
		let instruction = util::remove_all_whitespace(instruction);
		while i < instruction.len() {
			result.push(instruction.chars().nth(i).unwrap());

			// Add a line break after every line of code
			if i % 16 == 15 && i < instruction.len() - 1 {
				result.push('\n');
				for _ in 0..2 {
					result.push('\t')
				}
			// Add a space between each field of the instruction word
			} else if i % 4 == 3 {
				result.push(' ');
			}
			i += 1;
		}
		result
	}

	fn print_instr(expected: &str, actual: &str) {
		println!("\texpected:\n\t\t{}\n\n\tactual:\n\t\t{}", expected, actual);
	}

	fn test_code(test_name: &str, code: &str, expected: &str) {
		println!("Testing {test_name}:");
		let mut assembler = Assembler::from_str(code);
		match assembler.as_string() {
			Ok((actual, _)) => {
				let expected = format_instr(expected);
				let actual = format_instr(&actual);
				if expected != actual {
					print_instr(&expected, &actual);
				}
				assert_eq!(expected, actual);
				println!()
			}
			Err(e) => {
				eprintln!("{}\n", e);
				assert!(false);
			}
		}
	}

	#[test]
	fn test_sw_overflow() {
		// Test overflow with a value of 16 (5 bits, 4-bit imm)
		let mut test_name = "sw_overflow_16";
		let mut code = "sw ra9 16 (ra9)";
		let mut expected_binary = util::remove_all_whitespace(
			"1111 1110 0001 0000\
             0000 1110 1110 1001\
             1100 1001 1110 0000");
		test_code(test_name, code, &expected_binary);

		// Test overflow with a value of 16 (5 bits, 4-bit imm)
		test_name = "sw_overflow_255";
		code = "sw ra9 255 (ra9)";
		expected_binary = util::remove_all_whitespace(
			"1111 1110 1111 1111\
             0000 1110 1110 1001\
             1100 1001 1110 0000");
		test_code(test_name, code, &expected_binary);

		// Test overflow with a value of 256 (9 bits, 4-bit imm)
		test_name = "sw_overflow_256";
		code = "sw ra9 256 (ra9)";
		expected_binary = util::remove_all_whitespace(
			"1111 1110 0000 0001\
             1011 1110 1110 1000\
             0000 1110 1110 1001\
             1100 1001 1110 0000");
		test_code(test_name, code, &expected_binary);

		// Test overflow with a value of 65535 (16 bits, 4-bit imm)
		test_name = "sw_overflow_65535";
		code = "sw ra9 65535 (ra9)";
		expected_binary = util::remove_all_whitespace(
			"1111 1110 1111 1111\
             1011 1110 1110 0100\
             1000 1110 1110 1111\
             1011 1110 1110 0100\
             1000 1110 1110 1111\
             0000 1110 1110 1001\
             1100 1001 1110 0000");
		test_code(test_name, code, &expected_binary);
	}

	#[test]
	fn test_sw_0() {
		let test_name = "sw_0";
		let code = "sw ra0 0 (ra0)";
		let expected = util::remove_all_whitespace("1100 0000 0000 0000");
		test_code(test_name, code, &expected);
	}

	#[test]
	fn test_sw() {
		let test_name = "sw";
		let code = "sw ra9 6 (ra9)";
		let expected = util::remove_all_whitespace("1100 1001 1001 0110");
		test_code(test_name, code, &expected);
	}

	#[test]
	fn test_lw_0() {
		let test_name = "lw_0";
		let code = "lw ra0 0 (ra0)";
		let expected = util::remove_all_whitespace("1101 0000 0000 0000");
		test_code(test_name, code, &expected);
	}

	#[test]
	fn test_lw() {
		let test_name = "lw";
		let code = "lw ra9 6 (ra9)";
		let expected = util::remove_all_whitespace("1101 1001 1001 0110");
		test_code(test_name, code, &expected);
	}

}