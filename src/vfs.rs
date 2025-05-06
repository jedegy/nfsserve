use std::cmp::Ordering;
use std::sync::Once;
use std::time::SystemTime;

use async_trait::async_trait;

use crate::protocol::xdr::nfs3;

#[derive(Default, Debug)]
pub struct DirEntrySimple {
    pub fileid: nfs3::fileid3,
    pub name: nfs3::filename3,
}

#[derive(Default, Debug)]
pub struct ReadDirSimpleResult {
    pub entries: Vec<DirEntrySimple>,
    pub end: bool,
}

#[derive(Default, Debug)]
pub struct DirEntry {
    pub fileid: nfs3::fileid3,
    pub name: nfs3::filename3,
    pub attr: nfs3::fattr3,
}

#[derive(Default, Debug)]
pub struct ReadDirResult {
    pub entries: Vec<DirEntry>,
    pub end: bool,
}

impl ReadDirSimpleResult {
    fn from_readdir_result(result: &ReadDirResult) -> ReadDirSimpleResult {
        let entries: Vec<DirEntrySimple> = result
            .entries
            .iter()
            .map(|e| DirEntrySimple {
                fileid: e.fileid,
                name: e.name.clone(),
            })
            .collect();
        ReadDirSimpleResult {
            entries,
            end: result.end,
        }
    }
}

static mut GENERATION_NUMBER: u64 = 0;
static GENERATION_NUMBER_INIT: Once = Once::new();

fn get_generation_number() -> u64 {
    unsafe {
        GENERATION_NUMBER_INIT.call_once(|| {
            GENERATION_NUMBER = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        });
        GENERATION_NUMBER
    }
}

/// What capabilities are supported
pub enum Capabilities {
    ReadOnly,
    ReadWrite,
}

/// The basic API to implement to provide an NFS file system
///
/// Opaque FH
/// ---------
/// Files are only uniquely identified by a 64-bit file id. (basically an inode number)
/// We automatically produce internally the opaque filehandle which is comprised of
///  - A 64-bit generation number derived from the server startup time
///   (i.e. so the opaque file handle expires when the NFS server restarts)
///  - The 64-bit file id
//
/// readdir pagination
/// ------------------
/// We do not use cookie verifier. We just use the start_after.  The
/// implementation should allow startat to start at any position. That is,
/// the next query to readdir may be the last entry in the previous readdir
/// response.
//
/// There is a wierd annoying thing about readdir that limits the number
/// of bytes in the response (instead of the number of entries). The caller
/// will have to truncate the readdir response / issue more calls to readdir
/// accordingly to fill up the expected number of bytes without exceeding it.
//
/// Other requirements
/// ------------------
///  getattr needs to be fast. NFS uses that a lot
//
///  The 0 fileid is reserved and should not be used
///
#[async_trait]
pub trait NFSFileSystem: Sync {
    /// Returns the set of capabilities supported
    fn capabilities(&self) -> Capabilities;
    /// Returns the ID the of the root directory "/"
    fn root_dir(&self) -> nfs3::fileid3;
    /// Look up the id of a path in a directory
    ///
    /// i.e. given a directory dir/ containing a file a.txt
    /// this may call lookup(id_of("dir/"), "a.txt")
    /// and this should return the id of the file "dir/a.txt"
    ///
    /// This method should be fast as it is used very frequently.
    async fn lookup(
        &self,
        dirid: nfs3::fileid3,
        filename: &nfs3::filename3,
    ) -> Result<nfs3::fileid3, nfs3::nfsstat3>;

    /// Returns the attributes of an id.
    /// This method should be fast as it is used very frequently.
    async fn getattr(&self, id: nfs3::fileid3) -> Result<nfs3::fattr3, nfs3::nfsstat3>;

    /// Sets the attributes of an id
    /// this should return Err(nfsstat3::NFS3ERR_ROFS) if readonly
    async fn setattr(
        &self,
        id: nfs3::fileid3,
        setattr: nfs3::sattr3,
    ) -> Result<nfs3::fattr3, nfs3::nfsstat3>;

