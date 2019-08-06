#ifndef SECDED_H
#define SECDED_H
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct SECDED_128 {
    uint8_t encodable_size;
    uint8_t code_size;
    uint64_t encode_matrix[14];
    uint64_t decode_matrix[14];
    uint16_t syndromes[128];
} SECDED_128;

SECDED_128 SECDED_128_new(size_t encodable_bits);

void SECDED_128_encode(SECDED_128 *secded, uint8_t data[16]);

bool SECDED_128_decode(SECDED_128 *secded, uint8_t data[16]);

typedef struct SECDED_64 {
    uint8_t encodable_size;
    uint8_t code_size;
    uint64_t encode_matrix[7];
    uint64_t decode_matrix[7];
    uint16_t syndromes[64];
} SECDED_64;

SECDED_64 SECDED_64_new(size_t encodable_bits);

void SECDED_64_encode(SECDED_64 *secded, uint8_t data[8]);

bool SECDED_64_decode(SECDED_64 *secded, uint8_t data[8]);

#ifdef SECDED_FEATURES_DYN
typedef struct SECDED_DYN {} SECDED_DYN;

SECDED_DYN *SECDED_DYN_new(size_t encodable_bits);

SECDED_DYN SECDED_DYN_free(SECDED_DYN *secded);

void SECDED_DYN_encode(SECDED_DYN *secded, uint8_t *data, size_t size);

bool SECDED_DYN_decode(SECDED_DYN *secded, uint8_t *data, size_t size);

#endif
#endif