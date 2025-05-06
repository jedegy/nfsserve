use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::xdr::{self, XDR};

pub fn mountproc3_null(xid: u32, _: &mut impl Read, output: &mut impl Write) -> Result<(), anyhow::Error> {
    debug!("mountproc3_null({:?}) ", xid);
    // build an RPC reply
    let msg = xdr::rpc::make_success_reply(xid);
    debug!("\t{:?} --> {:?}", xid, msg);
    msg.serialize(output)?;
    Ok(())
}
