; Minimal monitor for the JAZZ80 virtual terminal.
; Design note: the OS reserves the shadow register bank (AF', BC', DE', HL') for
; its own work. User programs are launched in the primary bank, so they should
; not depend on the alternate register set being available.

TERM_DATA      EQU 00h
TERM_STATUS    EQU 01h
FS_DATA        EQU 20h
FS_CMD         EQU 21h
FS_STATUS      EQU 22h
CMD_BUFFER     EQU 4000h
CMD_BUFFER_END EQU 4040h
PROG_LOAD_ADDR EQU 8000h

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
    ; Switch into the monitor's bank so the main register set is left free for
    ; user programs. The shell and filesystem code keep their own state here.
    EX AF, AF'
    EXX
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

    LD IX, CMD_TABLE

.NEXT:
    LD E, (IX+0)
    LD D, (IX+1)

    LD A, D
    OR E
    JP Z, .UNKNOWN

    PUSH HL
    LD A, (IX+4)
    OR A
    JP Z, .COMPARE_EXACT
    CALL STR_PREFIX
    JP .COMPARE_RESULT
.COMPARE_EXACT:
    CALL STR_EQ
.COMPARE_RESULT:
    JP Z, .FOUND
    POP HL

    INC IX
    INC IX
    INC IX
    INC IX
    INC IX
    INC IX
    JP .NEXT

.FOUND:
    POP DE
    LD E, (IX+2)
    LD D, (IX+3)
    PUSH DE
    RET

.UNKNOWN:
    LD HL, MSG_UNKNOWN
    CALL PRINT_STRING
    RET

STR_EQ:
.LOOP:
    LD A, (DE)
    CP (HL)
    JP NZ, .NO_MATCH
    OR A
    JP Z, .MATCH
    INC HL
    INC DE
    JP .LOOP

.NO_MATCH:
    RET

.MATCH:
    RET

STR_PREFIX:
.LOOP:
    LD A, (DE)
    OR A
    RET Z
    CP (HL)
    JP NZ, .NO_MATCH
    INC HL
    INC DE
    JP .LOOP

.NO_MATCH:
    RET

CMD_TABLE:
    DW CMD_HELP_STR, CMD_HELP, 0000h
    DW CMD_CLEAR_STR, CMD_CLEAR, 0000h
    DW CMD_LS_STR, CMD_LS, 0000h
    DW CMD_CD_STR, CMD_CD, 0001h
    DW CMD_MKDIR_STR, CMD_MKDIR, 0001h
    DW CMD_RUN_STR, CMD_RUN, 0001h
    DW CMD_TYPE_STR, CMD_TYPE, 0001h
    DW 0000h, 0000h, 0000h

CMD_HELP_STR:
    DB "HELP", 0
CMD_CLEAR_STR:
    DB "CLEAR", 0
CMD_LS_STR:
    DB "LS", 0
CMD_CD_STR:
    DB "CD ", 0
CMD_MKDIR_STR:
    DB "MKDIR ", 0
CMD_RUN_STR:
    DB "RUN ", 0
CMD_TYPE_STR:
    DB "TYPE ", 0

CMD_HELP:
    LD HL, MSG_HELP
    CALL PRINT_STRING
    RET

CMD_CLEAR:
    LD HL, MSG_CLEAR
    CALL PRINT_STRING
    RET

CMD_LS:
    LD A, 03h
    OUT (FS_CMD), A
.LS_PUMP:
    IN A, (FS_STATUS)
    CP 02h
    RET NZ
    IN A, (FS_DATA)
    CALL PRINT_CHAR
    JR .LS_PUMP

CMD_CD:
    CALL SEND_FS_ARGS
    LD A, 02h
    OUT (FS_CMD), A
    IN A, (FS_STATUS)
    CP 01h
    RET NZ
    LD HL, MSG_DIR_ERR
    CALL PRINT_STRING
    RET

CMD_MKDIR:
    CALL SEND_FS_ARGS
    LD A, 01h
    OUT (FS_CMD), A
    RET

CMD_RUN:
    CALL SEND_FS_ARGS
    LD A, 04h
    OUT (FS_CMD), A
    IN A, (FS_STATUS)
    CP 01h
    JP Z, .FILE_ERR
    CP 02h
    RET NZ
    LD HL, PROG_LOAD_ADDR
.LOAD_LOOP:
    IN A, (FS_STATUS)
    CP 02h
    JR NZ, .EXEC
    IN A, (FS_DATA)
    LD (HL), A
    INC HL
    JR .LOAD_LOOP
.EXEC:
    ; The monitor keeps its own state in the shadow bank. When a program starts,
    ; make the main bank active for the user program, then restore the OS bank on
    ; return. User code should treat AF'/BC'/DE'/HL' as reserved by the OS.
    EX AF, AF'
    EXX
    CALL PROG_LOAD_ADDR
    EX AF, AF'
    EXX
    RET
.FILE_ERR:
    LD HL, MSG_FILE_ERR
    CALL PRINT_STRING
    RET

CMD_TYPE:
    CALL SEND_FS_ARGS
    LD A, 04h
    OUT (FS_CMD), A
.TYPE_LOOP:
    IN A, (FS_STATUS)
    CP 02h
    RET NZ
    IN A, (FS_DATA)
    CALL PRINT_CHAR
    JR .TYPE_LOOP

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

MSG_BANNER:
	DB "JAZZ80 MONITOR", 0Dh, 0Ah, 0
MSG_PROMPT:
    DB "> ", 0
MSG_NEWLINE:
    DB 0Dh, 0Ah, 0
MSG_HELP:
    DB "HELP - show commands", 0Dh, 0Ah
    DB "CLEAR - clear the terminal", 0Dh, 0Ah
    DB "CD <dir> - change directory", 0Dh, 0Ah
    DB "LS - list directory", 0Dh, 0Ah
    DB "MKDIR <dir> - create directory", 0Dh, 0Ah
    DB "TYPE <file> - show file contents", 0Dh, 0Ah
    DB "RUN <file> - execute program", 0Dh, 0Ah, 0
MSG_CLEAR:
    DB 1Bh, "[2J", 1Bh, "[H", 0
MSG_UNKNOWN:
    DB "Unknown command", 0Dh, 0Ah, 0
MSG_DIR_ERR:
    DB "Directory not found", 0Dh, 0Ah, 0
MSG_FILE_ERR:
    DB "File not found", 0Dh, 0Ah, 0