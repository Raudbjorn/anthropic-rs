//! Audio encoding/decoding utilities for the Realtime API.
//!
//! The Realtime API exchanges audio as base64-encoded PCM16 data at 24kHz
//! sample rate, little-endian byte order. This module provides helpers for
//! converting between common audio representations and the API's format.

use base64::{engine::general_purpose::STANDARD, Engine};

/// Encode raw PCM16 bytes to base64 for `input_audio_buffer.append`.
///
/// The input should be raw bytes of 16-bit PCM audio at 24kHz sample rate,
/// little-endian byte order.
pub fn encode_audio_base64(pcm16_bytes: &[u8]) -> String {
    STANDARD.encode(pcm16_bytes)
}

/// Decode base64 audio from `response.audio.delta` to raw PCM16 bytes.
pub fn decode_audio_base64(b64: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(b64)
}

/// Convert f32 audio samples [-1.0, 1.0] to PCM16 i16 samples.
///
/// Values outside [-1.0, 1.0] are clamped.
pub fn float_to_pcm16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            if clamped < 0.0 {
                (clamped * 0x8000 as f32) as i16
            } else {
                (clamped * 0x7FFF as f32) as i16
            }
        })
        .collect()
}

/// Convert PCM16 i16 samples to f32 audio samples [-1.0, 1.0].
pub fn pcm16_to_float(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| {
            if s < 0 {
                s as f32 / 0x8000 as f32
            } else {
                s as f32 / 0x7FFF as f32
            }
        })
        .collect()
}

/// Convert PCM16 i16 samples to raw bytes (little-endian).
pub fn pcm16_to_bytes(samples: &[i16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

/// Convert raw bytes (little-endian) to PCM16 i16 samples.
///
/// Returns `None` if the byte slice length is not even.
pub fn bytes_to_pcm16(bytes: &[u8]) -> Option<Vec<i16>> {
    if bytes.len() % 2 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let original: Vec<u8> = vec![0x00, 0x01, 0xFF, 0x7F, 0x00, 0x80];
        let encoded = encode_audio_base64(&original);
        let decoded = decode_audio_base64(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn decode_invalid_base64_returns_error() {
        assert!(decode_audio_base64("not valid base64!!!").is_err());
    }

    #[test]
    fn encode_empty_bytes() {
        let encoded = encode_audio_base64(&[]);
        assert_eq!(encoded, "");
        let decoded = decode_audio_base64(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn float_to_pcm16_silence() {
        let samples = vec![0.0, 0.0, 0.0];
        let pcm = float_to_pcm16(&samples);
        assert_eq!(pcm, vec![0, 0, 0]);
    }

    #[test]
    fn float_to_pcm16_max_values() {
        let samples = vec![1.0, -1.0];
        let pcm = float_to_pcm16(&samples);
        assert_eq!(pcm[0], 0x7FFF); // positive max
        assert_eq!(pcm[1], -0x8000); // negative max (i16::MIN)
    }

    #[test]
    fn float_to_pcm16_clamps() {
        let samples = vec![2.0, -3.0];
        let pcm = float_to_pcm16(&samples);
        assert_eq!(pcm[0], 0x7FFF);
        assert_eq!(pcm[1], -0x8000);
    }

    #[test]
    fn pcm16_to_float_roundtrip() {
        let original = vec![0i16, 16383, -16384, 0x7FFF, -0x8000];
        let floats = pcm16_to_float(&original);
        let back = float_to_pcm16(&floats);
        // Should be very close (within 1 sample of quantization error)
        for (orig, reconverted) in original.iter().zip(back.iter()) {
            assert!(
                (orig - reconverted).abs() <= 1,
                "expected {} got {}",
                orig,
                reconverted
            );
        }
    }

    #[test]
    fn pcm16_to_float_range() {
        let samples = vec![0x7FFFi16, -0x8000i16, 0i16];
        let floats = pcm16_to_float(&samples);
        assert!((floats[0] - 1.0).abs() < f32::EPSILON);
        assert!((floats[1] - (-1.0)).abs() < f32::EPSILON);
        assert!((floats[2]).abs() < f32::EPSILON);
    }

    #[test]
    fn pcm16_bytes_roundtrip() {
        let samples = vec![0i16, 1, -1, 0x7FFF, -0x8000, 12345];
        let bytes = pcm16_to_bytes(&samples);
        assert_eq!(bytes.len(), samples.len() * 2);
        let back = bytes_to_pcm16(&bytes).unwrap();
        assert_eq!(samples, back);
    }

    #[test]
    fn bytes_to_pcm16_odd_length_returns_none() {
        assert!(bytes_to_pcm16(&[0x00, 0x01, 0x02]).is_none());
    }

    #[test]
    fn bytes_to_pcm16_empty() {
        let result = bytes_to_pcm16(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pcm16_to_bytes_empty() {
        let result = pcm16_to_bytes(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn float_to_pcm16_empty() {
        let result = float_to_pcm16(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn pcm16_to_float_empty() {
        let result = pcm16_to_float(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn full_pipeline_float_to_base64_and_back() {
        let original_floats: Vec<f32> = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let pcm = float_to_pcm16(&original_floats);
        let bytes = pcm16_to_bytes(&pcm);
        let b64 = encode_audio_base64(&bytes);
        let decoded_bytes = decode_audio_base64(&b64).unwrap();
        let decoded_pcm = bytes_to_pcm16(&decoded_bytes).unwrap();
        let decoded_floats = pcm16_to_float(&decoded_pcm);

        for (orig, decoded) in original_floats.iter().zip(decoded_floats.iter()) {
            assert!(
                (orig - decoded).abs() < 0.001,
                "expected {} got {}",
                orig,
                decoded
            );
        }
    }

    #[test]
    fn little_endian_byte_order() {
        // 0x0100 = 256 in little-endian is [0x00, 0x01]
        let samples = vec![256i16];
        let bytes = pcm16_to_bytes(&samples);
        assert_eq!(bytes, vec![0x00, 0x01]);
    }
}
