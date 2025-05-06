use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, mount, XDR};

pub async fn mountproc3_umnt_all(
    xid: u32,
    _input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    debug!("mountproc3_umnt_all({:?}) ", xid);
    if let Some(ref chan) = context.mount_signal {
        let _ = chan.send(false).await;
    }
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    mount::mountstat3::MNT3_OK.serialize(output)?;
    Ok(())
}
