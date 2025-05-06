use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, XDR};

/*
 * We fake a portmapper here. And always direct back to the same host port
 */
pub fn pmapproc_getport(
    xid: u32,
    read: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut mapping = xdr::portmap::mapping::default();
    mapping.deserialize(read)?;
    debug!("pmapproc_getport({:?}, {:?}) ", xid, mapping);
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    let port = context.local_port as u32;
    debug!("\t{:?} --> {:?}", xid, port);
    port.serialize(output)?;
    Ok(())
}
