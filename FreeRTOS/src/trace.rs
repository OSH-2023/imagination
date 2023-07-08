use log::{error, warn, info, debug};
use crate::queue::*;
use crate::queue_h::*;
use crate::queue_api::*;
use crate::semaphore::*;
use crate::task_queue::*;

#[macro_export]
macro_rules! traceSTART {
    () => {
        trace!("Trace starts");
    };
}

#[macro_export]
macro_rules! traceEND {
    () => {
        trace!("Trace ends");
    };
}

#[macro_export]
macro_rules! traceTASK_SWITCHED_IN {
    () => {
        if let Some(current_task) = GetCurrentTaskHandleWrapped!() {
            trace!("Task {} switched in", current_task.GetName());
        } else {
            warn!("No task switched in");
        }
    };
}

#[macro_export]
macro_rules! traceINCREASE_TICK_COUNT {
    ($x: expr) => {
        trace!("Tick count increased from {}", $x);
    };
}

#[macro_export]
macro_rules! traceLOW_POWER_IDLE_BEGIN {
    () => {};
}

#[macro_export]
macro_rules! traceLOW_POWER_IDLE_END {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_SWITCHED_OUT {
    () => {
        if let Some(current_task) = GetCurrentTaskHandleWrapped!() {
            trace!("Task {} will be switched out", current_task.GetName());
        }
    };
}

#[macro_export]
macro_rules! traceTASK_PRIORITY_INHERIT {
    ($pxTCBOfMutexHolder: expr, $uxInheritedPriority: expr) => {};
}

macro_rules! traceTASK_PRIORITY_DISINHERIT {
    ($pxTCBOfMutexHolder: expr, $uxOriginalPriority: expr) => {};
}

// #[macro_export]
// macro_rules! traceBLOCKING_ON_QUEUE_RECEIVE {
//     ($pxQueue: expr) => {
//         trace!(
//             "Blocking task {} because it cannot read from {}.",
//             GetCurrentTaskHandle!().GetName(),
//             $pxQueue.uxQueueNumber
//         );
//     };
// }

// #[macro_export]
// macro_rules! traceBLOCKING_ON_QUEUE_SEND {
//     ($pxQueue: expr) => {
//         trace!(
//             "Blocking task {} because it cannot write to {}.",
//             GetCurrentTaskHandle!().GetName(),
//             $pxQueue.get_queue_number()
//         );
//     };
// }

#[macro_export]
macro_rules! traceMOVED_TASK_TO_READY_STATE {
    ($pxTCB: expr) => {
        trace!("Moving task {} to ready state.", $pxTCB.GetName());
    };
}

#[macro_export]
macro_rules! tracePOST_MOVED_TASK_TO_READY_STATE {
    ($pxTCB: expr) => {
        trace!("Task {} was moved to ready state.", $pxTCB.GetName());
    };
}

// #[macro_export]
// macro_rules! traceQUEUE_CREATE {
//     ($pxNewQueue: expr) => {
//         trace!("Created queue {}", $pxNewQueue.get_queue_number());
//     };
// }

#[macro_export]
macro_rules! traceQUEUE_CREATE_FAILED {
    ($ucQueueType: expr) => {
        warn!("Queue creation failed.");
    };
}

// #[macro_export]
// macro_rules! traceCREATE_MUTEX {
//     ($pxNewQueue: expr) => {
//         trace!("Created mutex {}", $pxNewQueue.0.get_queue_number());
//     };
// }

#[macro_export]
macro_rules! traceCREATE_MUTEX_FAILED {
    () => {
        warn!("Mutex creation failed.");
    };
}

#[macro_export]
macro_rules! traceGIVE_MUTEX_RECURSIVE {
    ($pxMutex: expr) => {};
}

#[macro_export]
macro_rules! traceGIVE_MUTEX_RECURSIVE_FAILED {
    ($pxMutex: expr) => {};
}

#[macro_export]
macro_rules! traceTAKE_MUTEX_RECURSIVE {
    ($pxMutex: expr) => {};
}

#[macro_export]
macro_rules! traceTAKE_MUTEX_RECURSIVE_FAILED {
    ($pxMutex: expr) => {};
}

#[macro_export]
macro_rules! traceCREATE_COUNTING_SEMAPHORE {
    () => {
        trace!("Created counting semaphore: {}");
    };
}

#[macro_export]
macro_rules! traceCREATE_COUNTING_SEMAPHORE_FAILED {
    () => {
        trace!("Counting semaphore creation failed.");
    };
}

// #[macro_export]
// macro_rules! traceQUEUE_SEND {
//     ($pxQueue: expr) => {
//         trace!("Sending to queue {}", $pxQueue.get_queue_number());
//     };
// }

#[macro_export]
macro_rules! traceQUEUE_SEND_FAILED {
    ($pxQueue: expr) => {
        warn!("Queue send failed!");
    };
}

// #[macro_export]
// macro_rules! traceQUEUE_RECEIVE {
//     ($pxQueue: expr) => {
//         trace!("Receiving from queue {}", $pxQueue.get_queue_number());
//     };
// }

// #[macro_export]
// macro_rules! traceQUEUE_PEEK {
//     ($pxQueue: expr) => {
//         trace!("Peeking from queue {}", $pxQueue.get_queue_number());
//     };
// }

#[macro_export]
macro_rules! traceQUEUE_PEEK_FROM_ISR {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_RECEIVE_FAILED {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_SEND_FROM_ISR {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_SEND_FROM_ISR_FAILED {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_RECEIVE_FROM_ISR {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_RECEIVE_FROM_ISR_FAILED {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_PEEK_FROM_ISR_FAILED {
    ($pxQueue: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_DELETE {
    ($pxQueue: expr) => {
        trace!("Deleting queue {}", pxQueue.get_queue_number());
    };
}

#[macro_export]
macro_rules! traceTASK_CREATE_FAILED {
    () => {
        warn!("Task creation failed!");
    };
}

#[macro_export]
macro_rules! traceTASK_DELAY_UNTIL {
    ($x: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_DELAY {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_PRIORITY_SET {
    ($pxTask: expr, $uxNewPriority: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_SUSPEND {
    ($pxTaskToSuspend: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_RESUME {
    ($pxTaskToResume: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_RESUME_FROM_ISR {
    ($pxTaskToResume: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_INCREMENT_TICK {
    ($xTickCount: expr) => {};
}

#[macro_export]
macro_rules! traceTIMER_CREATE {
    ($pxNewTimer: expr) => {};
}

#[macro_export]
macro_rules! traceTIMER_CREATE_FAILED {
    () => {};
}

#[macro_export]
macro_rules! traceTIMER_COMMAND_SEND {
    ($xTimer: expr, $xMessageID: expr, $xMessageValueValue: expr, $xReturn: expr) => {};
}

#[macro_export]
macro_rules! traceTIMER_EXPIRED {
    ($pxTimer: expr) => {};
}

#[macro_export]
macro_rules! traceTIMER_COMMAND_RECEIVED {
    ($pxTimer: expr, $xMessageID: expr, $xMessageValue: expr) => {};
}

#[macro_export]
macro_rules! traceMALLOC {
    ($pvAddress: expr, $uiSize: expr) => {};
}

#[macro_export]
macro_rules! traceFREE {
    ($pvAddress: expr, $uiSize: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_CREATE {
    ($xEventGroup: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_CREATE_FAILED {
    () => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_SYNC_BLOCK {
    ($xEventGroup: expr, $uxBitsToSet: expr, $uxBitsToWaitFor: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_SYNC_END {
    ($xEventGroup: expr, $uxBitsToSet: expr, $uxBitsToWaitFor: expr, $xTimeoutOccurred: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_WAIT_BITS_BLOCK {
    ($xEventGroup: expr, $uxBitsToWaitFor: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_WAIT_BITS_END {
    ($xEventGroup: expr, $uxBitsToWaitFor: expr, $xTimeoutOccurred: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_CLEAR_BITS {
    ($xEventGroup: expr, $uxBitsToClear: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_CLEAR_BITS_FROM_ISR {
    ($xEventGroup: expr, $uxBitsToClear: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_SET_BITS {
    ($xEventGroup: expr, $uxBitsToSet: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_SET_BITS_FROM_ISR {
    ($xEventGroup: expr, $uxBitsToSet: expr) => {};
}

#[macro_export]
macro_rules! traceEVENT_GROUP_DELETE {
    ($xEventGroup: expr) => {};
}

#[macro_export]
macro_rules! tracePEND_FUNC_CALL {
    ($xFunctionToPend: expr, $pvParameter1: expr, $ulParameter2: expr, $ret: expr) => {};
}

#[macro_export]
macro_rules! tracePEND_FUNC_CALL_FROM_ISR {
    ($xFunctionToPend: expr, $pvParameter1: expr, $ulParameter2: expr, $ret: expr) => {};
}

#[macro_export]
macro_rules! traceQUEUE_REGISTRY_ADD {
    ($xQueue: expr, $pcQueueName: expr) => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_TAKE_BLOCK {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_TAKE {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_WAIT_BLOCK {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_WAIT {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_FROM_ISR {
    () => {};
}

#[macro_export]
macro_rules! traceTASK_NOTIFY_GIVE_FROM_ISR {
    () => {};
}

#[macro_export]
macro_rules! mtCOVERAGE_TEST_MARKER {
    () => {};
}

#[macro_export]
macro_rules! mtCOVERAGE_TEST_DELAY {
    () => {};
}

#[macro_export]
macro_rules! DebugPrint {
    () => {};
}

