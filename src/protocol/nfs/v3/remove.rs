use std::io::{Read, Write};

use tracing::{debug, error, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_remove(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    // if we do not have write capabilities
    if !matches!(context.vfs.capabilities(), vfs::Capabilities::ReadWrite) {
        warn!("No write capabilities.");
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        nfs3::nfsstat3::NFS3ERR_ROFS.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }

    let mut dirops = nfs3::diropargs3::default();
    dirops.deserialize(input)?;

    debug!("nfsproc3_remove({:?}, {:?}) ", xid, dirops);

    // find the directory with the file
    let dirid = context.vfs.fh_to_id(&dirops.dir);
    if let Err(stat) = dirid {
        // directory does not exist
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        error!("Directory does not exist");
        return Ok(());
    }
    let dirid = dirid.unwrap();

    // get the object attributes before the write
    let pre_dir_attr = match context.vfs.getattr(dirid).await {
        Ok(v) => {
            let wccattr = nfs3::wcc_attr {
                size: v.size,
                mtime: v.mtime,
                ctime: v.ctime,
            };
            nfs3::pre_op_attr::attributes(wccattr)
        }
        Err(stat) => {
            error!("Cannot stat directory");
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::wcc_data::default().serialize(output)?;
            return Ok(());
        }
    };

    // delete!
    let res = context.vfs.remove(dirid, &dirops.name).await;

    // Re-read dir attributes for post op attr
    let post_dir_attr = match context.vfs.getattr(dirid).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(_) => nfs3::post_op_attr::Void,
    };
    let wcc_res = nfs3::wcc_data {
        before: pre_dir_attr,
        after: post_dir_attr,
    };

    match res {
        Ok(()) => {
            debug!("remove success");
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            wcc_res.serialize(output)?;
        }
        Err(e) => {
            error!("remove error {:?} --> {:?}", xid, e);
            // serialize CREATE3resfail
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            e.serialize(output)?;
            wcc_res.serialize(output)?;
        }
    }

    Ok(())
}
