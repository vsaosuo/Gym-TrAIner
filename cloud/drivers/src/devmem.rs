use tokio::{fs::File, io};

pub struct DevMem(pub(super) File);

impl DevMem {
    pub async fn new() -> io::Result<Self> {
        Ok(Self(
            tokio::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_SYNC)
                .open("/dev/mem")
                .await?,
        ))
    }
}