    /// Reads the contents of a file returning (bytes, EOF)
    /// Note that offset/count may go past the end of the file and that
    /// in that case, all bytes till the end of file are returned.
    /// EOF must be flagged if the end of the file is reached by the read.
    async fn read(
        &self,
        id: nfs3::fileid3,
        offset: u64,
        count: u32,
    ) -> Result<(Vec<u8>, bool), nfs3::nfsstat3>;

    /// Writes the contents of a file returning (bytes, EOF)
    /// Note that offset/count may go past the end of the file and that
    /// in that case, the file is extended.
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn write(
        &self,
        id: nfs3::fileid3,
        offset: u64,
        data: &[u8],
    ) -> Result<nfs3::fattr3, nfs3::nfsstat3>;

    /// Creates a file with the following attributes.
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn create(
        &self,
        dirid: nfs3::fileid3,
        filename: &nfs3::filename3,
        attr: nfs3::sattr3,
    ) -> Result<(nfs3::fileid3, nfs3::fattr3), nfs3::nfsstat3>;

    /// Creates a file if it does not already exist
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn create_exclusive(
        &self,
        dirid: nfs3::fileid3,
        filename: &nfs3::filename3,
    ) -> Result<nfs3::fileid3, nfs3::nfsstat3>;

    /// Makes a directory with the following attributes.
    /// If not supported dur to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn mkdir(
        &self,
        dirid: nfs3::fileid3,
        dirname: &nfs3::filename3,
    ) -> Result<(nfs3::fileid3, nfs3::fattr3), nfs3::nfsstat3>;

    /// Removes a file.
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn remove(
        &self,
        dirid: nfs3::fileid3,
        filename: &nfs3::filename3,
    ) -> Result<(), nfs3::nfsstat3>;

    /// Removes a file.
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn rename(
        &self,
        from_dirid: nfs3::fileid3,
        from_filename: &nfs3::filename3,
        to_dirid: nfs3::fileid3,
        to_filename: &nfs3::filename3,
    ) -> Result<(), nfs3::nfsstat3>;

    /// Returns the contents of a directory with pagination.
    /// Directory listing should be deterministic.
    /// Up to max_entries may be returned, and start_after is used
    /// to determine where to start returning entries from.
    ///
    /// For instance if the directory has entry with ids [1,6,2,11,8,9]
    /// and start_after=6, readdir should returning 2,11,8,...
    //
    async fn readdir(
        &self,
        dirid: nfs3::fileid3,
        start_after: nfs3::fileid3,
        max_entries: usize,
    ) -> Result<ReadDirResult, nfs3::nfsstat3>;

    /// Simple version of readdir.
    /// Only need to return filename and id
    async fn readdir_simple(
        &self,
        dirid: nfs3::fileid3,
        count: usize,
    ) -> Result<ReadDirSimpleResult, nfs3::nfsstat3> {
        Ok(ReadDirSimpleResult::from_readdir_result(
            &self.readdir(dirid, 0, count).await?,
        ))
    }

    /// Makes a symlink with the following attributes.
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn symlink(
        &self,
        dirid: nfs3::fileid3,
        linkname: &nfs3::filename3,
        symlink: &nfs3::nfspath3,
        attr: &nfs3::sattr3,
    ) -> Result<(nfs3::fileid3, nfs3::fattr3), nfs3::nfsstat3>;

    /// Reads a symlink
    async fn readlink(&self, id: nfs3::fileid3) -> Result<nfs3::nfspath3, nfs3::nfsstat3>;

    /// Creates a hard link
    /// If not supported due to readonly file system or other reasons
    /// this should return Err(nfsstat3::NFS3ERR_ROFS) or Err(nfsstat3::NFS3ERR_NOTSUPP)
    async fn link(
        &self,
        file_id: nfs3::fileid3,
        link_dir_id: nfs3::fileid3,
        link_name: &nfs3::filename3,
    ) -> Result<nfs3::fattr3, nfs3::nfsstat3>;

    /// Creates a special node (character device, block device, socket, or FIFO)
    /// If not supported due to readonly file system or other reasons
    /// this should return Err(nfsstat3::NFS3ERR_ROFS) or Err(nfsstat3::NFS3ERR_NOTSUPP)
    async fn mknod(
        &self,
        dir_id: nfs3::fileid3,
        name: &nfs3::filename3,
        ftype: nfs3::ftype3,
        specdata: nfs3::specdata3,
        attrs: &nfs3::sattr3,
    ) -> Result<(nfs3::fileid3, nfs3::fattr3), nfs3::nfsstat3>;

    /// Commits data written to a file to stable storage
    /// If not supported due to readonly file system
    /// this should return Err(nfsstat3::NFS3ERR_ROFS)
    async fn commit(
        &self,
        file_id: nfs3::fileid3,
        offset: u64,
        count: u32,
    ) -> Result<nfs3::fattr3, nfs3::nfsstat3>;

    /// Get static file system Information
    async fn fsinfo(&self, root_fileid: nfs3::fileid3) -> Result<nfs3::fsinfo3, nfs3::nfsstat3> {
        let dir_attr: nfs3::post_op_attr = match self.getattr(root_fileid).await {
            Ok(v) => nfs3::post_op_attr::attributes(v),
            Err(_) => nfs3::post_op_attr::Void,
        };

        let res = nfs3::fsinfo3 {
            obj_attributes: dir_attr,
            rtmax: 1024 * 1024,
            rtpref: 1024 * 124,
            rtmult: 1024 * 1024,
            wtmax: 1024 * 1024,
            wtpref: 1024 * 1024,
            wtmult: 1024 * 1024,
            dtpref: 1024 * 1024,
            maxfilesize: 128 * 1024 * 1024 * 1024,
            time_delta: nfs3::nfstime3 {
                seconds: 0,
                nseconds: 1000000,
            },
            properties: nfs3::FSF_SYMLINK | nfs3::FSF_HOMOGENEOUS | nfs3::FSF_CANSETTIME,
        };
        Ok(res)
    }

    /// Converts the fileid to an opaque NFS file handle. Optional.
    fn id_to_fh(&self, id: nfs3::fileid3) -> nfs3::nfs_fh3 {
        let gennum = get_generation_number();
        let mut ret: Vec<u8> = Vec::new();
        ret.extend_from_slice(&gennum.to_le_bytes());
        ret.extend_from_slice(&id.to_le_bytes());
        nfs3::nfs_fh3 { data: ret }
    }
    /// Converts an opaque NFS file handle to a fileid.  Optional.
    fn fh_to_id(&self, id: &nfs3::nfs_fh3) -> Result<nfs3::fileid3, nfs3::nfsstat3> {
        if id.data.len() != 16 {
            return Err(nfs3::nfsstat3::NFS3ERR_BADHANDLE);
        }
        let gen = u64::from_le_bytes(id.data[0..8].try_into().unwrap());
        let id = u64::from_le_bytes(id.data[8..16].try_into().unwrap());
        let gennum = get_generation_number();
        match gen.cmp(&gennum) {
            Ordering::Less => Err(nfs3::nfsstat3::NFS3ERR_STALE),
            Ordering::Greater => Err(nfs3::nfsstat3::NFS3ERR_BADHANDLE),
            Ordering::Equal => Ok(id),
        }
    }
    /// Converts a complete path to a fileid.  Optional.
    /// The default implementation walks the directory structure with lookup()
    async fn path_to_id(&self, path: &[u8]) -> Result<nfs3::fileid3, nfs3::nfsstat3> {
        let splits = path.split(|&r| r == b'/');
        let mut fid = self.root_dir();
        for component in splits {
            if component.is_empty() {
                continue;
            }
            fid = self.lookup(fid, &component.into()).await?;
        }
        Ok(fid)
    }

    fn serverid(&self) -> nfs3::cookieverf3 {
        let gennum = get_generation_number();
        gennum.to_le_bytes()
    }
}
