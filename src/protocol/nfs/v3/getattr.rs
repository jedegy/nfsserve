use std::io::{Read, Write};

use tracing::{debug, error};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_getattr(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut handle = nfs3::nfs_fh3::default();
    handle.deserialize(input)?;
    debug!("nfsproc3_getattr({:?},{:?}) ", xid, handle);

    let id = context.vfs.fh_to_id(&handle);
    // fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();
    match context.vfs.getattr(id).await {
        Ok(fh) => {
            debug!(" {:?} --> {:?}", xid, fh);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            fh.serialize(output)?;
        }
        Err(stat) => {
            error!("nfsproc3_getattr error {:?} --> {:?}", xid, stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
        }
    }
    Ok(())
}
