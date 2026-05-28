/*
 * SPDX-License-Identifier: BSD-3-Clause
 *
 * Non-variadic wrappers around opus_encoder_ctl / opus_decoder_ctl.
 *
 * The upstream ctl functions are C variadic (`int request, ...`), and
 * emscripten's STANDALONE_WASM mode cannot dispatch variadic args from JS:
 * the trailing arg silently disappears and the ctl runs with no effect.
 * These thin wrappers expose fixed-signature entry points so the JS shim
 * can set bitrate / get state across the wasm boundary.
 *
 * Lives outside `vendor/` so the vendored libopus source tree stays
 * unmodified upstream. Compiled alongside the libopus sources by build.rs.
 */

#include "opus.h"

/* ---- Encoder setters (fixed-arg) ---------------------------------------- */

int opus_helpers_encoder_set_bitrate(OpusEncoder *st, int bitrate_bps) {
    return opus_encoder_ctl(st, OPUS_SET_BITRATE(bitrate_bps));
}

int opus_helpers_encoder_set_complexity(OpusEncoder *st, int complexity) {
    return opus_encoder_ctl(st, OPUS_SET_COMPLEXITY(complexity));
}

int opus_helpers_encoder_set_signal(OpusEncoder *st, int signal) {
    return opus_encoder_ctl(st, OPUS_SET_SIGNAL(signal));
}

int opus_helpers_encoder_set_inband_fec(OpusEncoder *st, int on) {
    return opus_encoder_ctl(st, OPUS_SET_INBAND_FEC(on));
}

int opus_helpers_encoder_set_packet_loss_perc(OpusEncoder *st, int loss) {
    return opus_encoder_ctl(st, OPUS_SET_PACKET_LOSS_PERC(loss));
}

int opus_helpers_encoder_set_vbr(OpusEncoder *st, int on) {
    return opus_encoder_ctl(st, OPUS_SET_VBR(on));
}

/* ---- Encoder getters (fixed-arg, return value or sentinel) -------------- */

int opus_helpers_encoder_get_bitrate(OpusEncoder *st) {
    int b = 0;
    int r = opus_encoder_ctl(st, OPUS_GET_BITRATE(&b));
    return r == OPUS_OK ? b : -1;
}

/* ---- Decoder setters / getters ----------------------------------------- */

int opus_helpers_decoder_set_gain(OpusDecoder *st, int gain_q8) {
    return opus_decoder_ctl(st, OPUS_SET_GAIN(gain_q8));
}
