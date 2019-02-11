# Mars
A MARIE assembler & interpreter written in [Rust](https://rust-lang.org/), with some minor assembler & runtime extensions.

I might change name to resolve conflict with
[MA.RS, Corewar in Rust.](https://gitlab.com/imp/mars/)

# Usage, or lack thereof
Mars is not ready for consumption,
as it is missing an interface,
and likely still has bugs creeping around.

# Included programs
### Multiply
[`multiply`](./multiply.mas) is a standard MARIE program written by the folks at [MARIE.js](https://marie-js.github.io/MARIE.js/?multiply).

It is licensed under the MIT license. A copy of the MIT license is available under [LICENSE-MIT](./LICENSE-MIT)

### Heap5 and 6
[`heap5`](./heap5.mas) is a heap allocator written in standard MARIE. [`heap6`](./heap6.mas) is the same allocator, but with support for extended memory.

# Extensions
Mars includes both assembly and interpreter extensions. For more details, please see [extensions.md](./extensions.md)

# TODO:
## Proper error handling
I'd like to replace most of the panics with proper error handling.
For example,
It panics when the program counter ticks past `0x0FFF` in compliant mode,
and `0xFFFF` in extended mode.
Returning something like `Err(EndOfTape)` would be preferable.

## Modulization, Librarification
Not modularization so much as dividing main.rs into three modules:
1. Shared
2. Assembler
3. Interpreter

This would shift Mars towards being a library, rather than a program.

## IO Bus
I'm thinking about writing an IO bus for MARIE. My draft lives in [bus.md](./bus.md)

## License

`multiply.mas` is licensed under the MIT license,  by the MARIE.js Team

Mars is Licensed under either of:

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.