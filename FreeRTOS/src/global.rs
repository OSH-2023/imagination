use crate::list::ListLink;
use crate::tasks::*;
use crate::*;

pub type BaseType = i64;
pub type UBaseType = u64;
pub type TickType = u32;

use std::sync::{Arc, RwLock, Weak};

//定义全局变量
pub static mut TICK_COUNT: TickType = 0;
pub static mut TOP_READY_PRIORITY: UBaseType = 0;//所有Ready任务中，最高的优先级
pub static mut PENDED_TICKS: UBaseType = 0;
pub static mut SCHEDULER_RUNNING: bool = false;
pub static mut YIELD_PENDING: bool = false;
pub static mut NUM_OF_OVERFLOWS: BaseType = 0;
pub static mut TASK_NUMBER: UBaseType = 0;
pub static mut NEXT_TASK_UNBLOCK_TIME: TickType = 0;
pub static mut CURRENT_NUMBER_OF_TASKS: UBaseType = 0;

use log::{error, warn, info, debug};

use crate::lazy_static::*;

//lazy_static! 是一个 Rust 宏，相比原始的在编译时初始化静态变量，它可以在运行时初始化静态变量
//需要在运行时才能初始化的静态变量包括任何需要堆分配的内容，如向量或哈希映射，以及任何需要函数调用才能计算的内容
lazy_static! {
    //当前任务的TCB
    pub static ref CURRENT_TCB: RwLock<Option<TaskHandle>> = RwLock::new(None);

    //Ready List，没什么好说的
    pub static ref READY_TASK_LISTS: [ListLink; configMAX_PRIORITIES!()] = Default::default();

   //分别用于存储延迟任务和溢出延迟任务
    pub static ref DELAYED_TASK_LIST: ListLink = Default::default();
    pub static ref OVERFLOW_DELAYED_TASK_LIST: ListLink = Default::default();

    //存储在调度程序挂起时已准备就绪的任务
    pub static ref PENDING_READY_LIST: ListLink = Default::default();
}

