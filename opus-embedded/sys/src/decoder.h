#include <opus.h>

/*
 * `OpusDecoder` is opaque in libopus. Rather than have bindgen synthesize a
 * fixed-size stand-in from an annotation here (which broke under newer
 * libclang), the type is blocklisted in build.rs and hand-written in lib.rs as
 * a byte blob sized to hold opus-1.5.2's decoder. See OPUS_DECODER_SIZE_CH1 /
 * OPUS_DECODER_SIZE_CH2 in lib.rs for the reserved sizes.
 */
