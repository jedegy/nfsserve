use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_lookup(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut dirops = nfs3::diropargs3::default();
    dirops.deserialize(input)?;
    debug!("nfsproc3_lookup({:?},{:?}) ", xid, dirops);

    let dirid = context.vfs.fh_to_id(&dirops.dir);

    // fail if unable to convert file handle
    if let Err(stat) = dirid {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        return Ok(());
    }

    let dirid = dirid.unwrap();

    let dir_attr = match context.vfs.getattr(dirid).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(_) => nfs3::post_op_attr::Void,
    };

    match context.vfs.lookup(dirid, &dirops.name).await {
        Ok(fid) => {
            let obj_attr = match context.vfs.getattr(fid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            debug!("nfsproc3_lookup success {:?} --> {:?}", xid, obj_attr);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            context.vfs.id_to_fh(fid).serialize(output)?;
            obj_attr.serialize(output)?;
            dir_attr.serialize(output)?;
        }
        Err(stat) => {
            debug!(
                "nfsproc3_lookup error {:?}({:?}) --> {:?}",
                xid, dirops.name, stat
            );
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            dir_attr.serialize(output)?;
        }
    }
    Ok(())
}
