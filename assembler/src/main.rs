mod assembler;
mod binary_to_hex;
mod util;
mod test;

fn main() -> Result<(), assembler::AssemblyError> {
    let asm_filename = "input.asm";
    let binary_filename = "binary.txt";
    let hex_filename = "output.hex";

    let dir = util::find_dir(asm_filename);
    let asm_filepath = util::get_full_path(&dir, asm_filename);
    let binary_filepath = util::get_full_path(&dir, binary_filename);
    let hex_filepath = util::get_full_path(&dir, hex_filename);

    // Parse the assembly code into machine code
    let asm_result = assembler::Assembler::from_file(&asm_filepath);
    let to_return = match asm_result {
        Ok(mut asm) => {
            match asm.save_as_binary(&binary_filepath) {
                Ok(_) => {
                    println!("\nSuccessfully assembled the program!");
                    Ok(())
                },
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Now turn the binary code into hex for Logisim
    let conversion_result = binary_to_hex::convert_hex(&binary_filepath, &hex_filepath);
    match conversion_result {
        Ok(_) => {
            println!("Sucessfully encoded the binary program into hex!")
        },
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    to_return
}