use std::io::{Read, Write};

use tracing::{debug, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_link(
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
        nfs3::post_op_attr::Void.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }

    let mut args = nfs3::LINK3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_link({:?}, {:?}) ", xid, args);

    // Get the file id
    let fileid = context.vfs.fh_to_id(&args.file);
    if let Err(stat) = fileid {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }
    let fileid = fileid.unwrap();

    // Get the directory id
    let dirid = context.vfs.fh_to_id(&args.link.dir);
    if let Err(stat) = dirid {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }
    let dirid = dirid.unwrap();

    // Get the directory attributes before the operation
    let pre_dir_attr = match context.vfs.getattr(dirid).await {
        Ok(v) => {
            let wccattr = nfs3::wcc_attr {
                size: v.size,
                mtime: v.mtime,
                ctime: v.ctime,
            };
            nfs3::pre_op_attr::attributes(wccattr)
        }
        Err(_) => nfs3::pre_op_attr::Void,
    };

    // Call VFS link method
    match context.vfs.link(fileid, dirid, &args.link.name).await {
        Ok(fattr) => {
            // Get file attributes
            let file_attr = nfs3::post_op_attr::attributes(fattr);

            // Get the directory attributes after the operation
            let post_dir_attr = match context.vfs.getattr(dirid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            let wcc_res = nfs3::wcc_data {
                before: pre_dir_attr,
                after: post_dir_attr,
            };

            debug!("nfsproc3_link success");
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            file_attr.serialize(output)?;
            wcc_res.serialize(output)?;
        }
        Err(stat) => {
            // Get file attributes
            let file_attr = match context.vfs.getattr(fileid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            // Get the directory attributes after the operation (unchanged)
            let post_dir_attr = match context.vfs.getattr(dirid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            let wcc_res = nfs3::wcc_data {
                before: pre_dir_attr,
                after: post_dir_attr,
            };

            debug!("nfsproc3_link failed: {:?}", stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            file_attr.serialize(output)?;
            wcc_res.serialize(output)?;
        }
    }

    Ok(())
}
