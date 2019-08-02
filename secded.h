#ifndef SECDED_H
#define SECDED_H
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct SECDED {
    uint8_t encodable_size;
    uint8_t code_size;
    __uint128_t encode_matrix[8];
    __uint128_t decode_matrix[8];
    uint16_t syndromes[128];
} SECDED;

SECDED SECDED_new(size_t encodable_bits);

void SECDED_encode(SECDED *secded, uint8_t *data, size_t size);

bool SECDED_decode(SECDED *secded, uint8_t *data, size_t size);

#endif