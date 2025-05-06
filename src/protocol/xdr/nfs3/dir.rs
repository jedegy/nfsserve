// Types for directory operations in NFS3 from RFC
#![allow(dead_code)]
// And its nice to keep the original RFC names and case
#![allow(non_camel_case_types)]

use std::io::{Read, Write};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum devicetype3 {
    #[default]
    NF3CHR = 0,
    NF3BLK = 1,
    NF3SOCK = 2,
    NF3FIFO = 3,
}
XDREnumSerde!(devicetype3);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct MKDIR3args {
    pub dirops: diropargs3,
    pub attributes: sattr3,
}
XDRStruct!(MKDIR3args, dirops, attributes);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct SYMLINK3args {
    pub dirops: diropargs3,
    pub symlink: symlinkdata3,
}
XDRStruct!(SYMLINK3args, dirops, symlink);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct entry3 {
    pub fileid: fileid3,
    pub name: filename3,
    pub cookie: cookie3,
}
XDRStruct!(entry3, fileid, name, cookie);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct READDIR3args {
    pub dir: nfs_fh3,
    pub cookie: cookie3,
    pub cookieverf: cookieverf3,
    pub dircount: count3,
}
XDRStruct!(READDIR3args, dir, cookie, cookieverf, dircount);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct entryplus3 {
    pub fileid: fileid3,
    pub name: filename3,
    pub cookie: cookie3,
    pub name_attributes: post_op_attr,
    pub name_handle: post_op_fh3,
}
XDRStruct!(
    entryplus3,
    fileid,
    name,
    cookie,
    name_attributes,
    name_handle
);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct READDIRPLUS3args {
    pub dir: nfs_fh3,
    pub cookie: cookie3,
    pub cookieverf: cookieverf3,
    pub dircount: count3,
    pub maxcount: count3,
}
XDRStruct!(
    READDIRPLUS3args,
    dir,
    cookie,
    cookieverf,
    dircount,
    maxcount
);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct MKNOD3args {
    pub where_dir: diropargs3,
    pub what: mknoddata3,
}
XDRStruct!(MKNOD3args, where_dir, what);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct devicedata3 {
    pub dev_type: devicetype3,
    pub device: specdata3,
}
XDRStruct!(devicedata3, dev_type, device);

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct mknoddata3 {
    pub mknod_type: ftype3,
    pub device: devicedata3,
}
XDRStruct!(mknoddata3, mknod_type, device);
