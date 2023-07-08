#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod config;
#[macro_use]
pub mod global;
pub mod kernel;
pub mod list;
#[macro_use]
pub mod port;
pub mod projdefs;
pub mod queue_api;
pub mod queue;
pub mod queue_h;
pub mod semaphore;
pub mod task_queue;
pub mod tasks;
#[macro_use]
pub mod trace;

use log::{error, warn, info, debug, trace};

// use crate::port::UBaseType_t;
// // use crate::projdefs::pdFALSE;
// use crate::tasks::{TaskHandle, TCB};
// use crate::global::*;
// use crate::port::{TickType_t, UBaseType_t, BaseType_t, StackType_t};
// pub type CVoidPointer = *mut std::os::raw::c_void;
use std::boxed::*;
use std::mem;
use std::sync::{Arc, RwLock, Weak};

fn main() {
    println!("Hello, world!");
}

