; this is a comment!
; this program reverses the order of an "array" of length 6 located at memory addresses 0-5.

addi ra7 ra7 3 ; inline comments work too

lw ra0, 2 (ra7) ; you can use commas between arguments
; lw ra0, 2, (ra7) ; but this doesn't work because the last base address and offset are treated as one argument
                  ; try uncommenting it and see what happens!
lw ra2 0 (ra7)
lw ra1 1 (ra7)
lw ra3 2 (ra6)
lw ra4 1 (ra6)
lw ra5 0 (ra6)

; you can define aliases to make your code more readable
zero = 0	; substitute 0 in anywhere "zero" is found before the code is assembled
; this is terrible and probably makes the code *less* readable, but you get the point

sw ra0 zero (ra6) ; equivalent to "sw ra0 0 (ra6)"

; you can use aliases for registers too
zero_base_addr = ra6
sw ra1 1 (zero_base_addr) ; equivalent to "sw ra1 1 (ra6)"
sw ra2 2 (ra6)

; had to do this before, because of the limitations of a 4-bit CPU and the size of our instruction word
;sw ra3 0 (ra7)
;sw ra4 1 (ra7)
;sw ra5 2 (ra7)

; now you can use immediates greater than 3, so there's no need to use a second register with a precomputed offset
; (at least, when the "array" only has 6 elements!)
sw ra3 3 (zero_base_addr)
sw ra4 4 (zero_base_addr)
sw ra5 5 (zero_base_addr)

; aliases can't point to one another
; zero_base_addr_but_cooler = zero_base_addr ; try uncommenting this line and see what happens!

jmpi 0
jmpr ra0