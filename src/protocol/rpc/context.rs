use std::fmt;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::protocol::xdr;
use crate::vfs;

#[derive(Clone)]
pub struct Context {
    pub local_port: u16,
    pub client_addr: String,
    pub auth: xdr::rpc::auth_unix,
    pub vfs: Arc<dyn vfs::NFSFileSystem + Send + Sync>,
    pub mount_signal: Option<mpsc::Sender<bool>>,
    pub export_name: Arc<String>,
    pub transaction_tracker: Arc<super::TransactionTracker>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("rpc::Context")
            .field("local_port", &self.local_port)
            .field("client_addr", &self.client_addr)
            .field("auth", &self.auth)
            .finish()
    }
}
