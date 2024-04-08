// SPDX-License-Identifier: GPL-2.0

//! Rust completion device

use core::cell::UnsafeCell;
use core::ops::Deref;
use kernel::prelude::*;
use kernel::sync::Mutex;
use kernel::task::Task;
use kernel::{bindings, chrdev, file};

module! {
    type: Completiondev,
    name: "completion",
    author: "maoyutofu",
    description: "Completion device",
    license: "GPL",
}

struct Completion(UnsafeCell<bindings::completion>);

impl Deref for Completion {
    type Target = UnsafeCell<bindings::completion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for Completion {}

static COMPLETION: Mutex<Option<Pin<Box<Completion>>>> = unsafe { Mutex::new(None) };

struct CompletionFile {}

#[vtable]
impl file::Operations for CompletionFile {
    type Data = ();

    fn open(_context: &(), _file: &file::File) -> Result<()> {
        pr_info!("function open is invoked");
        Ok(())
    }

    fn read(
        _data: (),
        _file: &file::File,
        _writer: &mut impl kernel::io_buffer::IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read is invoked\n");
        pr_info!("process {} is going to sleep\n", Task::current().pid());
        let lock = COMPLETION.lock();
        let completion = lock.deref().as_ref().unwrap().get();
        drop(lock);
        unsafe { bindings::wait_for_completion(completion) };
        pr_info!("awoken {}\n", Task::current().pid());
        Ok(0)
    }

    fn write(
        _data: (),
        _file: &file::File,
        reader: &mut impl kernel::io_buffer::IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("write is invoked\n");
        pr_info!(
            "process {} awakening the readers...\n",
            Task::current().pid()
        );
        let lock = COMPLETION.lock();
        let completion = lock.deref().as_ref().unwrap().get();
        drop(lock);
        unsafe { bindings::complete(completion) };
        pr_info!("write success\n");
        Ok(reader.len())
    }
}

struct Completiondev {
    _dev: Pin<Box<chrdev::Registration<1>>>,
}

impl kernel::Module for Completiondev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Completiondev is loaded\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        let mut completion = COMPLETION.lock();
        let inner = Pin::new(Box::try_new(Completion(UnsafeCell::new(
            bindings::completion::default(),
        )))?);
        unsafe { bindings::init_completion(inner.get()) };
        *completion = Some(inner);

        chrdev_reg.as_mut().register::<CompletionFile>()?;

        Ok(Completiondev { _dev: chrdev_reg })
    }
}

impl Drop for Completiondev {
    fn drop(&mut self) {
        pr_info!("drop completion device\n");
    }
}
