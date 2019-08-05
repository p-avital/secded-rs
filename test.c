#include "secded.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char const *argv[])
{
    uint8_t expected[8] = {0,0,0,0,5,0,0,0};
    uint8_t buffer[8];
    memcpy(buffer, expected, 8);
    SECDED_64 secded = SECDED_64_new(57);
    SECDED_64_encode(&secded, buffer);
    buffer[7] ^= 1<<1;
    if (!SECDED_64_decode(&secded, buffer)) {
        printf("PANIC: DECODE FAILED\n");
        return 1;
    }
    for (int i = 0; i < 7; i++) {
        if (expected[i] != buffer[i]) {
            printf("PANIC: DECODE WRONG: [%d]: %d != %d\n", i, expected[i], buffer[i]);
            return 1;
        }
        printf("[%d]: %d == %d\n", i, expected[i], buffer[i]);
    }
    return 0;
}
