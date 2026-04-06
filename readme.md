## Context

This project was originally my final project for CSC-270, my computer organization class. For that class, students were given the option to pick their own final project. I chose to make an 8-bit version of the 4-bit CPU I'd already implemented for a lab assignment, as well as an assembler to go with it. I had enough fun with that ~~and was displeased enough with it~~ that I decided to make a 16-bit version with more features.

## Limitations of the 8-bit version

#### Instruction word

For the 4-bit CPU we made in class, we were given a 16-bit instruction word that we had to implement. I ended up doing something similar with my 8-bit CPU, where the instruction word was 24 bits. It was smaller than the original (relative to the bit width of the CPU itself), but I still wasn't really happy with it. So this 16-bit CPU uses a 16-bit instruction word, and uses some neat assembler trickery to avoid some problems with such a thing. More on that later.

#### Number of instructions

The 4 and 8-bit CPUs and their assemblers didn't implement very many instructions - just the very basics:

- add, and, or, sub (and their immediate equivalents)

- sw (store word)

- lw (load word)

- (8-bit CPU only) jmp (now called jmpr) and jmpi
  
  - jmpr jumps to the address in the specified register
  
  - jmpi jumps to an immediate address

In addition to these, the 16-bit assembler implements:

- li (load immediate)

- sll (shift left logical)

And I have plans to implement the following:

- slt (set less than)

- jal (jump and link)

- beq (branch if equal)

- bne (branch if not equal)

- mov (move, except it actually copies a value instead of moving it but this is apparently the convention for some reason)

- cp (short for copy. same as mov, except the name actually makes sense)

- inc (short for increment. increments the value of a register by 1)

- dec (short for decrement. I think you can figure it out.)

How did I fit all of this into a tiny 16-bit instruction word, you might ask? Well... I didn't. I kind of cheated. Please see the "technical details" section below.

## Assembly Technical Details

In this section, I'll go over all of the fancy features this assembler has, starting with...

#### Pseudo-instructions

The main trick I used to pack so much information into only a 16-bit opcode is by using lots of pseudo-instructions. Maybe too many. I set aside a special register, called 'rat' (**t**emporary register), used to store intermediate values. The assembler will decide when this is necessary and expand these pseudo-instructions into multiple real ones. For example, the field in my instruction word for immediate values is normally only 4 bits. However, instructions with only one register argument (such as li) have 4 bits that are unused. In this case, the CPU will treat the lower 8 bits as an immediate value, instead of just the lower 4, expanding the range from 0-15 to 0-255. The assembler knows this and won't bother expanding the instructions. But let's say you want to use an immediate value larger than 4 bits for an instruction that takes two register arguments, like addi. Take the following line of code as an example:

```asm6502
addi ra0 ra1 128
```

Because 128 can't fit into just 4 bits, the assembler will expand that instruction into two:

```asm6502
li rat 128  ; load 128 into temporary register

add ra0 ra1 rat  ; add temporary register to ra1, store in ra0
```

And when the value is larger than 255:

```asm6502
addi ra0 ra1 256
```

... assembles to:

```asm6502
li ra14 1  ; load 0b000_0000_0000_0001 = 0d1
sll ra14 ra14 8  ; shift it over by 8 -> 0b0000_0001_0000_0000 = 0d256
add ra0 ra1 ra14
```

The assembler will do something similar for most operations, using as few instructions as it can.

#### Aliases

Another important feature of my assembly language is *aliases*. They're essentially just text substitution - think along the lines of `##define true 1` in C. They can be used for nearly anything, as long as you're not using a reserved word (such as the name of an instruction).

#### Comments

You know what a comment is already... don't you?

```asm6502
; this is a comment!
li ra0 23  ; this works too
```

#### Error Handling

My assembler supports some (limited) error handling. If there's an error in your code, you'll get an error message telling you where the problem is. Depending on the error, you might also get an explanatory error message. For example, consider the following program:

```asm6502
x = 5
add ra0 ra0 x
```

The following error message will be produced:

```asm6502
Invalid argument on line 2, position 4:
        add ra0 ra0 5
Explanation: R-type instructions don't take an immediate value as an argument.
```

#### More Details

For more details about the assembly language I implemented, see `reverse_array_with_comments.asm` for some examples and `instruction_word_notes.asm` for more details about the instruction word and supported instructions. Note that the latter of these is not actually supposed to be an assembly program; I just used the .asm file extension because I wanted cool syntax highlighting in my IDE.

## Non-project files

My assembler has my own HashMap implementation as a prerequisite, which in turn has my LinkedList implementation as a prerequisite. I've included both of these in case you'd like to compile and run my assembler code yourself, but please don't look at them. I haven't looked at them for a while, but they're probably disgusting.

## Running the assembler

I've included a precompiled version of the assembler for MacOS running on ARM and Windows running on x86_64. If you'd like to compile and run it yourself, make sure you have a Rust compiler installed, navigate to /assembler/ and run `cargo run`.

## Limitations

As of right now, the input file is hard-coded to be named "input.asm", and it should be in the same directory as the executable. The output file names are hardcoded as well. I plan to make these things more configurable in the future, but for now, you can just swap around the names of the input files, renaming whichever one you want to assemble to "input.asm".
