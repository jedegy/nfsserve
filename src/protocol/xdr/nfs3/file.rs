// Types for file operations in NFS3 from RFC
#![allow(dead_code)]
// And its nice to keep the original RFC names and case
#![allow(non_camel_case_types)]

use std::io::{Read, Write};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct READ3args {
    pub file: nfs_fh3,
    pub offset: offset3,
    pub count: count3,
}
XDRStruct!(READ3args, file, offset, count);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct READ3resok {
    pub file_attributes: post_op_attr,
    pub count: count3,
    pub eof: bool,
    pub data: Vec<u8>,
}
XDRStruct!(READ3resok, file_attributes, count, eof, data);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct COMMIT3args {
    pub file: nfs_fh3,
    pub offset: offset3,
    pub count: count3,
}
XDRStruct!(COMMIT3args, file, offset, count);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct COMMIT3resok {
    pub file_wcc: wcc_data,
    pub verf: writeverf3,
}
XDRStruct!(COMMIT3resok, file_wcc, verf);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct LINK3args {
    pub file: nfs_fh3,
    pub link: diropargs3,
}
XDRStruct!(LINK3args, file, link);

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum stable_how {
    #[default]
    UNSTABLE = 0,
    DATA_SYNC = 1,
    FILE_SYNC = 2,
}
XDREnumSerde!(stable_how);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct WRITE3args {
    pub file: nfs_fh3,
    pub offset: offset3,
    pub count: count3,
    pub stable: u32,
    pub data: Vec<u8>,
}
XDRStruct!(WRITE3args, file, offset, count, stable, data);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct WRITE3resok {
    pub file_wcc: wcc_data,
    pub count: count3,
    pub committed: stable_how,
    pub verf: writeverf3,
}
XDRStruct!(WRITE3resok, file_wcc, count, committed, verf);
