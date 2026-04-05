# Pre-Context Context

This project was originally written for my CSC-270 (Computer Organization) class; this README was originally written for the version of the project that I submitted - so it's quite outdated, and addressed directly to my professor, and... just generally not accurate. The most important change is that the assembler is now for a 16-bit CPU (which I haven't implemented (yet?)) instead of an 8-bit one. Also, the instruction word is now only 16 bits, so instructions can actually fit in memory. I'll get around to updating this README when I have more time, but for anyone reading this now, just keep in mind that there will be a lot of inconsistencies. That being said, the code itself is documented fairly well, and most of the inconsistencies will be in this file.

# Context

As of right now, this project is far from complete. Like we talked about in person, I just wanted to submit something so I could focus on studying for finals and not worry about finishing this _too much_. 

# Summary of the project

This project consists of two main parts - a CPU and an assembler for it. The CPU is just an 8-bit version of my Lab 4 CPU that also supports rudimentary jump instructions; it uses an instruction word I designed. The assembler is written in Rust and uses my own custom assembly language, the details of which are outlined below.

# CPU Overview & Features

My CPU is capable of everything the Lab 4 one was, plus some jump instructions (somewhat different from those of MIPS, more details later). It has 16 unnamed registers, and supports up to 8-bit immediate values. It uses a 24-bit instruction word, which only really works because I have separate instruction and data memory, and they have different bit widths. For the final version of this project, if I even make it that far, I plan to fix this issue and add support for various other instructions, in particular branch and its related instructions (slt, etc.). For now, the only extra instructions it has are two jump instructions, jmp (equivalent to jr) and jmpi (equivalent to j). Here are the fields of my instruction word:

```
IMM  SW  LW  JMP OP    DST   SRC1  SRC2/IMM
0    0   0   0   0000  0000  0000  0000/0000
```

I used 8 bits for the last field so that the user can use 8-bit immediate values, which is really convenient, because the length of my instruction word was already well over 8 bits. For the 32-bit version, ~~if I ever get around to making it,~~ my instruction word will be 32 bits.

# Assembler features

The most noteworthy feature of my assembler is that it supports what I've been calling "aliases". They're functionally similar to the constants you can define for QtSpim, but they work for registers as well as immediate values, so they're more flexible. The assembler also has some helpful error messages for invalid syntax, etc.

# Non-project files

My assembler has my own HashMap implementation as a prerequisite, which in turn has my LinkedList implementation as a prerequisite. I've included both of these in case you'd like to compile and run my assembler code yourself, but please don't look at them. They're disgusting.

# Running the assembler

I've included an executable binary (/logisim/assembler) for my assembler so you don't have to compile it yourself. If you'd like to compile it yourself, navigate to /assembler/8-bit_assembler/ in your terminal and run `cargo run`

# Limitations

As of right now, the input file is hard-coded to be named "input.asm", and it should be in the same directory as the executable (in this case, /logisim/). The output file names are hardcoded as well. I plan to make these things more configurable in the future, but for now, you can just swap around the names of the input files, renaming whichever one you want to assemble to "input.asm". Also, empty lines or lines that are just comments aren't counted for the line number, so the line number for error messages that contain a line number refer to the nth line *that has an instruction on it*, not the nth line overall. I plan to modify this behavior as well.

# Assembly Syntax

My assembly syntax is very similar to that of MIPS assembly. The main difference is the names of the registers; I'm using a prefix of ra instead of $. Additionally, I have a space between the immediate value and the base address register for lw/sw instructions. I'll probably fix that at some point so the assembly code looks nicer, but I'm prioritizing the actual functionality of the assembler for now. The operations also have pretty similar names, with the only difference being the jump instructions, as outlined above. Refer to /assembler/8-bit_assembler/input.asm for an example (this is the reverse program from lab 4).
