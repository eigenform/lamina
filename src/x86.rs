//! Collection of weird encodings for some x86_64 instructions.

pub const CLWB_BYTE_PTR_R15: [u8; 5] = [ 0x66, 0x41, 0x0f, 0xae, 0x37 ];

pub const MCOMMIT: [u8; 4] = [ 0xf3, 0x0f, 0x01, 0xfa ];

pub const MFENCE: [u8; 3] = [ 0x0f, 0xae, 0xf0 ];
pub const MFENCE_8: [u8; 8] = [
    0x67, 0x67, 0x67, 0x67, 0x67,
    0x0f, 0xae, 0xf0
];

pub const LFENCE: [u8; 3] = [ 0x0f, 0xae, 0xe8 ];
pub const LFENCE_8: [u8; 8] = [ 
    0x67, 0x67, 0x67, 0x67, 0x67, 
    0x0f, 0xae, 0xe8 
];

pub const RDPRU: [u8; 3] = [ 0x0f, 0x01, 0xfd ];
pub const RDPRU_8: [u8; 8] = [
    0x66, 0x66, 0x66, 0x66, 0x66,
    0x0f, 0x01, 0xfd
];

pub const RDPMC: [u8; 2] = [ 0x0f, 0x33 ];
pub const RDPMC_8: [u8; 8] = [
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x0f, 0x33
];

pub const SUB_R8_RAX: [u8; 3] = [ 0x49, 0x29, 0xc0 ];
pub const SUB_R8_RAX_8: [u8; 8] = [ 
    0x66, 0x66, 0x66, 0x66, 0x66,
    0x49, 0x29, 0xc0 
];

pub const MOV_RDI_RAX: [u8; 3] = [ 0x48, 0x89, 0xc7 ];
pub const MOV_RDI_RAX_8: [u8; 8] = [
    0x66, 0x66, 0x66, 0x66, 0x66,
    0x48, 0x89, 0xc7
];

pub const MOV_RCX_1: [u8; 7] = [ 0x48, 0xc7, 0xc1, 0x01, 0x00, 0x00, 0x00 ];
pub const MOV_RCX_1_8: [u8; 8] = [ 
    0x66, 0x48, 0xc7, 0xc1, 0x01, 0x00, 0x00, 0x00 
];

pub const XOR_R8_R8_1: [u8; 3] = [ 0x4d, 0x31, 0xc0 ];
pub const XOR_R8_R8_8: [u8; 8] = [ 
    0x66, 0x66, 0x66, 0x66, 0x66,
    0x4d, 0x31, 0xc0 
];

pub const NOP_1:  [u8; 1]  = [ 0x90 ];
pub const NOP_2:  [u8; 2]  = [ 0x66, 0x90 ];
pub const NOP_3:  [u8; 3]  = [ 0x0F, 0x1F, 0x00 ];
pub const NOP_4:  [u8; 4]  = [ 0x0F, 0x1F, 0x40, 0x00 ];
pub const NOP_5:  [u8; 5]  = [ 0x0F, 0x1F, 0x44, 0x00, 0x00 ];
pub const NOP_6:  [u8; 6]  = [ 0x66, 0x0F, 0x1F, 0x44, 0x00, 0x00 ];
pub const NOP_7:  [u8; 7]  = [ 0x0F, 0x1F, 0x80, 0x00, 0x00, 0x00, 0x00 ];
pub const NOP_8:  [u8; 8]  = [ 
    0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00 
];
pub const NOP_9:  [u8; 9]  = [
    0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_10: [u8; 10] = [
    0x66, 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_11: [u8; 11] = [
    0x66, 0x66, 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_12: [u8; 12] = [
    0x66, 0x66, 0x66, 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_13: [u8; 13] = [
    0x66, 0x66, 0x66, 0x66, 0x66, 0x0F, 0x1F, 0x84, 
    0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_14: [u8; 14] = [
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x0F, 0x1F, 
    0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];
pub const NOP_15: [u8; 15] = [
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x0F, 
    0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00
];


