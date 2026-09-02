# Z80 Example Programs

This directory contains small Z80 programs for exploring JAZZ80's CPU, interrupt, and device simulation. The examples are listed in [`examples.json`](./examples.json), and the source files below are the ones included with the application. [`examples.json`](./examples.json) exists for the web build to bundle the examples so that they are available without loading anything of your own first.

## Before you run an example

1. Open JAZZ80 and load the `.asm` file from this directory.
2. Add the devices listed for that example from the device menu.
3. Keep the device ports at their defaults unless the example says otherwise.
4. Run the program, or use single-step mode when inspecting an interrupt handler.

The simulator treats I/O ports as 8-bit values for these devices. A device configured at port `0x02`, for example, responds to `OUT (02h), A`.

## Examples

### `fib.asm` - Fibonacci sequence

Calculates the Fibonacci number for the value stored in `N` and writes the 16-bit result to `RESULT`. `N` is initially `16` (`10h`), so the program stores `F(16)` in memory before halting.

This is a CPU and memory example. It does not use I/O, interrupts, or external devices.

To try another input, change the byte at `N` and run the program again. The result is stored as a little-endian word by `LD (RESULT), HL`.

### `im0.asm` - Interrupt Mode 0

Waits in a `HALT` loop with interrupt mode 0 enabled. The interrupting device supplies an opcode to the CPU:

- Vector/opcode `FF` executes the handler at `0x0038`, which writes `3` to the display.
- Vector/opcode `EF` executes the handler at `0x0028`, which writes `2` to the display.

Add these devices:

| Device | Port | Purpose |
| --- | --- | --- |
| Interrupt Controller | `0x00` | Generates the interrupt and supplies `FF` or `EF`. |
| Display | `0x02` | Shows `2` or `3` from the selected handler. |

The Interrupt Controller defaults to vector/opcode `FF`. To exercise the `EF` branch, edit its vector field to `EF` before pressing **TRIGGER INTERRUPT**.

### `im1.asm` - Interrupt Mode 1

Waits for an interrupt in mode 1. The Z80 always enters the restart address `0x0038` in this mode, regardless of the vector byte supplied by the device. The handler reads the last key from the keypad and sends that value to the display.

Add these devices at their defaults:

| Device | Port | Purpose |
| --- | --- | --- |
| Keypad | `0x01` | Supplies a key code and raises an interrupt when a key is pressed. |
| Display | `0x02` | Shows the key code read by the handler. |

Press a key in the Keypad window. The program wakes from `HALT`, reads the key, updates the display, and returns to the wait loop.

### `im2.asm` - Interrupt Mode 2

Sets the `I` register to `0x10` and uses a vector table beginning at `0x1000`. The handlers at `0x3000`, `0x4000`, and `0x5000` write `1`, `2`, and `3` to the display respectively.

Add these devices:

| Device | Port | Purpose |
| --- | --- | --- |
| Interrupt Controller | `0x00` | Generates the interrupt and supplies the vector byte. |
| Display | `0x02` | Shows the value selected by the vector table. |

The Interrupt Controller must be configured with vector/opcode `00` for this example. With `I = 10h`, the CPU uses the vector table entry beginning at `0x1000`; the `00` vector selects the first entry and reaches `0x3000`, so the display shows `1`. The other table entries are included to make the layout and alternate vector targets visible.

### `nmi.asm` - Non-maskable interrupt

Waits in a `HALT` loop until a non-maskable interrupt is triggered. The handler at `0x0066` increments `A` and writes the counter to port `0x01` before returning with `RETN`.

Add an **NMI Trigger** and a **Display**. The NMI Trigger does not use an I/O port. Because this example writes to `0x01`, configure the Display manually at port `0x01` instead of its default `0x02`. Trigger the NMI repeatedly to watch the counter advance.

### `os.asm` - JAZZ80 monitor

Provides a small interactive monitor backed by JAZZ80's virtual terminal and virtual filesystem. It starts at `0x0000`, prints a prompt, reads a command line, and supports:

| Command | Action |
| --- | --- |
| `HELP` | Lists the available commands. |
| `CLEAR` | Clears the terminal output. |
| `LS` | Lists files and directories in the current directory. |
| `CD <dir>` | Changes the current directory. |
| `MKDIR <dir>` | Creates a directory. |
| `TYPE <file>` | Prints a file's contents. |
| `RUN <file>` | Loads a file at `0x8000` and calls it as a program. |

Add these devices at their defaults:

| Device | Ports | Purpose |
| --- | --- | --- |
| Virtual Terminal | `0x00-0x01` | Port `0x00` carries characters; port `0x01` reports transmit readiness and received input. |
| DOS Controller | `0x20-0x22` | Port `0x20` carries data, `0x21` selects commands, and `0x22` reports status. |

Type commands into the Virtual Terminal window. Files can be created or uploaded through the DOS Controller window. The monitor keeps its own state in the alternate register bank, so programs launched with `RUN` should treat `AF'`, `BC'`, `DE'`, and `HL'` as reserved.

## Device reference

### Keypad

The on-screen hexadecimal keypad returns a one-byte key code when the CPU reads its configured port. Keys `0`-`9` return their numeric values; `A`-`F` return `0x0A`-`0x0F`. Pressing a key also raises a maskable interrupt. Reading the port clears the pending interrupt.

Default port: `0x01`.

### Display

The seven-segment Display accepts a byte written to its configured port. Its window shows the value in hexadecimal and binary. It is output-only.

Default port: `0x02`.

### LCD Display

The LCD Display appends printable bytes written to its configured port to a text buffer. Reading from the same port returns bytes from its built-in input buffer (`Hello Input`), then returns zero when the buffer is exhausted.

Default port: `0x03`.

### Interrupt Controller

The Interrupt Controller is a manually triggered maskable interrupt source. Its vector/opcode field is used by interrupt mode 0 and as the vector byte in mode 2. It does not respond to normal `IN` or `OUT` instructions.

Default port: `0x00`; default vector/opcode: `0xFF`.

### Virtual Terminal

The Virtual Terminal provides a simple two-port serial-style interface. Its base port carries output and input bytes. The following port reports status: bit 1 is always set to indicate that transmission is ready, and bit 0 is set when input is waiting. Text entered in the window is queued for the CPU and is terminated with carriage return. The terminal also understands the clear-screen sequence used by the monitor.

Default base port: `0x00`; status port: `0x01`.

### DOS Controller

The DOS Controller is an in-memory filesystem, not access to the host computer's filesystem. Its data port receives command arguments or returns command output. Its command port executes an operation, and its status port reports `0` for success or idle, `1` for an error, and `2` when output data is ready.

Default ports:

| Port | Role |
| --- | --- |
| `0x20` | Data and command-argument bytes |
| `0x21` | Command register |
| `0x22` | Status register |

The command values used by `os.asm` are `0` to clear the argument buffer, `1` for `MKDIR`, `2` for `CD`, `3` for `LS`, and `4` for file reads used by `TYPE` and `RUN`. The device window lets you browse directories, create folders, and upload files.

### NMI Trigger

The NMI Trigger has no I/O port and does not need to be connected to an `IN` or `OUT` instruction. Use its control in the application to assert a non-maskable interrupt. The CPU enters the fixed NMI handler address `0x0066`; the handler returns with `RETN`.

## Port conflicts

Devices share the same I/O space. Two devices should not be assigned overlapping ports unless you are deliberately testing bus behavior. The default ports are compatible for the examples in this directory. The exception is `nmi.asm`, which intentionally expects a Display at `0x01`, so change that Display from its default port.
