
#include <stdio.h>
#include <string.h>
#include "secded.h"

int test_u64() {
    printf("TESTING U64: \r\n");
    const uint8_t expected[8] = {0, 0, 0, 0, 5, 0, 0, 0};
    uint8_t buffer[8];
    memcpy(buffer, expected, 8);
    const SECDED_64 secded = SECDED_64_new(57);
    SECDED_64_encode(&secded, buffer);
    buffer[7] ^= 1u << 1u;
    if (!SECDED_64_decode(&secded, buffer)) {
        printf("TESTING U64 -- FAILED: DECODE FAILED\n");
        return 1;
    }
    for (int i = 0; i < 8; i++) {
        if (expected[i] != buffer[i]) {
            printf("TESTING U64 -- FAILED: DECODE WRONG: [%d]: %d != %d\n", i, expected[i], buffer[i]);
            return 1;
        }
    }
    printf("TESTING U64 -- OK\r\n");
    return 0;
}

int test_u128() {
    printf("TESTING U128: \r\n");
    const uint8_t expected[16] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0};
    uint8_t buffer[16];
    memcpy(buffer, expected, 16);
    const SECDED_128 secded = SECDED_128_new(57);
    SECDED_128_encode(&secded, buffer);
    {  // Borrow `buffer`'s active part for test modification
        uint8_t *active_buffer = &buffer[8];
        active_buffer[7] ^= 1u << 1u;
    }
    if (!SECDED_128_decode(&secded, buffer)) {
        printf("TESTING U128 -- FAILED: DECODE FAILED\n");
        return 2;
    }
    for (int i = 0; i < 16; i++) {
        if (expected[i] != buffer[i]) {
            printf("TESTING U128 -- FAILED: DECODE WRONG: [%d]: %d != %d\n", i, expected[i], buffer[i]);
            return 2;
        }
    }
    printf("TESTING U128 -- OK\r\n");
    return 0;
}

#ifdef SECDED_FEATURES_DYN
int test_dyn() {
    const uint8_t expected[8] = {0, 0, 0, 0, 5, 0, 0, 0};
    uint8_t buffer[8];
    memcpy(buffer, expected, 8);
    printf("TESTING DYN:\r\n");
    const SECDED_DYN *secded = SECDED_DYN_new(57);
    SECDED_DYN_encode(secded, buffer, 8);
    buffer[0] ^= 128u;
    if (!SECDED_DYN_decode(secded, buffer, 8)) {
        printf("TESTING DYN -- FAILED: DECODE FAILED\n");
        return 4;
    }
    for (int i = 0; i < 8; i++) {
        if (expected[i] != buffer[i]) {
            printf("TESTING DYN -- FAILED: DECODE WRONG: [%d]: %d != %d\n", i, expected[i], buffer[i]);
            return 4;
        }
    }
    SECDED_DYN_free(secded);
    printf("TESTING DYN -- OK\r\n");
    return 0;
}
#endif

int main(int argc, char const *argv[]) {
    int status = test_u64();
    status |= test_u128();
#ifdef SECDED_FEATURES_DYN
    status |= test_dyn();
#endif
    return status;
}