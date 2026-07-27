#ifndef CELESTE_PHYSICS_H
#define CELESTE_PHYSICS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Maximum output size currently required for a named-MessagePack PlayerSnapshot. */
uint32_t celeste_snapshot_size(void);

/* Returns 1 for a supported MessagePack map or Celeste BinaryPacker .bin, otherwise 0. */
uint32_t celeste_validate_map(const uint8_t *map_ptr, uint32_t map_len);

/*
 * All three input buffers contain MessagePack bytes, except map_ptr which may also contain an
 * original Celeste .bin. inputs_len is the byte length of the serialized InputState array;
 * frames is the number of array elements to execute. Returns bytes written, or 0 on failure.
 */
uint32_t celeste_simulate(
    const uint8_t *snapshot_ptr,
    uint32_t snapshot_len,
    const uint8_t *inputs_ptr,
    uint32_t inputs_len,
    const uint8_t *map_ptr,
    uint32_t map_len,
    uint32_t frames,
    uint8_t *out_buffer,
    uint32_t out_size
);

#ifdef __cplusplus
}
#endif

#endif
