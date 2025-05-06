use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};

pub async fn nfsproc3_commit(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut args = nfs3::COMMIT3args::default();
    args.deserialize(input)?;
    debug!("nfsproc3_commit({:?}, {:?}) ", xid, args);

    let id = context.vfs.fh_to_id(&args.file);
    // fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::wcc_data::default().serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();

    // get the object attributes before the commit
    let pre_obj_attr = match context.vfs.getattr(id).await {
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

    // Call VFS commit method
    match context.vfs.commit(id, args.offset, args.count).await {
        Ok(fattr) => {
            let post_obj_attr = nfs3::post_op_attr::attributes(fattr);

            let res = nfs3::COMMIT3resok {
                file_wcc: nfs3::wcc_data {
                    before: pre_obj_attr,
                    after: post_obj_attr,
                },
                verf: context.vfs.serverid(),
            };

            debug!("nfsproc3_commit success");
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3_OK.serialize(output)?;
            res.serialize(output)?;
        }
        Err(stat) => {
            let post_obj_attr = match context.vfs.getattr(id).await {
                Ok(v) => nfs3::post_op_attr::attributes(v),
                Err(_) => nfs3::post_op_attr::Void,
            };

            let wcc_data = nfs3::wcc_data {
                before: pre_obj_attr,
                after: post_obj_attr,
            };

            debug!("nfsproc3_commit error: {:?}", stat);
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            wcc_data.serialize(output)?;
        }
    }

    Ok(())
}
