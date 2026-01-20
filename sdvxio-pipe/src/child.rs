use crate::error::Error;
use sdvxio_pipe_proto::{ChildToParent, Message, ParentToChild, Receiver, Sender};
use std::process::{ChildStdin, ChildStdout};

pub struct ChildSdvxIo {
    pub child: std::process::Child,
    pub tx: Sender<ChildStdin, Message<ParentToChild>>,
    pub rx: Receiver<ChildStdout, Message<ChildToParent>>,
}

impl ChildSdvxIo {
    pub(crate) fn request(&mut self, msg: ParentToChild) -> Result<ChildToParent, Error> {
        let message = Message::new(msg);
        let id = message.id;
        self.tx.send(&message)?;
        let response = self.rx.recv()?;
        if response.id != id {
            Err(Error::WrongResponseId {
                expected: id as u64,
                got: response.id as u64,
            })
        } else {
            Ok(response.payload)
        }
    }
}
