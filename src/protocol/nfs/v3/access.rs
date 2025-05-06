use std::io::{Read, Write};

use tracing::debug;

use crate::protocol::rpc;
use crate::protocol::xdr::{self, nfs3, XDR};
use crate::vfs;

pub async fn nfsproc3_access(
    xid: u32,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &rpc::Context,
) -> Result<(), anyhow::Error> {
    let mut handle = nfs3::nfs_fh3::default();
    handle.deserialize(input)?;
    let mut access: u32 = 0;
    access.deserialize(input)?;
    debug!("nfsproc3_access({:?},{:?},{:?})", xid, handle, access);

    let id = context.vfs.fh_to_id(&handle);
    // Fail if unable to convert file handle
    if let Err(stat) = id {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        stat.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        return Ok(());
    }
    let id = id.unwrap();

    // Get object attributes
    let obj_attr = match context.vfs.getattr(id).await {
        Ok(v) => nfs3::post_op_attr::attributes(v),
        Err(stat) => {
            // If we can't get attributes, return an error
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            stat.serialize(output)?;
            nfs3::post_op_attr::Void.serialize(output)?;
            return Ok(());
        }
    };

    // Check if the object exists
    if let nfs3::post_op_attr::Void = obj_attr {
        xdr::rpc::make_success_reply(xid).serialize(output)?;
        nfs3::nfsstat3::NFS3ERR_NOENT.serialize(output)?;
        nfs3::post_op_attr::Void.serialize(output)?;
        return Ok(());
    }

    // Extract object attributes
    let attr = match obj_attr {
        nfs3::post_op_attr::attributes(attr) => attr,
        _ => {
            // This should not happen, since we already checked that obj_attr is not Void
            xdr::rpc::make_success_reply(xid).serialize(output)?;
            nfs3::nfsstat3::NFS3ERR_SERVERFAULT.serialize(output)?;
            nfs3::post_op_attr::Void.serialize(output)?;
            return Ok(());
        }
    };

    // Check access permissions based on file type and attributes
    let mut granted_access = 0;

    // Always allow LOOKUP for existing objects
    granted_access |= nfs3::ACCESS3_LOOKUP;

    // Check permissions based on file type
    match attr.ftype {
        nfs3::ftype3::NF3REG => {
            // For regular files
            if !matches!(context.vfs.capabilities(), vfs::Capabilities::ReadWrite) {
                // If the file system is read-only, allow only reading
                if access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE) != 0 {
                    granted_access |= access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE);
                }
            } else {
                // If the file system supports read and write, check access permissions
                // Here you can add a check for real access permissions based on file attributes
                // For example, check owner, group, and access permissions

                // For simplicity, allow all requested permissions
                granted_access |= access;
            }
        }
        nfs3::ftype3::NF3DIR => {
            // For directories
            if !matches!(context.vfs.capabilities(), vfs::Capabilities::ReadWrite) {
                // If the file system is read-only, allow only reading
                if access & nfs3::ACCESS3_READ != 0 {
                    granted_access |= nfs3::ACCESS3_READ;
                }
            } else {
                // If the file system supports read and write, check access permissions
                // For directories, allow reading and execution
                if access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE) != 0 {
                    granted_access |= access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE);
                }

                // For operations that modify the directory, check write permissions
                if access & (nfs3::ACCESS3_MODIFY | nfs3::ACCESS3_EXTEND | nfs3::ACCESS3_DELETE)
                    != 0
                {
                    // Here you can add a check for real access permissions
                    granted_access |= access
                        & (nfs3::ACCESS3_MODIFY | nfs3::ACCESS3_EXTEND | nfs3::ACCESS3_DELETE);
                }
            }
        }
        nfs3::ftype3::NF3LNK => {
            // For symbolic links, allow only reading
            if access & nfs3::ACCESS3_READ != 0 {
                granted_access |= nfs3::ACCESS3_READ;
            }
        }
        _ => {
            // For other file types (devices, sockets, etc.)
            // Allow only reading and execution
            if access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE) != 0 {
                granted_access |= access & (nfs3::ACCESS3_READ | nfs3::ACCESS3_EXECUTE);
            }
        }
    }

    debug!(" {:?} ---> {:?}", xid, granted_access);
    xdr::rpc::make_success_reply(xid).serialize(output)?;
    nfs3::nfsstat3::NFS3_OK.serialize(output)?;
    obj_attr.serialize(output)?;
    granted_access.serialize(output)?;
    Ok(())
}
