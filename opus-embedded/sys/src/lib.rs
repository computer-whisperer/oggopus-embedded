/*
 * Copyright (c) 2025 Tomi Leppänen
 * SPDX-License-Identifier: BSD-3-Clause
 */
/*!
 * Minimal bindings for opus decoder.
 *
 * Focused on no_alloc use on embedded ARM platforms.
 *
 * The documentation is not very well-formatted in places and you might want to look at [Opus
 * documentation instead](https://www.opus-codec.org/docs/html_api/index.html) instead. In
 * particular, the page about [Opus
 * Decoder](https://www.opus-codec.org/docs/html_api/group__opusdecoder.html) may be handy.
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![no_std]

#[cfg(target_os = "none")]
use core::ffi::{c_char, c_int, CStr};

pub const OPUS_DECODER_SIZE_CH1: usize = 17860;
pub const OPUS_DECODER_SIZE_CH2: usize = 26580;

/// Reserved size of the hand-written [`OpusDecoder`] blob. The `stereo` feature
/// selects the larger two-channel layout.
#[cfg(not(feature = "stereo"))]
const OPUS_DECODER_SIZE: usize = OPUS_DECODER_SIZE_CH1;
#[cfg(feature = "stereo")]
const OPUS_DECODER_SIZE: usize = OPUS_DECODER_SIZE_CH2;

/// Opaque libopus decoder state, reserved with enough storage (and 4-byte
/// alignment) to hold opus-1.5.2's `OpusDecoder` so it can live on the stack or
/// in caller-provided memory.
///
/// This is hand-written rather than bindgen-generated on purpose: bindgen's
/// `<div rustbindgen replaces=...>` annotation (formerly used in `decoder.h` to
/// reserve this size) is parsed from clang's doc-comment AST and its handling is
/// libclang-version-sensitive. Under libclang >= 21 the annotation is silently
/// ignored, the type collapses to an opaque size-1 struct, and bindgen's
/// generated layout assertion (`size_of - 17860`) underflows at const-eval —
/// breaking any build on a host with a newer system libclang. Defining the type
/// here keeps the reserved size pinned in Rust and independent of the toolchain
/// the host happens to ship. `build.rs` blocklists the C `OpusDecoder` so the
/// generated FFI signatures reference this definition.
#[repr(C, align(4))]
pub struct OpusDecoder {
    _reserved: [u8; OPUS_DECODER_SIZE],
}

impl Default for OpusDecoder {
    fn default() -> Self {
        // Zero-initialized, matching the prior bindgen-derived `Default` and the
        // `write_bytes(.., 0, ..)` path in the higher-level crate. libopus
        // initializes the state itself via `opus_decoder_init`.
        Self {
            _reserved: [0u8; OPUS_DECODER_SIZE],
        }
    }
}

impl core::fmt::Debug for OpusDecoder {
    // The reserved byte array is too large to derive `Debug` on; the contents
    // are opaque libopus state anyway.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OpusDecoder").finish_non_exhaustive()
    }
}

include!(concat!(env!("OUT_DIR"), "/opus_decoder_gen.rs"));

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn celt_fatal(str_: *const c_char, file: *const c_char, line: c_int) {
    /*!
     * Celt fatal implementation that doesn't need C stdlib.
     *
     * # Panics
     * Always.
     *
     * # Safety
     * Caller should ensure that these are valid C strings. Additionally this checks for null
     * pointers.
     */
    unsafe {
        if str_.is_null() {
            panic!("celt_fatal: str_ is null");
        }
        if file.is_null() {
            panic!("celt_fatal: file is null");
        }
        let str_ = CStr::from_ptr(str_);
        let file = CStr::from_ptr(file);
        panic!(
            "{}: {}: {}",
            str_.to_str().unwrap(),
            file.to_str().unwrap(),
            line
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_reported_size() {
        let size = unsafe { opus_decoder_get_size(1) };
        assert_eq!(size, OPUS_DECODER_SIZE_CH1.try_into().unwrap());
        let size = unsafe { opus_decoder_get_size(2) };
        assert_eq!(size, OPUS_DECODER_SIZE_CH2.try_into().unwrap());
    }

    #[test]
    fn check_struct_size() {
        assert_eq!(
            core::mem::size_of::<OpusDecoder>(),
            if cfg!(feature = "stereo") {
                OPUS_DECODER_SIZE_CH2
            } else {
                OPUS_DECODER_SIZE_CH1
            }
        );
    }
}
