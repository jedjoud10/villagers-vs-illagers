
pub const SPRITE_WIDTH: u32 = 80;
pub const SPRITE_HEIGHT: u32 = 40;
pub const SPRITE_FLAGS: u32 = 1; // BLIT_2BPP
pub const SPRITE: [u8; 800] = [
    0x80, 0x02, 0xa8, 0x00, 0x2a, 0xa0, 0x00, 0xaa, 0x55, 0xaa, 0xa5, 0x55, 0xaa, 0x11, 0x1a, 0xa5,
    0x55, 0xaa, 0x00, 0x0a, 0xb0, 0x0e, 0xab, 0x00, 0xea, 0xac, 0x03, 0xa9, 0xea, 0xea, 0xa5, 0x55,
    0xa4, 0x44, 0x41, 0xa5, 0x55, 0xaa, 0x00, 0x0a, 0xbc, 0x3e, 0xab, 0xc3, 0xea, 0xaf, 0x0f, 0xa9,
    0xbb, 0x6a, 0xaf, 0xff, 0xaa, 0xff, 0xfa, 0xa0, 0x00, 0xaa, 0xff, 0xfa, 0x88, 0x0a, 0xa8, 0x80,
    0xaa, 0xa2, 0x08, 0xa9, 0x88, 0x4a, 0xa6, 0x59, 0xaa, 0x65, 0x9a, 0xa3, 0x0c, 0xaa, 0x20, 0x8a,
    0x81, 0x42, 0xa8, 0x14, 0x2a, 0xa0, 0x50, 0xa9, 0xaa, 0x4a, 0xa5, 0x55, 0xaa, 0x55, 0x5a, 0xa0,
    0x50, 0xaa, 0x0f, 0x0a, 0x81, 0x42, 0xa8, 0x14, 0x26, 0xa0, 0x50, 0xa8, 0x29, 0xba, 0xa7, 0x5d,
    0xaa, 0x75, 0xda, 0xa7, 0x5d, 0xaa, 0x0f, 0x0a, 0xbd, 0x7e, 0xab, 0xd7, 0x7c, 0xaf, 0x5f, 0xa8,
    0x69, 0x6a, 0xaf, 0x5f, 0xaa, 0xf5, 0xfa, 0xaf, 0x5f, 0xa0, 0x5f, 0x50, 0xbf, 0xfe, 0xab, 0xff,
    0xe6, 0xaf, 0xcf, 0xaa, 0x69, 0xaa, 0xaf, 0xff, 0xaa, 0xff, 0xfa, 0xaf, 0xff, 0xa0, 0x00, 0x00,
    0xaf, 0xfa, 0xab, 0xff, 0xaa, 0xab, 0x3e, 0xaa, 0x55, 0xaa, 0xab, 0xfe, 0xaa, 0xbf, 0xea, 0xab,
    0xfe, 0xa0, 0x40, 0x10, 0xaf, 0xfa, 0xab, 0xfe, 0xaa, 0xab, 0x3e, 0xaa, 0xaa, 0xaa, 0xab, 0xfe,
    0xaa, 0xbf, 0xea, 0xab, 0xfe, 0xaa, 0x05, 0x0a, 0x80, 0x02, 0xa8, 0x00, 0x2a, 0x60, 0x00, 0x9a,
    0x55, 0xaa, 0x33, 0x0c, 0xc2, 0xa0, 0xf0, 0xa0, 0x00, 0xaa, 0x00, 0x0a, 0xb0, 0x0e, 0xab, 0x00,
    0xea, 0xac, 0x03, 0xa9, 0xea, 0xea, 0xdd, 0xff, 0x7f, 0xc3, 0x0c, 0xac, 0x03, 0xaa, 0x3f, 0xca,
    0xbc, 0x3e, 0xab, 0xc3, 0xea, 0x6f, 0x0f, 0x99, 0xbb, 0x6a, 0xd5, 0xff, 0x72, 0xac, 0x83, 0xa3,
    0xfc, 0xaa, 0xc0, 0x3a, 0x88, 0x0a, 0xe8, 0x80, 0xaa, 0x92, 0x08, 0x69, 0x88, 0x6a, 0x37, 0x0d,
    0xca, 0xae, 0x8b, 0xa8, 0x08, 0xaa, 0x20, 0x8a, 0x81, 0x42, 0x08, 0x14, 0x2a, 0x20, 0x50, 0x89,
    0xaa, 0x68, 0x0c, 0x03, 0x0a, 0xae, 0x8b, 0x00, 0xf0, 0x0a, 0xcf, 0x0a, 0x81, 0x42, 0xc8, 0x14,
    0x29, 0x3c, 0x53, 0xc8, 0x29, 0x52, 0xaa, 0xaa, 0xaa, 0xa3, 0xac, 0x00, 0xf0, 0x0a, 0x0f, 0x3a,
    0xbd, 0x7f, 0xeb, 0xd7, 0x55, 0xaf, 0x5f, 0xa8, 0x69, 0xba, 0xaa, 0xaa, 0xaa, 0xa0, 0xf0, 0x05,
    0xf5, 0x03, 0x5f, 0x50, 0xff, 0xff, 0xab, 0xff, 0x29, 0xab, 0xce, 0xaa, 0x69, 0x6a, 0xaa, 0xaa,
    0xaa, 0xaa, 0xaa, 0xa0, 0x00, 0xac, 0x30, 0x33, 0x0f, 0xfa, 0xab, 0xff, 0xaa, 0xab, 0x3e, 0xaa,
    0x55, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xa4, 0x01, 0xa0, 0xc0, 0xd0, 0xaf, 0xfa, 0xab, 0xfe,
    0xaa, 0xab, 0x3e, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xa0, 0x50, 0xaa, 0x05, 0x3a,
    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xa2, 0xaa, 0xaa, 0xaa, 0xaa,
    0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xab, 0xff, 0xea, 0xaa, 0x8a, 0xa3, 0xff, 0xea, 0x2a, 0xa2, 0x8b,
    0xff, 0xea, 0x2a, 0xaa, 0xaa, 0xa2, 0xaa, 0xaa, 0xaa, 0xab, 0x55, 0xea, 0xaa, 0xa2, 0x8b, 0x55,
    0xe2, 0xaa, 0x8a, 0xab, 0x55, 0xe2, 0xaa, 0xaa, 0xaa, 0x80, 0xa2, 0xaa, 0xaa, 0xab, 0x55, 0xea,
    0xaa, 0xaa, 0xa3, 0x55, 0xe0, 0xaa, 0xa8, 0xa3, 0x55, 0xe0, 0xaa, 0xaa, 0xaa, 0xa0, 0x82, 0xaa,
    0xaa, 0xff, 0xff, 0xff, 0xaa, 0xa8, 0xc3, 0xff, 0xf3, 0xa2, 0xaa, 0xcf, 0xff, 0xf3, 0xa2, 0xaa,
    0x8a, 0xaa, 0x82, 0xaa, 0xaa, 0xd5, 0x55, 0x57, 0xaa, 0xa2, 0xc4, 0x5f, 0xf4, 0x8a, 0xa2, 0xc4,
    0x5f, 0xf4, 0x8a, 0xaa, 0xaa, 0x2a, 0x82, 0xaa, 0xaa, 0xd5, 0x55, 0x57, 0xaa, 0xaa, 0xd5, 0x57,
    0xd7, 0xaa, 0xa8, 0xd5, 0x57, 0xd7, 0xaa, 0xaa, 0x8a, 0x0a, 0xa2, 0xaa, 0xbf, 0xff, 0xff, 0xff,
    0xfe, 0x3c, 0xff, 0xff, 0xff, 0xfe, 0x8f, 0xff, 0xff, 0xff, 0xfe, 0xaa, 0xfe, 0x8a, 0x82, 0xaa,
    0xb5, 0x55, 0x55, 0x55, 0x5e, 0x35, 0x55, 0x55, 0x55, 0x52, 0x35, 0x55, 0x55, 0x55, 0x52, 0xaa,
    0xd7, 0x2a, 0x0a, 0xaa, 0xb5, 0x55, 0x55, 0x55, 0x5e, 0x81, 0x55, 0x55, 0x15, 0x4e, 0x81, 0x55,
    0x55, 0x15, 0x4e, 0xaa, 0xd7, 0x2a, 0x22, 0xaa, 0xbf, 0xff, 0xff, 0xff, 0xfe, 0x8f, 0xff, 0x3f,
    0xff, 0xce, 0x8f, 0xff, 0x3f, 0xff, 0xce, 0xaa, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xff, 0xff, 0xff,
    0xaa, 0xaa, 0xf3, 0xff, 0xff, 0xaa, 0xaa, 0xf3, 0xff, 0xff, 0xaa, 0x8a, 0xff, 0x55, 0x57, 0xa2,
    0xaa, 0xd5, 0x55, 0x57, 0xaa, 0xaa, 0xd4, 0x55, 0x17, 0xaa, 0xaa, 0xd4, 0x55, 0x17, 0xaa, 0xaa,
    0xf5, 0x7d, 0x77, 0xa2, 0xaa, 0xd5, 0x55, 0x57, 0xaa, 0xaa, 0xd4, 0x14, 0x57, 0xaa, 0xaa, 0xd4,
    0x14, 0x57, 0xaa, 0x8a, 0xfd, 0xff, 0x7f, 0xaa, 0xaa, 0xd5, 0xff, 0x57, 0xaa, 0xaa, 0xd5, 0x3c,
    0x57, 0xaa, 0xaa, 0xd5, 0x3c, 0x57, 0xaa, 0x8a, 0xf5, 0xff, 0x7f, 0x8a, 0xaa, 0xd5, 0xff, 0x57,
    0xaa, 0xaa, 0xd5, 0xff, 0x57, 0xaa, 0xaa, 0xd5, 0xff, 0x57, 0xaa, 0x82, 0xd5, 0xff, 0x77, 0xfe,
    0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xb3,
    0xc3, 0xff, 0x03, 0x57, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xaa, 0xc0,
    0xff, 0x03, 0xaa, 0xdf, 0xc0, 0xff, 0x03, 0x7f, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xaa, 0xc0, 0xff,
    0x03, 0xaa, 0xaa, 0xc0, 0xff, 0x03, 0xaa, 0xf5, 0xc3, 0xff, 0x03, 0x57, 0xaa, 0xff, 0xff, 0xff,
    0xaa, 0xaa, 0xff, 0xff, 0xff, 0xaa, 0xaa, 0xff, 0xff, 0xff, 0xaa, 0xff, 0xff, 0xff, 0xff, 0xff,
];