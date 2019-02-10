# Nonstandard Assembler Extensions
## Compatibility
MARIE Source will assemble under Mars,
but Mars Source probably won't assemble under MARIE.

Mars assembly is a superset of MARIE's

## Lax Labels
Labels do not have to be directly followed by an instruction.
```MARIE
/ Standard MARIE
MyLabel1, Halt
/ Extension: You can put instruction on a newline
MyLabel2,
Halt
/ Extension: You can put multiple labels before an instruction (also works on one line)
MyLabel3,
MyLabel4,
Halt
/ Extension: You can label the end of the file
MyLabel5,
```
### Motivation:
Convenience.
Allowing label aliasing makes subroutine variable sharing easier.
## Label Literals (Lbl)
Similar to JnS, but will work with 16-bit addresses. (an interpreter extension discussed below)
```MARIE
/Standard MARIE
myVarAddr1, JnS myVar / 0001
/Extension: Lbl literal. (does not truncate address to 12 bits)
myVarAddr2, Lbl myVar / 0001

myVar, dec
```
### Motivation:
`JnS` can only store the 12 least significant bits of an address, and with 16 bit addresses, its good to be able to take the full address of a label.
This is especially relevant when relocating programs, for adjusting their addresses.

## Relocatable Code
### (Draft)
Allow code to reside in any location in extended memory by generating metadata about the assembled file.

`.rel` is the file extension I'll use to refer to this metadata

```
//example.mas   .bin    .rel (2 bits at a time)
Load ONE    /   1003    10
Output      /   6000    00
Halt        /   7000    00
ONE, Dec 1  /   0001    00
Lbl ONE     /   0003    11
```


## Linking
### (Draft)
Assembler will include `Ext` literal, for an externally linked address,
and a `Pub` prefix for label.

`Pub`lic labels will have their address and name saved to a `.lnk` file along with the binary, followed by the addresses of any `Ext`ernal literals.
At runtime, a linker will iterate over the .rel files and link them.

```MARIE
// Bin.mas
// A simulated JnS, using indirect instructions.
Load MyRetAddr
StoreI Multiply_r
JumpI Multiply
MyRet, Output
Halt
/ Variables:
Multiply, Ext Multiply+1
Multiply_retaddr, Ext Multiply
MyRetAddr, Jns MyRet

// Lib.mas
/ A 'multiply' function that just returns 1
Pub Multiply, Halt
Pub Multiply+1, Load ONE
JumpI Multiply
ONE, Dec 1
```

### Motivation:

## Skipcond Not Equal condition
### (Unimplemented)
```
binary description
000   Less than..
400   Equal..
800   Greater than..
B00   Not Equal.. (NEW!)
.. Zero.

Example:
Skipcond B00
Jump ACC_WAS_ZERO
```
### Motivation
This would improve readability of skipconds,
and allow for flexibility regarding zero.

Not Equal generates binaries incompatible with MARIE,
but the Mnemonics are entirely an assembler extension.

# Interpreter Extensions
*Embrace, Extend, Extinguish.*

## 16 bit addresses
### (Unimplemented)
Standard MARIE has 8kB of memory (4096 words)

This extension allows for 128kB of memory (65536 words)

All the direct addressing instructions such as `Jump` access 'local memory'. Local memory is the 8kB of memory at which the *Program Counter* points. Indirect instructions such as `JumpI` can use the entire 128kB memory space.
### Compatibility
Most standard MARIE programs will continue to work, save for those relying upon **wrapping** indirect accesses.

Compatibility with MARIE could be preserved by checking for wrapping, and upon detection, not using extended memory.
```
/ MARIE.js: outputs 1
/ Extension:outputs 0
/ Original Java MARIE: untested
LoadI myVarAddr
Output
Halt
myVar, dec 1
myVarAddr, Load myVar / Hex 1003
```
### More on MARIE.js' memory
```
/ An unusual program
Jump FFF
```
MARIE.js triggers a runtime error when attempting to execute the `JnS 000` at `0x0FFF` from this program, because the program counter goes to `0x1000`


In my extension, the `JnS 000` instruction (`0x0000`) stored at `0x0FFF` will succeed.

The Process of executing `JnS 000` from `0x0FFF`:
1. The program counter is incremented before the instruction is decoded.
2. Program counter enters the second 8kB, pointing at 0x1000
3. JnS is decoded, recording the new Program Counter (0x1000) to the destination (0x1000)
4. and setting the Program Counter to 0x1000
### Motivation:
Enable MARIE to run more memory demanding programs, especially useful when simulating a Heap, Stack, or other large data structure.

## Double Word Instructions
### (Undesirable Draft)
Additional Instructions can be added without
breaking compatibility with MARIE programs,
by leveraging the unused `F` instruction.
Additional instructions would utilize a second word, to store the operands and such.

Here is what the instruction table would begin to look like:
```
0 // First instruction, 1 word long
1 ... E
F0 // First 2 byte instruction, 2 words long
F1 .. FE
FF // Reserved for future expansion (quadruple word anyone?)
```
All of the true MARIE instructions take the following form
```
IAAA
```
Extended instructions could take any number of shapes
```
FIIR? AAAA
F being the MARIE Reserved instruction,
II being a two-digit instruction (256 instructions!)
R being a Register ID
AAAA being a wide pointer.
```
I suggest `F0{0..A}(Register)` being versions of the MARIE instructions that operate with a different register instead of ACC, and with a full length pointer.

## Instructions modifying existing instruction's behaviour
### (Undesirable Draft)
Adding some in-machine instructions to enable and disable functionality would allow for some interesting sandboxing.
For example, a Mars program could store a MARIE program into an 8kB range, set a HALT handler, and enable Memory wrapping, before jumping to the first instruction of the MARIE program.
