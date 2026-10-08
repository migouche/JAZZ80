# How to use the example Operating System

The example OS is a simple interactive monitor that runs on the JAZZ80 simulator. It starts at address `0x0000`, prints a prompt, reads a command line, and supports a few basic commands. It is implemented in `os.asm` in the `z80 files/` directory.

This document will work on the web version of JAZZ80, since it comes with the example files bundled. If you are running the native version, you will need to download the example files from the [z80 files/](./z80%20files) directory and load them into the simulator.

## Basic Setup

### Load the devices

The OS requires 2 devices: the virtual terminal (input/output) and the virtual filesystem (file storage). Add them via `Devices > Add Virtual Terminal` and `Devices > Add Virtual Filesystem`. Leave the default ports. The OS will not work without these devices.

### Load the OS program

Open `os.asm` via `examples > os.asm` (web version) or `File > Open` (native version). It will be assembled and loaded at address `0x0000`. Hit **Run** to start the OS. It prints a prompt and waits for input — type commands and hit **Enter** to execute them.

### Commands

| Command | Description |
| --- | --- |
| `HELP` | List all available commands. |
| `CLEAR` | Clears the terminal output screen. |
| `LS` | List files and directories in the current directory. |
| `CD <dir>` | Change the current working directory. Supports `..` to go up and `/` for root. |
| `MKDIR <dir>` | Create a directory inside the current directory. |
| `TYPE <file>` | Print a file's contents to the terminal. |
| `RUN <file>` | Load a file at `0x8000` and execute it as a program. |

Commands are case-sensitive (uppercase). Arguments are separated by a single space. Unknown commands get `Unknown command`.

### Using the terminal

The Virtual Terminal window has two sections:

- **OS Output** — everything the OS prints: banner, command results, file contents.
- **Send Input to OS** — type commands here. Press **Enter** or click **Send** to submit.

Backspace is supported during input. Maximum command length is 64 characters — the OS auto-submits when the limit is reached.

### Using the filesystem

The DOS Controller window has three controls:

- **Up** — go to the parent directory.
- **Upload file...** — upload a file from your computer into the virtual filesystem.
- **Create folder** — create a new directory (same as `MKDIR`, but through the GUI).

The filesystem is in-memory only — not connected to your computer's actual filesystem. Uploaded files are lost when you reset or reload.

The filesystem starts empty except for the root directory `/`. Create directories and upload files through the DOS Controller window or the `MKDIR` and `TYPE`/`RUN` commands.

## How the OS works internally

### Memory map

| Address range | Purpose |
| --- | --- |
| `0x0000` | OS entry point (`JP OS_BOOT`) |
| `0x0008` | `RST 08h` — `PRINT_CHAR` routine |
| `0x0010` | `RST 10h` — `WAIT_CHAR` routine |
| `0x0018` | `RST 18h` — `PRINT_STRING` routine |
| `0x4000` – `0x403F` | Command input buffer (64 bytes) |
| `0x8000` | Program load address for `RUN` command |

### Register bank usage

The OS reserves the **shadow register bank** (`AF'`, `BC'`, `DE'`, `HL'`) for itself. It switches to the shadow bank at boot with `EX AF,AF'` and `EXX`, leaving the primary set free for user programs.

When `RUN` executes, the OS:

1. Switches back to the primary bank (`EX AF,AF'`, `EXX`)
2. Calls the program at `0x8000`
3. When the program returns (`RET`), switches back to the shadow bank

User programs launched with `RUN` should treat `AF'`, `BC'`, `DE'`, and `HL'` as reserved. Use the primary bank registers (`AF`, `BC`, `DE`, `HL`, `IX`, `IY`) freely.

### I/O ports

**Virtual Terminal** (base port `0x00`):

| Port | Direction | Purpose |
| --- | --- | --- |
| `0x00` | Read | Next character typed by the user (blocks until available). |
| `0x00` | Write | Send a character to the terminal display. |
| `0x01` | Read | Status: bit 1 = TX ready (always set), bit 0 = RX data available. |

**DOS Controller** (base port `0x20`):

| Port | Direction | Purpose |
| --- | --- | --- |
| `0x20` | Write | Send a command argument byte (part of a filename or directory name). |
| `0x20` | Read | Next output byte (file contents or directory listing). |
| `0x21` | Write | Execute a command (see below). |
| `0x22` | Read | Status: `0` = OK/idle, `1` = error, `2` = data ready. |

**DOS commands** (written to port `0x21`):

