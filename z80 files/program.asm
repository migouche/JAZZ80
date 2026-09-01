    ORG 8000h
START:
    LD HL, MSG
    RST 18h
    RET
MSG:
    DB "Hi!", 0