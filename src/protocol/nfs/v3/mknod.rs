use std::io::{Read, Write};

use tracing::{debug, error, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_mknod(
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

    let mut args = nfs3::MKNOD3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_mknod({:?}, {:?}) ", xid, args);

    // find the directory we are supposed to create the special file in
    let dirid = context.vfs.fh_to_id(&args.where_dir.dir);
    if let Err(stat) = dirid {
        // directory does not exist
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        error!("Directory does not exist");
        return Ok(());
    }
    // found the directory, get the attributes
    let dirid = dirid.unwrap();

    // get the object attributes before the operation
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

    // Create default attributes if necessary
    let attr = nfs3::sattr3::default();

    // Call VFS mknod method
    match context
        .vfs
        .mknod(
            dirid,
            &args.where_dir.name,
            args.what.mknod_type,
            args.what.device.device,
            &attr,
        )
        .await
    {
        Ok((fid, fattr)) => {
            debug!("nfsproc3_mknod success --> {:?}, {:?}", fid, fattr);

            // Get the directory attributes after the operation
            let post_dir_attr = match context.vfs.getattr(dirid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            let wcc_res = nfs3::wcc_data {
                before: pre_dir_attr,
                after: post_dir_attr,
            };

            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            // serialize MKNOD3resok
            let fh = context.vfs.id_to_fh(fid);
            nfs3::post_op_fh3::handle(fh).serialize(output)?;
            nfs3::post_op_attr::attributes(fattr).serialize(output)?;
            wcc_res.serialize(output)?;
        }
        Err(stat) => {
            debug!("nfsproc3_mknod error --> {:?}", stat);

            // Get the directory attributes after the operation (unchanged)
            let post_dir_attr = match context.vfs.getattr(dirid).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            let wcc_res = nfs3::wcc_data {
                before: pre_dir_attr,
                after: post_dir_attr,
            };

            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            wcc_res.serialize(output)?;
        }
    }

    Ok(())
}
