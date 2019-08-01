#include "secded.h"
#include <stdio.h>

int main(int argc, char const *argv[])
{
    uint8_t expected[8] = {0,0,0,0,0,0,0,5};
    uint8_t buffer[8];
    struct SECDED* secded = SECDED_new(57);
    SECDED_encode(secded, expected, 8, buffer);
    buffer[7] ^= 1<<1;
    if (!SECDED_decode(secded, buffer, 8, buffer)) {
        printf("PANIC: DECODE FAILED\n");
        return 1;
    }
    for (int i = 0; i < 8; i++) {
        if (expected[i] != buffer[i]) {
            printf("PANIC: DECODE WRONG: [%d]: %d != %d\n", i, expected[i], buffer[i]);
            return 1;
        }
        printf("[%d]: %d == %d\n", i, expected[i], buffer[i]);
    }
    SECDED_free(secded);
    return 0;
}
