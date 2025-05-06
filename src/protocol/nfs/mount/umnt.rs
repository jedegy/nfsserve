use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, mount, XDR};

pub async fn mountproc3_umnt(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut path = mount::dirpath::new();
    path.deserialize(input)?;
    let utf8path = std::str::from_utf8(&path).unwrap_or_default();
    debug!("mountproc3_umnt({:?},{:?}) ", xid, utf8path);
    if let Some(ref chan) = context.mount_signal {
        let _ = chan.send(false).await;
    }
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    mount::mountstat3::MNT3_OK.serialize(output)?;
    Ok(())
}
