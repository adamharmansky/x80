# The x80 assembler

This is a z80 assembler with namespaces and static variables, made just for my [personal needs](https://www.harmansky.xyz/work/z80), but maybe you will also find it useful.

## Running x80

To install x80, run `cargo install --path .`. To run x80, use the following command:

```
x80 output.bin input1.x80 input2.x80 input3.x80 ...
```

The inputs will just get concatenated in the order of the arguments.

## Usage

I am writing this readme 7 months after I had written the last line of this code, so don't expect it to be complete. Read the [example code at my website](https://www.harmansky.xyz/work/z80/basic_code.zip) instead.

The instruction syntax is the normal z80 syntax. You can use any indentation.
Each instruction uses exactly one line. Anything starting with `@` and no space is a macro or a tag, anything starting with `@` and a space after it is a comment

```
start:
  ld a, 80        @ loads a with decimal 80
  ld a, 0x80      @ loads a with hex 80
  ld a, 00000101b @ loads a with binary 101 (5)
  jp @start       @ jumps to tag start
```

### Defining tags and scopes

 * `tag:` creates a tag at program counter
 * `0x0100:` sets the program counter to a given position and pads the space with `0x00`
 * `@define tag 0x100` creates a tag with a given value

 * `@mod name` creates a namespace
 * `@function` creates a namespace and a tag
 * `@scope` creates an anonymous namespace
 * `@end` ends any namespace or function

 * `@byte 0xff` inserts a byte at the program counter
 * `@string {hello world}` inserts a string, strings are enclosed in `{}`, escaping with `/` works

 * things enclosed in `{}` are expressions, except for `@string`

 * to reference a tag in the current scope, use `@tag`
 * to reference a tag in the parent scope, use `@@tag`
 * to reference a tag in the main scope, use `scope__tag`, `__` is like `::` in C++

example:

```
@ the code will start at address 0x100, padded with 256 zeroes
0x0100:

@mod main
  @function main
    @define text_offset 7 @ defines a constant
    ld hl, {@text+@text_offset} @ example of an expression
    call io__display__puts
    ret

    text:
      @string  {hello, world!}
      @byte 0
  @end
@end

@mod io
  @mod display
    @define buffer 0x8000
    @function puts
      loop:
        @scope
        loop:
          @ this tag is also called loop but it's in a different scope...
          jp nz, @loop
        @end
        @ ...some more code...
        jp nz, loop
      ret
    @end
    @function putchar
      ld (@@buffer), a @ parent scope
      ret
    @end
  @end
@end
```

### Static variables

 * `@alloc name n` allocates a static variable n-bytes long
 * `@alloc_from address` sets the address from which to allocate static variables

for example:

```
@alloc_from 0x8000 @ let's say that our RAM starts here

@alloc buffer 0x100 @ allocate 256 bytes in RAM at 0x8000
@alloc variable 4 @ allocate 4 bytes in the next address (0x8100)

@scope
  @alloc another_variable 1 @ this variable will reside at address 0x8104
@end
```
