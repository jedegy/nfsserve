// this is just a complete enumeration of everything in the RFC
#![allow(dead_code)]
// And its nice to keep the original RFC names and case
#![allow(non_camel_case_types)]

use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;

use super::*;

// Transcribed from RFC 1057 Appendix A

pub const PROGRAM: u32 = 100005;
pub const VERSION: u32 = 3;

pub const MNTPATHLEN: u32 = 1024; /* Maximum bytes in a path name */
pub const MNTNAMLEN: u32 = 255; /* Maximum bytes in a name */
pub const FHSIZE3: u32 = 64; /* Maximum bytes in a V3 file handle */

pub type fhandle3 = Vec<u8>;
pub type dirpath = Vec<u8>;
pub type name = Vec<u8>;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum mountstat3 {
    MNT3_OK = 0,                 /* no error */
    MNT3ERR_PERM = 1,            /* Not owner */
    MNT3ERR_NOENT = 2,           /* No such file or directory */
    MNT3ERR_IO = 5,              /* I/O error */
    MNT3ERR_ACCES = 13,          /* Permission denied */
    MNT3ERR_NOTDIR = 20,         /* Not a directory */
    MNT3ERR_INVAL = 22,          /* Invalid argument */
    MNT3ERR_NAMETOOLONG = 63,    /* Filename too long */
    MNT3ERR_NOTSUPP = 10004,     /* Operation not supported */
    MNT3ERR_SERVERFAULT = 10006, /* A failure on the server */
}
XDREnumSerde!(mountstat3);

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct mountres3_ok {
    pub fhandle: fhandle3, // really same thing as nfs::nfs_fh3
    pub auth_flavors: Vec<u32>,
}
XDRStruct!(mountres3_ok, fhandle, auth_flavors);

/*
From RFC 1813 Appendix I
program MOUNT_PROGRAM {
 version MOUNT_V3 {
    void      MOUNTPROC3_NULL(void)    = 0;
    mountres3 MOUNTPROC3_MNT(dirpath)  = 1;
    mountlist MOUNTPROC3_DUMP(void)    = 2;
    void      MOUNTPROC3_UMNT(dirpath) = 3;
    void      MOUNTPROC3_UMNTALL(void) = 4;
    exports   MOUNTPROC3_EXPORT(void)  = 5;
 } = 3;
} = 100005;
*/

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum MountProgram {
    MOUNTPROC3_NULL = 0,
    MOUNTPROC3_MNT = 1,
    MOUNTPROC3_DUMP = 2,
    MOUNTPROC3_UMNT = 3,
    MOUNTPROC3_UMNTALL = 4,
    MOUNTPROC3_EXPORT = 5,
    INVALID,
}
XDREnumSerde!(MountProgram);
