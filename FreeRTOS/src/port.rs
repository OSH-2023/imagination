// use crate::bindings::*;
// use crate::projdefs::FreeRtosError;
use std::ffi::c_void;


use log::{error, warn, info, debug};
use crate::projdefs::{FreeRtosError};

pub type StackType = usize;
pub type BaseType = i64;
pub type UBaseType = u64;
pub type TickType = u32;
pub type CVoidPointer = *mut std::os::raw::c_void;


pub type StackType_t= usize;
pub type BaseType_t = i64;
pub type UBaseType_t = u64;
pub type TickType_t = u32;

#[cfg(target_arch = "x86_64")]
pub const portBYTE_ALIGNMENT_MASK: UBaseType = 8;
#[cfg(not(target_arch = "x86_64"))]
pub const portBYTE_ALIGNMENT_MASK: UBaseType = 4;

#[cfg(feature = "configUSE_16_BIT_TICKS")]
pub const portMAX_DELAY: TickType = 0xffff;
#[cfg(not(feature = "configUSE_16_BIT_TICKS"))]
pub const portMAX_DELAY: TickType = 0xffffffff;

#[macro_export]
macro_rules! portYIELD {
    () => {
        // unsafe { crate::bindings::vPortYield() }
    };
}

#[macro_export]
macro_rules! portYIELD_WITHIN_API {
    () => {
        portYIELD!()
    };
}

#[macro_export]
macro_rules! portEND_SWITCHING_ISR {
    ($xSwitchRequired: expr) => {
        if $xSwitchRequired {
            unsafe {
                // crate::bindings::vPortYieldFromISR();
            }
        }
    };
}

#[macro_export]
macro_rules! portYIELD_FROM_ISR {
    ($xSwitchRequired: expr) => {
        // unsafe { portEND_SWITCHING_ISR($xSwitchRequired) }
    };
}

#[macro_export]
macro_rules! portSET_INTERRUPT_MASK_FROM_ISR {
    () => {
        port_initialize_blocks();
        // unsafe { (crate::bindings::xPortSetInterruptMask() as BaseType) }
    };
}

#[macro_export]
macro_rules! portCLEAR_INTERRUPT_MASK_FROM_ISR {
    ($xMask: expr) => {
        // unsafe { crate::bindings::vPortClearInterruptMask($xMask as BaseType) }
    };
}

#[macro_export]
macro_rules! portSET_INTERRUPT_MASK {
    () => {
        // unsafe { crate::bindings::vPortDisableInterrupts() }
    };
}

#[macro_export]
macro_rules! portCLEAR_INTERRUPT_MASK {
    () => {
        // unsafe { crate::bindings::vPortEnableInterrupts() }
    };
}

#[macro_export]
macro_rules! portDISABLE_INTERRUPTS {
    () => {
        // unsafe { portSET_INTERRUPT_MASK!() }
    };
}

#[macro_export]
macro_rules! portENABLE_INTERRUPTS {
    () => {
        // unsafe { portCLEAR_INTERRUPT_MASK() }
    };
}

#[macro_export]
macro_rules! portENTER_CRITICAL {
    () => {
        // unsafe { crate::bindings::vPortEnterCritical() }
    };
}

#[macro_export]
macro_rules! portEXIT_CRITICAL {
    () => {
        // unsafe { crate::bindings::vPortExitCritical() }
        port_initialize_blocks();
    };
}

#[macro_export]
macro_rules! portNOP {
    () => {
        // This is an empty function.
    };
}

#[macro_export]
macro_rules! traceTASK_DELETE {
    ($pxTaskToDelete: expr) => {
        // unsafe {
        //     // bindings::vPortForciblyEndThread(std::sync::Arc::into_raw($pxTaskToDelete) as *mut _)
        // }
    };
}

#[macro_export]
macro_rules! traceTASK_CREATE {
    ($pxTaskHandle: expr) => {
        // unsafe {
        //     info!("Task creation accomplished.");
        //     // bindings::vPortAddTaskHandle($pxTaskHandle.as_raw())
        // }
    };
}

#[macro_export]
macro_rules! portCONFIGURE_TIMER_FOR_RUN_TIME_STATS {
    () => {
        // unsafe { crate::bindings::vPortFindTicksPerSecond() }
    };
}

#[macro_export]
macro_rules! portGET_RUN_TIME_COUNTER_VALUE {
    () => {
        // unsafe { crate::bindings::ulPortGetTimerValue() }
    };
}

#[macro_export]
macro_rules! portTICK_PERIOS_MS {
    () => {
        // 1000 as TickType / config::configTICK_RATE_HZ!()
    };
}

#[macro_export]
macro_rules! portCLEAN_UP_TCB {
    ($pxTCB: expr) => {
        $pxTCB
    };
}

#[macro_export]
macro_rules! portPRE_TASK_DELETE_HOOK {
    ($pvTaskToDelete:expr, $pxYieldPending: expr) => {};
}

#[macro_export]
macro_rules! portSETUP_TCB {
    ($pxTCB:expr) => {
        $pxTCB
    };
}

#[macro_export]
macro_rules! portSUPPRESS_TICKS_AND_SLEEP {
    ($xExpectedIdleTime:expr) => {};
}

#[macro_export]
macro_rules! portTASK_USES_FLOATING_POINT {
    () => {};
}

#[macro_export]
macro_rules! portASSERT_IF_INTERRUPT_PRIORITY_INVALID {
    () => {};
}

#[macro_export]
macro_rules! portASSERT_IF_IN_ISR {
    () => {};
}

#[macro_export]
macro_rules! portRESET_READY_PRIORITY {
    ($uxPriority: expr, $uxTopReadyPriority: expr) => {

    };
}


 pub fn port_malloc(size: usize)-> Result<CVoidPointer, FreeRtosError> {
     unsafe {
        // let ret_ptr = pvPortMalloc(size);
        // if ret_ptr.is_null() {
        let mut ret_ptr: *mut c_void = std::ptr::null_mut();
        if size == 0{    
           error!("Malloc returned null.");
            Err(FreeRtosError::OutOfMemory)
        } else {
            Ok(ret_ptr)
    }
 }
}

pub fn port_free(pv: *mut ::std::os::raw::c_void) {
    // unsafe { vPortFree(pv) }
}

pub fn port_initialize_blocks() {
        // unsafe {
            // vPortInitialiseBlocks()
        // }
}

pub fn port_get_free_heap_size() -> usize{
        // unsafe {
        //     xPortGetFreeHeapSize()
        // }
        0
}

pub fn port_get_minimum_ever_free_heap_size() -> usize {
        // unsafe {
        //     xPortGetMinimumEverFreeHeapSize()
        // }
        0
}


pub fn port_start_scheduler()
  -> BaseType 
 {
    //  unsafe { xPortStartScheduler() }
    0
}

pub fn port_end_scheduler() {
    //  unsafe { vPortEndScheduler() }
}

pub fn port_initialise_stack(
    pxTopOfStack: *mut StackType,
    // pxCode: TaskFunction_t,
    pxCode: StackType,
    pvParameters: *mut ::std::os::raw::c_void,
) -> Result<*mut StackType, FreeRtosError> {
    // let ret_val = unsafe { pxPortInitialiseStack(pxTopOfStack, pxCode, pvParameters) };
    let num : usize = 0;
    let mut ret_val: *mut usize = std::ptr::null_mut();
    if ret_val.is_null() {
        error!("Port failed to initialise task stack!");
        Err(FreeRtosError::PortError)
    } else {
        Ok(ret_val)
    }
}