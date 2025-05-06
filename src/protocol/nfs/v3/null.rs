use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::xdr::{self, XDR};

pub fn nfsproc3_null(
    xid: u32,
    _: &mut impl Read,
    output: &mut impl Write,
) -> Result<(), anyhow::Error> {
    debug!("nfsproc3_null({:?}) ", xid);
    let msg = xdr::rpc::make_success_reply(xid);
    debug!("\t{:?} --> {:?}", xid, msg);
    msg.serialize(output)?;
    Ok(())
}
