#ifndef CELESTE_PHYSICS_H
#define CELESTE_PHYSICS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Fixed-layout hot-path values. Input flags: jump pressed/held, dash pressed,
 * crouch-dash pressed, grab held (bits 0..4). State flags: on-ground,
 * player-on-ground, ducking, can-dream-dash, dead, death-freeze-pending,
 * red-booster (bits 0..6). */
typedef struct CelesteInputPod {
    int8_t move_x;
    int8_t move_y;
    uint8_t flags;
} CelesteInputPod;

typedef struct CelestePlayerPod {
    float pos_x;
    float pos_y;
    float speed_x;
    float speed_y;
    float dash_dir_x;
    float dash_dir_y;
    float boost_target_x;
    float boost_target_y;
    float last_aim_x;
    float last_aim_y;
    float stamina;
    float state_timer;
    uint8_t dashes;
    uint8_t state;
    uint8_t facing;
    uint8_t flags;
    uint16_t respawn_frames;
} CelestePlayerPod;

typedef struct CelesteMapHandle CelesteMapHandle;
typedef struct CelesteSimulatorHandle CelesteSimulatorHandle;

uint32_t celeste_input_pod_size(void);
uint32_t celeste_player_pod_size(void);
CelesteMapHandle *celeste_map_create(const uint8_t *map_ptr, uint32_t map_len);
void celeste_map_destroy(CelesteMapHandle *handle);
CelesteSimulatorHandle *celeste_simulator_create_pod(
    const CelestePlayerPod *snapshot, const CelesteMapHandle *map);
CelesteSimulatorHandle *celeste_simulator_create_msgpack(
    const uint8_t *snapshot_ptr, uint32_t snapshot_len, const CelesteMapHandle *map);
void celeste_simulator_destroy(CelesteSimulatorHandle *handle);
uint32_t celeste_simulator_run_pod(
    CelesteSimulatorHandle *handle,
    const CelesteInputPod *inputs_ptr,
    uint32_t inputs_len,
    CelestePlayerPod *out_states,
    uint32_t out_capacity);
uint32_t celeste_simulator_snapshot_msgpack(
    const CelesteSimulatorHandle *handle, uint8_t *out_buffer, uint32_t out_size);
uint32_t celeste_player_pod_from_msgpack(
    const uint8_t *snapshot_ptr, uint32_t snapshot_len, CelestePlayerPod *out_snapshot);
uint32_t celeste_player_pod_to_msgpack(
    const CelestePlayerPod *snapshot, uint8_t *out_buffer, uint32_t out_size);

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
