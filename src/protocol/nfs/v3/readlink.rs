use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_readlink(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut handle = nfs3::nfs_fh3::default();
    handle.deserialize(input)?;
    debug!("nfsproc3_readlink({:?},{:?}) ", xid, handle);

    let id = context.vfs.fh_to_id(&handle);
    // fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        return Ok(());
    }

    let id = id.unwrap();
    // if the id does not exist, we fail
    let symlink_attr = match context.vfs.getattr(id).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(stat) => {
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::post_op_attr::Void.serialize(output)?;
            return Ok(());
        }
    };

    match context.vfs.readlink(id).await {
        Ok(path) => {
            debug!(" {:?} --> {:?}", xid, path);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            symlink_attr.serialize(output)?;
            path.serialize(output)?;
        }
        Err(stat) => {
            // failed to read link
            // retry with failure and the post_op_attr
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            symlink_attr.serialize(output)?;
        }
    }

    Ok(())
}
