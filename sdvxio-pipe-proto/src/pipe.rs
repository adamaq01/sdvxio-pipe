use std::io::{Read, Write};

pub struct Sender<W: Write, T> {
    ipc: W,
    phantom: std::marker::PhantomData<T>,
}

pub struct Receiver<R: Read, T> {
    ipc: R,
    phantom: std::marker::PhantomData<T>,
}

impl<W: Write, T: serde::Serialize> Sender<W, T> {
    pub fn new(ipc: W) -> Self {
        Self {
            ipc,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, msg: &T) -> std::io::Result<()> {
        let data = postcard::to_allocvec(msg).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Serialization error: {}", err),
            )
        })?;
        self.ipc.write(data.as_slice())?;
        self.ipc.flush()?;
        Ok(())
    }
}

impl<R: Read, T: serde::de::DeserializeOwned> Receiver<R, T> {
    pub fn new(ipc: R) -> Self {
        Self {
            ipc,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn recv(&mut self) -> std::io::Result<T> {
        let mut buffer = vec![0u8; 128];
        let (msg, _): (T, _) = postcard::from_io((&mut self.ipc, &mut buffer)).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Deserialization error: {}", err),
            )
        })?;
        Ok(msg)
    }
}
