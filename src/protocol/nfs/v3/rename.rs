use std::io::{Read, Write};

use tracing::{debug, error, warn};

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_rename(
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

    let mut fromdirops = nfs3::diropargs3::default();
    let mut todirops = nfs3::diropargs3::default();
    fromdirops.deserialize(input)?;
    todirops.deserialize(input)?;

    debug!(
        "nfsproc3_rename({:?}, {:?}, {:?}) ",
        xid, fromdirops, todirops
    );

    // find the from directory
    let from_dirid = context.vfs.fh_to_id(&fromdirops.dir);
    if let Err(stat) = from_dirid {
        // directory does not exist
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        error!("Directory does not exist");
        return Ok(());
    }

    // find the to directory
    let to_dirid = context.vfs.fh_to_id(&todirops.dir);
    if let Err(stat) = to_dirid {
        // directory does not exist
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        error!("Directory does not exist");
        return Ok(());
    }

    // found the directory, get the attributes
    let from_dirid = from_dirid.unwrap();
    let to_dirid = to_dirid.unwrap();

    // get the object attributes before the write
    let pre_from_dir_attr = match context.vfs.getattr(from_dirid).await {
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

    // get the object attributes before the write
    let pre_to_dir_attr = match context.vfs.getattr(to_dirid).await {
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

    // rename!
    let res = context
        .vfs
        .rename(from_dirid, &fromdirops.name, to_dirid, &todirops.name)
        .await;

    // Re-read dir attributes for post op attr
    let post_from_dir_attr = match context.vfs.getattr(from_dirid).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(_) => nfs3::post_op_attr::Void,
    };
    let post_to_dir_attr = match context.vfs.getattr(to_dirid).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(_) => nfs3::post_op_attr::Void,
    };
    let from_wcc_res = nfs3::wcc_data {
        before: pre_from_dir_attr,
        after: post_from_dir_attr,
    };

    let to_wcc_res = nfs3::wcc_data {
        before: pre_to_dir_attr,
        after: post_to_dir_attr,
    };

    match res {
        Ok(()) => {
            debug!("rename success");
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            from_wcc_res.serialize(output)?;
            to_wcc_res.serialize(output)?;
        }
        Err(e) => {
            error!("rename error {:?} --> {:?}", xid, e);
            // serialize CREATE3resfail
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            e.serialize(output)?;
            from_wcc_res.serialize(output)?;
            to_wcc_res.serialize(output)?;
        }
    }

    Ok(())
}
