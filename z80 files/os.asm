; Minimal monitor for the JAZZ80 virtual terminal.

TERM_DATA      EQU 00h
TERM_STATUS    EQU 01h
FS_DATA        EQU 20h
FS_CMD         EQU 21h
FS_STATUS      EQU 22h
CMD_BUFFER     EQU 4000h
CMD_BUFFER_END EQU 4040h

ORG 0000h
    JP OS_BOOT

ORG 0008h
    JP PRINT_CHAR

ORG 0010h
    JP WAIT_CHAR

ORG 0018h
    JP PRINT_STRING

OS_BOOT:
    LD SP, 0FFFFh
    LD HL, MSG_BANNER
    CALL PRINT_STRING

SHELL_LOOP:
    LD HL, MSG_PROMPT
    CALL PRINT_STRING
    CALL READ_LINE
    CALL EXECUTE_COMMAND
    JR SHELL_LOOP

WAIT_CHAR:
.POLL_RX:
    IN A, (TERM_STATUS)
    BIT 0, A
    JR Z, .POLL_RX
    IN A, (TERM_DATA)
    RET

PRINT_CHAR:
    PUSH AF
.POLL_TX:
    IN A, (TERM_STATUS)
    BIT 1, A
    JR Z, .POLL_TX
    POP AF
    OUT (TERM_DATA), A
    RET

PRINT_STRING:
.NEXT:
    LD A, (HL)
    OR A
    JR Z, .DONE
    CALL PRINT_CHAR
    INC HL
    JR .NEXT
.DONE:
    RET

READ_LINE:
    LD HL, CMD_BUFFER
    LD B, 40h

.READ:
    CALL WAIT_CHAR
    CP 0Dh
    JR Z, .END
    CP 08h
    JR Z, .BACKSPACE

    LD (HL), A
    INC HL
    CALL PRINT_CHAR
    DEC B
    JR Z, .FULL
    JR .READ

.BACKSPACE:
    LD A, B
    CP 40h
    JR Z, .READ
    DEC HL
    INC B
    LD A, 08h
    CALL PRINT_CHAR
    LD A, 20h
    CALL PRINT_CHAR
    LD A, 08h
    CALL PRINT_CHAR
    JR .READ

.FULL:
    LD (HL), 00h
    LD HL, MSG_NEWLINE
    CALL PRINT_STRING
    RET

.END:
    LD (HL), 00h
    LD HL, MSG_NEWLINE
    CALL PRINT_STRING
    RET

EXECUTE_COMMAND:
    LD HL, CMD_BUFFER
    LD A, (HL)
    OR A
    RET Z

    CP 48h
    JR Z, .CHECK_HELP
    CP 43h
    JR Z, .CHECK_C_COMMANDS
    CP 4Ch
    JR Z, .CHECK_LS
    CP 4Dh
    JR Z, .CHECK_MKDIR
    JP .UNKNOWN

.CHECK_C_COMMANDS:
    INC HL
    LD A, (HL)
    CP 4Ch
    JP Z, .CHECK_CLEAR
    CP 44h
    JR Z, .CHECK_CD
    JP .UNKNOWN

.CHECK_HELP:
    INC HL
    LD A, (HL)
    CP 45h
    JP NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 4Ch
    JP NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 50h
    JP NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    OR A
    JP NZ, .UNKNOWN
    LD HL, MSG_HELP
    CALL PRINT_STRING
    RET

.CHECK_LS:
    INC HL
    LD A, (HL)
    CP 53h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    OR A
    JR NZ, .UNKNOWN

    LD A, 03h
    OUT (FS_CMD), A
.LS_PUMP:
    IN A, (FS_STATUS)
    CP 02h
    RET NZ
    IN A, (FS_DATA)
    CALL PRINT_CHAR
    JR .LS_PUMP

.CHECK_CD:
    INC HL
    LD A, (HL)
    CP 20h
    JR NZ, .UNKNOWN
    INC HL
    CALL SEND_FS_ARGS
    LD A, 02h
    OUT (FS_CMD), A
    IN A, (FS_STATUS)
    CP 01h
    RET NZ
    LD HL, MSG_DIR_ERR
    CALL PRINT_STRING
    RET

.CHECK_MKDIR:
    INC HL
    LD A, (HL)
    CP 4Bh
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 44h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 49h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 52h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 20h
    JR NZ, .UNKNOWN
    INC HL
    CALL SEND_FS_ARGS
    LD A, 01h
    OUT (FS_CMD), A
    RET

.CHECK_CLEAR:
    INC HL
    LD A, (HL)
    CP 4Ch
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 45h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 41h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    CP 52h
    JR NZ, .UNKNOWN
    INC HL
    LD A, (HL)
    OR A
    JR NZ, .UNKNOWN
    LD HL, MSG_CLEAR
    CALL PRINT_STRING
    RET

.UNKNOWN:
    LD HL, MSG_UNKNOWN
    CALL PRINT_STRING
    RET

SEND_FS_ARGS:
    LD A, 00h
    OUT (FS_CMD), A
.SEND_FS_ARGS_LOOP:
    LD A, (HL)
    OR A
    RET Z
    OUT (FS_DATA), A
    INC HL
    JR .SEND_FS_ARGS_LOOP

MSG_BANNER: DB "JAZZ80 MONITOR", 0Dh, 0Ah, 0
MSG_PROMPT:
    DB "> ", 0
MSG_NEWLINE:
    DB 0Dh, 0Ah, 0
MSG_HELP:
    DB "HELP - show commands", 0Dh, 0Ah
    DB "CLEAR - clear the terminal", 0Dh, 0Ah
    DB "CD <dir> - change directory", 0Dh, 0Ah
    DB "LS - list directory", 0Dh, 0Ah
    DB "MKDIR <dir> - create directory", 0Dh, 0Ah, 0
MSG_CLEAR:
    DB 1Bh, "[2J", 1Bh, "[H", 0
MSG_UNKNOWN:
    DB "Unknown command", 0Dh, 0Ah, 0
MSG_DIR_ERR:
    DB "Directory not found", 0Dh, 0Ah, 0