#[cfg(feature = "INCLUDE_vTaskDelete")]
lazy_static! {
    //用来保存已经被删除，但是内存没被释放的任务
    pub static ref TASKS_WAITING_TERMINATION: ListLink = Default::default();
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
lazy_static! {
    //挂起列表
    pub static ref SUSPENDED_TASK_LIST: ListLink = Default::default();
}
/* ------------------ End global lists ------------------- */

//初始化为 0,指示调度程序是否已挂起
//当调度程序挂起时，上下文切换将被挂起，
//并且中断不能操作 TCB 的 xStateListItem 或任何可以从中引用 xStateListItem 的列表
pub static mut SCHEDULER_SUSPENDED: UBaseType = 0;

//用于运行时间统计信息的静态变量
//保存上一次任务切换时计时器/计数器的值
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
pub static mut TASK_SWITCHED_IN_TIME: u32 = 0;

//保存运行时间计数器时钟定义的执行时间总量
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
pub static mut TOTAL_RUN_TIME: u32 = 0;

//存储等待清理的已删除任务的数量
#[cfg(feature = "INCLUDE_vTaskDelete")]
pub static mut DELETED_TASKS_WAITING_CLEAN_UP: UBaseType = 0;

//接下来的一系列宏用于通俗易懂的set和get，对以上全局变量进行操作
#[macro_export]
macro_rules! SetSchedulerSuspended {
    ($next_val: expr) => {
        unsafe {
            info!("SCHEDULER_SUSPENDED was set to {}", $next_val);
            crate::global::SCHEDULER_SUSPENDED = $next_val;
        }
    };
}

#[macro_export]
macro_rules! GetSchedulerSuspended {
    () => {
        unsafe { crate::global::SCHEDULER_SUSPENDED }
    };
}

#[macro_export]
macro_rules! SetDeletedTasksWaitingCleanUp {
    ($next_val: expr) => {
        unsafe {
            info!("DELETED_TASKS_WAITING_CLEAN_UP was set to {}", $next_val);
            crate::global::DELETED_TASKS_WAITING_CLEAN_UP = $next_val;
        }
    };
}

#[macro_export]
macro_rules! GetDeletedTasksWaitingCleanUp {
    () => {
        unsafe { crate::global::DELETED_TASKS_WAITING_CLEAN_UP }
    };
}

#[macro_export]
macro_rules! GetTopReadyPriority {
    () => {
        unsafe { crate::global::TOP_READY_PRIORITY }
    };
}

#[macro_export]
macro_rules! SetTopReadyPriority {
    ($new_top_ready_priority: expr) => {
        unsafe {
            info!("TOP_READY_PRIORITY was set to {}", $new_top_ready_priority);
            crate::global::TOP_READY_PRIORITY = $new_top_ready_priority;
        }
    };
}

#[macro_export]
macro_rules! SetPendedTicks {
    ($next_val: expr) => {
        unsafe {
            info!("PENDED_TICKS was set to {}", $next_val);
            crate::global::PENDED_TICKS = $next_val
        }
    };
}

#[macro_export]
macro_rules! GetPendedTicks {
    () => {
        unsafe { crate::global::PENDED_TICKS }
    };
}

#[macro_export]
macro_rules! SetTaskNumber {
    ($next_val: expr) => {
        unsafe {
            info!("TASK_NUMBER was set to {}", $next_val);
            crate::global::TASK_NUMBER = $next_val
        }
    };
}

#[macro_export]
macro_rules! GetTaskNumber {
    () => {
        unsafe { crate::global::TASK_NUMBER }
    };
}

#[macro_export]
macro_rules! GetYieldPending {
    () => {
        unsafe { crate::global::YIELD_PENDING }
    };
}

#[macro_export]
macro_rules! SetYieldPending {
    ($true_or_flase: expr) => {
        unsafe {
            info!("YIELD_PENDING was set to {}", $true_or_flase);
            crate::global::YIELD_PENDING = $true_or_flase;
        }
    };
}

#[macro_export]
macro_rules! SetCurrentNumberOfTasks {
    ($next_val: expr) => {
        unsafe {
            info!("CURRENT_NUMBER_OF_TASKS was set to {}", $next_val);
            crate::global::CURRENT_NUMBER_OF_TASKS = $next_val;
        }
    };
}

#[macro_export]
macro_rules! GetCurrentNumberOfTasks {
    () => {
        unsafe { crate::global::CURRENT_NUMBER_OF_TASKS }
    };
}

#[macro_export]
macro_rules! SetSchedulerRunning {
    ($true_or_flase: expr) => {
        unsafe {
            info!("SCHEDULER_RUNNING was set to {}", $true_or_flase);
            crate::global::SCHEDULER_RUNNING = $true_or_flase
        }
    };
}

#[macro_export]
macro_rules! GetSchedulerRunning {
    () => {
        unsafe { crate::global::SCHEDULER_RUNNING }
    };
}

#[macro_export]
macro_rules! GetNextTaskUnblockTime {
    () => {
        unsafe { crate::global::NEXT_TASK_UNBLOCK_TIME }
    };
}

#[macro_export]
macro_rules! SetNextTaskUnblockTime {
    ($new_time: expr) => {
        unsafe {
            info!("NEXT_TASK_UNBLOCK_TIME was set to {}", $new_time);
            crate::global::NEXT_TASK_UNBLOCK_TIME = $new_time;
        }
    };
}

#[macro_export]
macro_rules! GetTickCount {
    () => {
        unsafe { crate::global::TICK_COUNT }
    };
}

#[macro_export]
macro_rules! SetTickCount {
    ($next_tick_count: expr) => {
        unsafe {
            info!("TICK_COUNT was set to {}", $next_tick_count);
            crate::global::TICK_COUNT = $next_tick_count;
        }
    };
}

#[macro_export]
macro_rules! GetNumOfOverflows {
    () => {
        unsafe { crate::global::NUM_OF_OVERFLOWS }
    };
}

#[macro_export]
macro_rules! SetNumOfOverflows {
    ($next_tick_count: expr) => {
        unsafe {
            info!("NUM_OF_OVERFLOWS was set to {}", $next_tick_count);
            crate::global::NUM_OF_OVERFLOWS = $next_tick_count;
        }
    };
}

#[macro_export]
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
macro_rules! SetTotalRunTime {
    ($next_val: expr) => {
        unsafe {
            info!("TOTAL_RUN_TIME was set to {}", $next_val);
            TOTAL_RUN_TIME = $next_val
        }
    };
}

#[macro_export]
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
macro_rules! GetTotalRunTime {
    () => {
        unsafe { TOTAL_RUN_TIME }
    };
}

#[macro_export]
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
macro_rules! SetTaskSwitchInTime {
    ($next_val: expr) => {
        unsafe {
            info!("TASK_SWITCHED_IN_TIME was set to {}", $next_val);
            TASK_SWITCHED_IN_TIME = $next_val
        }
    };
}

#[macro_export]
#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
macro_rules! GetTaskSwitchInTime {
    () => {
        unsafe { TASK_SWITCHED_IN_TIME }
    };
}

#[macro_export]
macro_rules! GetCurrentTaskHandleWrapped {
    () => {
        crate::global::CURRENT_TCB.read().unwrap().as_ref()
    };
}

#[macro_export]
macro_rules! GetCurrentTaskHandle {
    () => {
        crate::global::CURRENT_TCB.read().unwrap().as_ref().unwrap().clone()
    };
}

#[macro_export]
macro_rules! SetCurrentTaskHandle {
    ($cloned_new_task: expr) => {
        info!("CURRENT_TCB changed!");
        *(crate::global::CURRENT_TCB).write().unwrap() = Some($cloned_new_task)
    };
}

#[macro_export]
macro_rules! GetCurrentTaskPriority {
    () => {
        GetCurrentTaskHandle!().GetPriority()
    };
}

#[cfg(feature = "INCLUDE_xTaskAbortDelay")]
#[macro_export]
macro_rules! GetCurrentTaskDelayAborted {
    () => {
        get_current_task_handle!().get_delay_aborted()
    };
}

#[macro_export]
macro_rules! taskCHECK_FOR_STACK_OVERFLOW {
    () => {
    };
}

#[macro_export]
macro_rules! SwitchDelayedList {
    () => {
        unsafe {
            let mut delayed = DELAYED_TASK_LIST.write().unwrap();
            let mut overflowed = OVERFLOW_DELAYED_TASK_LIST.write().unwrap();
            let tmp = (*delayed).clone();
            *delayed = (*overflowed).clone();
            *overflowed = tmp;
        }
    };
}
