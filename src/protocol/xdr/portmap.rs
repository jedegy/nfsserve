// this is just a complete enumeration of everything in the RFC
#![allow(dead_code)]
// And its nice to keep the original RFC names and case
#![allow(non_camel_case_types)]

use std::io::{Read, Write};

use num_derive::{FromPrimitive, ToPrimitive};

use super::*;

// Transcribed from RFC 1057 Appendix A

/// Device Number information. Ex: Major / Minor device
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct mapping {
    pub prog: u32,
    pub vers: u32,
    pub prot: u32,
    pub port: u32,
}
XDRStruct!(mapping, prog, vers, prot, port);
pub const IPPROTO_TCP: u32 = 6; /* protocol number for TCP/IP */
pub const IPPROTO_UDP: u32 = 17; /* protocol number for UDP/IP */
pub const PROGRAM: u32 = 100000;
pub const VERSION: u32 = 2;

/*
 From RFC 1057 Appendix A

 program PMAP_PROG {
    version PMAP_VERS {
       void PMAPPROC_NULL(void)         = 0;
       bool PMAPPROC_SET(mapping)       = 1;
       bool PMAPPROC_UNSET(mapping)     = 2;
       unsigned int PMAPPROC_GETPORT(mapping)   = 3;
       pmaplist PMAPPROC_DUMP(void)         = 4;
       call_result PMAPPROC_CALLIT(call_args)  = 5;
    } = 2;
 } = 100000;
*/

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum PortmapProgram {
    PMAPPROC_NULL = 0,
    PMAPPROC_SET = 1,
    PMAPPROC_UNSET = 2,
    PMAPPROC_GETPORT = 3,
    PMAPPROC_DUMP = 4,
    PMAPPROC_CALLIT = 5,
    INVALID,
}
