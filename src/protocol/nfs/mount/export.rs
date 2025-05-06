use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, XDR};

pub fn mountproc3_export(
    xid: u32,
    _: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    debug!("mountproc3_export({:?}) ", xid);
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    true.serialize(output)?;
    // dirpath
    context.export_name.as_bytes().to_vec().serialize(output)?;
    // groups
    false.serialize(output)?;
    // next exports
    false.serialize(output)?;
    Ok(())
}
