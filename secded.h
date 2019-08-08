#ifndef SECDED_H
#define SECDED_H
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct SECDED_64 {
    uint8_t encodable_size;
    uint8_t code_size;
    uint8_t mask;
    uint64_t coding_matrix[6];
    uint16_t syndromes[64];
} SECDED_64;

//  Returns a `SecDed64` Codec, which you can use through `SECDED_64_encode(...)` and `SECDED_64_decode(...)`
//  Since `SecDed64`'s drop is trivial, the value is returned and no `free` call is required
SECDED_64 SECDED_64_new(size_t encodable_bits);

//  A wrapper to `secded.encode(&data[..8])`
void SECDED_64_encode(const SECDED_64 *secded, uint8_t data[8]);

//  A wrapper to `secded.decode(&data[..8])`
bool SECDED_64_decode(const SECDED_64 *secded, uint8_t data[8]);

typedef struct SECDED_128 {
    uint8_t encodable_size;
    uint8_t code_size;
    uint8_t mask;
    uint64_t correction_matrix[14];
    uint16_t syndromes[128];
} SECDED_128;

//  Returns a `SecDed128` Codec, which you can use through `SECDED_128_encode(...)` and `SECDED_128_decode(...)`
//  Since `SecDed128`'s drop is trivial, the value is returned and no `free` call is required
SECDED_128 SECDED_128_new(size_t encodable_bits);

//  A wrapper to `secded.encode(&data[..16])`
void SECDED_128_encode(const SECDED_128 *secded, uint8_t data[16]);

//  A wrapper to `secded.decode(&data[..16])`
bool SECDED_128_decode(const SECDED_128 *secded, uint8_t data[16]);

#ifdef SECDED_FEATURES_DYN
//  Since `SecDedDynamic` can't be easily represented in C, and has a specialized `free` operation,
//  you can only manipulate it as a pointer.
//  This 0-sized type is only here for type-safety.
typedef struct SECDED_DYN {
} SECDED_DYN;

//  Returns a pointer to a `SecDedDynamic` Codec instance, which you can use through `SECDED_DYNAMIC_encode(...)` and
//  `SECDED_DYNAMIC_decode(...)`.
//  Dropping SECDED_DYN is non-trivial and should be done using `SECDED_DYN_free()`
const SECDED_DYN *SECDED_DYN_new(size_t encodable_bits);

//  De-allocates `secded`'s internals and secded itself.
void SECDED_DYN_free(const SECDED_DYN *secded);

//  A wrapper to `secded.encode(&data[..size])`
void SECDED_DYN_encode(const SECDED_DYN *secded, uint8_t *data, size_t size);

//  A wrapper to `secded.decode(&data[..size])`
bool SECDED_DYN_decode(const SECDED_DYN *secded, uint8_t *data, size_t size);

#endif
#endif