| Value | Command | Description |
| --- | --- | --- |
| `0` | CLEAR | Clears the argument buffer. Send before a new argument. |
| `1` | MKDIR | Create a directory using the argument buffer as the name. |
| `2` | CD | Change directory using the argument buffer as the target. |
| `3` | LS | List the current directory contents. |
| `4` | READ | Reads a file using the argument buffer as the filename. Used by `TYPE` and `RUN`. |

### RST vectors

Three routines are placed at Z80 restart addresses, callable via `RST` from user programs:

| Address | Instruction | Routine | Usage |
| --- | --- | --- | --- |
| `0x0008` | `RST 08h` | `PRINT_CHAR` | Print the character in `A` to the terminal. |
| `0x0010` | `RST 10h` | `WAIT_CHAR` | Wait for a keypress and return it in `A`. |
| `0x0018` | `RST 18h` | `PRINT_STRING` | Print a null-terminated string pointed to by `HL`. |

### How strings work

Strings are plain byte sequences terminated by a `0` (null) byte. The routines read bytes until they encounter `0`.

```asm
MSG_HELLO:
    DB "Hello, world!", 0Dh, 0Ah, 0
```

`0Dh` (carriage return, `\r`) returns the cursor to the start of the line. `0Ah` (line feed, `\n`) moves it down one row. Together they produce a newline. The final `0` is the terminator.

**`RST 18h` (PRINT_STRING)** reads bytes from `HL` one at a time, calling `PRINT_CHAR` for each. It stops at `0`:

```asm
PRINT_STRING:
.NEXT:
    LD A, (HL)    ; load byte at HL
    OR A          ; set Z flag if A == 0
    JR Z, .DONE   ; terminator found — return
    CALL PRINT_CHAR
    INC HL        ; advance to next byte
    JR .NEXT
.DONE:
    RET
```

Point `HL` at the string and call `RST 18h`:

```asm
LD HL, MSG_HELLO
RST 18h
```

**`RST 08h` (PRINT_CHAR)** prints a single character from `A`:

```asm
LD A, 'H'
RST 08h
```

**`RST 10h` (WAIT_CHAR)** waits for a keypress and returns it in `A`. It polls the terminal status port until input is available. `READ_LINE` uses this to build the command string byte by byte.

## Writing and running your own programs

### Creating a program

Write your program in the built-in editor. It should:

1. Start at `0x8000` (`ORG 8000h`)
2. Use the primary register bank (the OS handles the bank switch)
3. Return to the OS with `RET` when finished
4. Use `RST 08h`, `RST 10h`, and `RST 18h` for terminal I/O

Minimal example:

```asm
    ORG 8000h
START:
    LD HL, MSG
    RST 18h         ; Print string
    RET             ; Return to OS

MSG:
    DB "Hello from my program!", 0Dh, 0Ah, 0
```

### Running a program

1. Upload your program file through the DOS Controller window (click **Upload file...**).
2. At the OS prompt, type `RUN yourfile.bin` and press **Enter**.
3. The OS loads the file at `0x8000` and calls it.
4. When your program executes `RET`, control returns to the OS.

### The `program.asm` example

The bundled `program.asm` is ready to run:

```asm
    ORG 8000h
START:
    LD HL, MSG
    RST 18h
    RET
MSG:
    DB "Hi!", 0
```



1. Upload `program.bin` through the DOS Controller window. (On the web version, example files are already bundled — no download needed.)
2. Type `RUN program.bin` at the OS prompt.
3. `Hi!` appears at the terminal.

### Tips for user programs

- **Stack**: The OS sets `SP` to `0xFFFF` at boot. The stack grows downward from the top of memory.
- **Memory**: Your program loads at `0x8000`. The OS uses `0x0000`–`0x403F`. Avoid writing below `0x8000`.
- **Interrupts**: The OS does not use them. If your program enables interrupts, save and restore the interrupt state before returning.
- **Shadow registers**: Do not modify `AF'`, `BC'`, `DE'`, or `HL'` — the OS uses them for its state. Modifying them may crash the OS after your program returns.

## Troubleshooting

| Problem | Cause | Solution |
| --- | --- | --- |
| OS does not respond to input | Virtual Terminal not added | Add the Virtual Terminal device via `Devices > Add Virtual Terminal`. |
| `Unknown command` | Command not recognized or wrong case | Check spelling and use uppercase. |
| `Directory not found` | `CD` target does not exist | Use `LS` to see available directories. |
| `File not found` | `TYPE` or `RUN` target does not exist | Use `LS` to see available files. |
| Program crashes after `RUN` | Program corrupted OS state | Make sure your program ends with `RET` and does not modify shadow registers. |
| Terminal shows gibberish | Non-printable characters | The terminal only displays printable ASCII. Non-printable bytes are shown as `.`. |
