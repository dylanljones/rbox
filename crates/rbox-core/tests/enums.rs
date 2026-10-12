// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use rbox_core::FileType;

#[test]
fn file_type_codes_and_extension_aliases_are_preserved() {
    for (value, code) in [
        (FileType::INVALID, 0),
        (FileType::MP3, 1),
        (FileType::MP4, 3),
        (FileType::M4A, 4),
        (FileType::FLAC, 5),
        (FileType::ALAC, 6),
        (FileType::WAV, 11),
        (FileType::AIFF, 12),
        (FileType::VIDEO, 16),
        (FileType::StreamApple, 26),
    ] {
        assert_eq!(value as i32, code);
        assert_eq!(FileType::try_from(code).unwrap(), value);
    }
    for code in [-1, 2, i32::MAX] {
        assert!(FileType::try_from(code).is_err());
    }
    assert_eq!(FileType::try_from_extension("AAC").unwrap(), FileType::AAC);
    assert_eq!(FileType::try_from_extension("aif").unwrap(), FileType::AIF);
    assert_eq!(FileType::try_from_extension("m4a").unwrap(), FileType::M4A);
    assert_eq!(
        FileType::try_from_extension("alac").unwrap(),
        FileType::ALAC
    );
    assert!(FileType::try_from_extension("unknown").is_err());
}
