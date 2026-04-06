Old instruction word (24-bit for 8-bit CPU):
IMM  SW  LW  JMP OP    DST   SRC1  SRC2/IMM
1    1   1   1   4     4     4     8


16-bit instruction word for 16-bit CPU:
4  4   4        4
OP DST SRC1/IMM SRC2/IMM

# Note: IMM/SW/LW are now part of opcode, rather than each being their own field
# but first bit being 1 *usually* means an IMM instruction... just not always
Opcode encoding:
add     = 0000
and     = 0001  # andi and ori become pseudo-instructions that use $at with the immediate value
or      = 0010
sub     = 0011  # subi inverts the immediate and assembles to use addi instead
jmpr    = 0110
addi    = 1000
sll     = 1011
sw      = 1100
lw      = 1101
jmpi    = 1110
li      = 1111  # in addition to its normal uses, also used for imm instructions when 15 < imm value <= 255

Allocated but not yet implemented:
slt = 0101
jal = 1010
beq = 0111
bne = 1001

Unallocated opcodes:
0100

Pseudo-instructions using rat:
andi
ori
subi

Regular pseudo instructions (not yet implemented):
mov - move a value from src reg to dst reg
cp - identical to move, except the name actually makes sense
inc - increment a register's value by 1 (equivalent to addi ra[x] ra[x] 1)
dec - decrement a register's value by 1 (equivalent to addi ra[x] ra[x] -1)
