#ifndef SECDED_H
#define SECDED_H
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct SECDED SECDED;

SECDED* SECDED_new(size_t encodable_bits);
void SECDED_free(SECDED* secded);

void SECDED_encode(SECDED *secded, const uint8_t *data, size_t size, uint8_t *out_buffer);

bool SECDED_decode(SECDED *secded, const uint8_t *data, size_t size, uint8_t *out_buffer);

#endif