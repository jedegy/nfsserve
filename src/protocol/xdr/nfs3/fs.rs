// Types for file system operations in NFS3 from RFC
#![allow(dead_code)]
// And its nice to keep the original RFC names and case
#![allow(non_camel_case_types)]

use std::io::{Read, Write};

use super::*;

// Section 3.3.19. Procedure 19: FSINFO - Get static file system Information
// The following constants are used in fsinfo to construct the bitmask 'properties',
// which represents the file system properties.

/// If this bit is 1 (TRUE), the file system supports hard links.
pub const FSF_LINK: u32 = 0x0001;

/// If this bit is 1 (TRUE), the file system supports symbolic links.
pub const FSF_SYMLINK: u32 = 0x0002;

/// If this bit is 1 (TRUE), the information returned by
/// PATHCONF is identical for every file and directory
/// in the file system. If it is 0 (FALSE), the client
/// should retrieve PATHCONF information for each file
/// and directory as required.
pub const FSF_HOMOGENEOUS: u32 = 0x0008;

/// If this bit is 1 (TRUE), the server will set the
/// times for a file via SETATTR if requested (to the
/// accuracy indicated by time_delta). If it is 0
/// (FALSE), the server cannot set times as requested.
pub const FSF_CANSETTIME: u32 = 0x0010;

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct fsinfo3 {
    pub obj_attributes: post_op_attr,
    pub rtmax: u32,
    pub rtpref: u32,
    pub rtmult: u32,
    pub wtmax: u32,
    pub wtpref: u32,
    pub wtmult: u32,
    pub dtpref: u32,
    pub maxfilesize: size3,
    pub time_delta: nfstime3,
    pub properties: u32,
}
XDRStruct!(
    fsinfo3,
    obj_attributes,
    rtmax,
    rtpref,
    rtmult,
    wtmax,
    wtpref,
    wtmult,
    dtpref,
    maxfilesize,
    time_delta,
    properties
);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct FSSTAT3resok {
    pub obj_attributes: post_op_attr,
    pub tbytes: size3,
    pub fbytes: size3,
    pub abytes: size3,
    pub tfiles: size3,
    pub ffiles: size3,
    pub afiles: size3,
    pub invarsec: u32,
}
XDRStruct!(
    FSSTAT3resok,
    obj_attributes,
    tbytes,
    fbytes,
    abytes,
    tfiles,
    ffiles,
    afiles,
    invarsec
);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct PATHCONF3resok {
    pub obj_attributes: post_op_attr,
    pub linkmax: u32,
    pub name_max: u32,
    pub no_trunc: bool,
    pub chown_restricted: bool,
    pub case_insensitive: bool,
    pub case_preserving: bool,
}
XDRStruct!(
    PATHCONF3resok,
    obj_attributes,
    linkmax,
    name_max,
    no_trunc,
    chown_restricted,
    case_insensitive,
    case_preserving
);
