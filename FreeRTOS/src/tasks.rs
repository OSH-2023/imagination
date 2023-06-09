use crate::*;
use crate::port::{TickType_t, UBaseType_t, BaseType_t, StackType_t};
use crate::port::{TickType, UBaseType, BaseType, StackType};
use crate::port::*;

pub type CVoidPointer = *mut std::os::raw::c_void;
pub type xTaskHandle = *mut ::std::os::raw::c_void;

use core::ops::FnOnce;
use std::mem;
use std::sync::{Arc, RwLock, Weak};

use crate::global::*;
use crate::list::*;
use crate::config::*;

use log::{error, warn, info, debug};
use crate::projdefs::FreeRtosError;

// #[macro_export]
// macro_rules! portMAX_DELAY {
//     () => 256;
// }

// #[macro_use]
// use crate::{GetTopReadyPriority};

// #[macro_use]
// use crate::{GetSchedulerRunning};

// #[macro_use]
// use crate::{GetCurrentTaskHandle};

// #[macro_use]
// use crate::{GetNextTaskUnblockTime};

// #[macro_use]
// use crate::{GetTickCount};

// #[macro_use]
// use crate::{configMAX_PRIORITIES};

// #[macro_use]
// use crate::{configMINIMAL_STACK_SIZE};

// #[macro_use]
// use crate::{DebugPrint};

// #[macro_use]
// use crate::{SetTopReadyPriority};

// #[macro_use]
// use crate::{GetHandleFromOption};

// #[macro_use]
// use crate::{traceTASK_PRIORITY_SET};

// #[macro_use]
// use crate::{portRESET_READY_PRIORITY};

// #[macro_use]
// use crate::{traceMOVED_TASK_TO_READY_STATE};

// #[macro_use]
// use crate::{tracePOST_MOVED_TASK_TO_READY_STATE};

// #[macro_use]
// use crate::{GetCurrentNumberOfTasks};

// #[macro_use]
// use crate::{SetCurrentNumberOfTasks};

// #[macro_use]
// use crate::{SetTaskNumber};

// #[macro_use]
// use crate::{GetTaskNumber};

// #[macro_use]
// use crate::{GetCurrentTaskPriority};

// #[macro_use]
// use crate::{SetNextTaskUnblockTime};

// #[macro_use]
// use crate::{SetCurrentTaskHandle};

// #[macro_use]
// use crate::{traceTASK_CREATE};

// #[macro_export]
// macro_rules! configMAX_PRIORITIES {
//     () => 16
// }
pub static mut portMAX_DELAY: u32 = 256;
pub static mut taskEVENT_LIST_ITEM_VALUE_IN_USE: u32 = 0;
/* 
#[macro_export]
macro_rules!  taskYIELD_IF_USING_PREEMPTION {
    () => ()
}

#[macro_export]
macro_rules!  taskENTER_CRITICAL {
    () => ()
}


#[macro_export]
macro_rules!  taskEXIT_CRITICAL {
    () => ()
}
 */
#[derive(Debug)]
pub struct TCB {
    //* basic information
    StateListItem: ItemLink,
    EventListItem: ItemLink,
    TaskName: String,
    StackDepth: UBaseType_t,
    Priority: UBaseType_t,
    StackPointer: StackType_t,

    #[cfg(feature = "portCRITICAL_NESTING_IN_TCB")]
    CriticalNesting: UBaseType_t,
    BasePriority: UBaseType_t,
    #[cfg(feature = "configUSE_MUTEXES")]
    MutexedHeld: UBaseType_t,
    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    RuntimeCounter: TickType_t,
    #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
    NotifiedValue: u32,
    #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
    NotifyState: u8,
    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    DelayAborted: bool,
}

impl PartialEq for TCB {
    fn eq(&self, other: &Self) -> bool {
        self.StackPointer == other.StackPointer
    }
}

extern "C" fn run_wrapper(func_to_run: CVoidPointer) {
    info!(
        "Run_wrapper: The function is at position: {:X}",
        func_to_run as u64
    );
    unsafe {
        let func_to_run = Box::from_raw(func_to_run as *mut Box<dyn FnOnce() + 'static>);
        func_to_run();
    }
}

impl TCB {
    pub fn new() -> Self {
        TCB {
            StateListItem: Default::default(),
            EventListItem: Default::default(),
            Priority: 1,       //initialized with 1
            StackDepth: configMINIMAL_STACK_SIZE!(),
            TaskName: String::from("Unnamed"),
            StackPointer: 0,

            #[cfg(feature = "portCRITICAL_NESTING_IN_TCB")]
            CriticalNesting: 0,

            BasePriority: 0,
            #[cfg(feature = "configUSE_MUTEXES")]
            MutexedHeld: 0,
            #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
            RuntimeCounter: 0,
            #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
            NotifiedValue: 0,
            #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
            NotifyState: 0,
            #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
            DelayAborted: false,
        }
    }
    
    pub fn GetName(&self) -> String {
        self.TaskName.clone()
    }

    pub fn SetName(&mut self, name: &str) -> &mut Self {
        self.TaskName = name.to_owned().to_string();
        self
    }

    pub fn SetStackDepth(&mut self, stacksize: UBaseType_t) -> &mut Self {
        self.StackDepth = stacksize;
        self
    }

    pub fn GetPriority(&self) -> UBaseType_t {
        self.Priority.clone()
    }

    // pub fn SetPriority(mut self, priority: UBaseType_t) ->  Self {
    //     if priority >= configMAX_PRIORITIES!() {
    //         warn!("Specified priority larger than system maximum priority, will be reduced.");
    //         info!(
    //             "MAX_PRIORITY is {}, but got {}",
    //             configMAX_PRIORITIES!() - 1,
    //             priority
    //         );
    //         self.Priority = configMAX_PRIORITIES!() - 1;
    //     } else {
    //         self.Priority = priority;
    //     }
    //     self
    // }
    pub fn SetPriority(&mut self, new_priority: UBaseType) {
        self.Priority = new_priority;
    }

    pub fn GetBasePriority(&self) -> UBaseType_t {
        self.BasePriority
    }

    pub fn SetBasePriority(&mut self, new_val: UBaseType_t) {
        self.BasePriority = new_val
    }

    pub fn GetStateListItem(&self) -> ItemLink {
        Arc::clone(&self.StateListItem)
    }

    pub fn GetEventListItem(&self) -> ItemLink {
        Arc::clone(&self.EventListItem)
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn GetRunTime(&self) -> TickType_t {
        self.RuntimeCounter
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn SetRunTime(&mut self, next_val: TickType_t) -> TickType_t {
        let prev_val: u32 = self.RuntimeCounter;
        self.RuntimeCounter = next_val;
        prev_val
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn GetDelayAborted(&self) -> bool {
        self.DelayAborted
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn SetDelayAborted(&mut self, next_val: bool) -> bool {
        let prev_val: bool = self.DelayAborted;
        self.DelayAborted = next_val;
        prev_val
    }
    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn GetMutexHeldCount(&self) -> UBaseType_t {
        self.MutexedHeld
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn SetMutexHeldCount(&mut self, new_count: UBaseType_t) {
        self.MutexedHeld = new_count;
    }

    pub fn INITIALIZE<F>(mut self, func: F) -> Result<TaskHandle, FreeRtosError>
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let size_of_stacktype = std::mem::size_of::<StackType>();
        let stacksize_as_bytes = size_of_stacktype * self.StackDepth as usize;
        info!(
            "Initialising Task: {}, stack size: {} bytes",
            self.TaskName,
            stacksize_as_bytes
        );
        let px_stack = port::port_malloc(stacksize_as_bytes)?;

        self.StackPointer = px_stack as StackType;
        info!(
            "StackPointer for task {} is {}",
            self.TaskName,
            self.StackPointer
        );

        let mut top_of_stack = self.StackPointer + self.StackDepth as StackType - 1;
        top_of_stack = top_of_stack & portBYTE_ALIGNMENT_MASK as StackType;

        let f = Box::new(Box::new(func) as Box<dyn FnOnce()>); // Pass task function as a parameter.
        let param_ptr = &*f as *const _ as *mut _; // Convert to raw pointer.
        info!(
            "Function ptr of {} is at {:X}",
            self.GetName(),
            param_ptr as u64
        );

        let result =
        //Some(run_wrapper)
            port::port_initialise_stack(top_of_stack as *mut _, 32, param_ptr);
        match result {
            Ok(_) => {
                info!("Stack initialisation succeeded");
                mem::forget(f);
            }
            Err(e) => return Err(e),
        }

        #[cfg(feature = "configUSE_MUTEXES")]
        {
            self.mutexes_held = 0;
            self.base_priority = self.task_priority;
        }

        #[cfg(feature = "portCRITICAL_NESTING_IN_TCB")]
        {
            self.critical_nesting = 0;
        }

        #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
        {
            self.runtime_counter = 0;
        }

        #[cfg(feature = "config_USE_TASK_NOTIFICATIONS")]
        {
            self.notify_state = taskNOT_WAITING_NOTIFICATION;
            self.notified_value = 0;
        }

        let sp = self.StackPointer;
        let handle = TaskHandle(Arc::new(RwLock::new(self)));
        let state_list_item = handle.GetStateListItem();
        let event_list_item = handle.GetEventListItem();
        list::set_list_item_owner(&state_list_item, handle.clone());
        list::set_list_item_owner(&event_list_item, handle.clone());
        let item_value = (configMAX_PRIORITIES!() - handle.GetPriority()) as TickType;
        list::listSET_LIST_ITEM_VALUE(&state_list_item, item_value);

        handle.AddNewTaskToReadyList()?;

        Ok(handle)
    }
}

#[derive(Clone)]
pub struct TaskHandle(Arc<RwLock<TCB>>);

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        *self.0.read().unwrap() == *other.0.read().unwrap()
    }
}

impl From<Weak<RwLock<TCB>>> for TaskHandle {
    fn from(weak_link: Weak<RwLock<TCB>>) -> Self {
        TaskHandle(
            weak_link
                .upgrade()
                .unwrap_or_else(|| panic!("Owner is not set")),
        )
    }
}

impl From<TaskHandle> for Weak<RwLock<TCB>> {
    fn from(task: TaskHandle) -> Self {
        Arc::downgrade(&task.0)
    }
}

#[macro_use]
pub fn RecordReadyPriority(priority: UBaseType_t) {
    if priority > GetTopReadyPriority!() {
        SetTopReadyPriority!(priority);
    }
}
#[macro_export]
macro_rules! GetTCB_read {
    ($handle: expr) => {
        match $handle.0.try_read() {
            Ok(a) => a,
            Err(_) => {
                warn!("TCB was locked, read failed");
                panic!("Task handle locked!");
            }
        }
    };
}

#[macro_export]
macro_rules! GetTCB_write {
    ($handle: expr) => {
        match $handle.0.try_write() {
            Ok(a) => a,
            Err(_) => {
                warn!("TCB was locked, write failed");
                panic!("Task handle locked!");
            }
        }
    };
}

// #[macro_export]
// macro_rules! GetHandleFromOption {
//         ($option: expr) => {
//             match $option {
//                 Some(handle) => handle,
//                 None => get_current_task_handle!(),
//             }
//         };
//     }

impl TaskHandle {
    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn GetRunTime(&self) -> TickType_t {
        GetTCB_read!(self).GetRunTime()
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn SetRunTime(&self, next_val: TickType_t) -> TickType_t {
        GetTCB_write!(self).SetRunTime(next_val)
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn GetDelayAborted(&self) -> bool {
        GetTCB_read!(self).GetDelayAborted()
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn SetDelayAborted(&self, next_val: bool) -> bool {
        GetTCB_write!(self).SetDelayAborted(next_val)
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn GetMutexHeldCount(&self) -> UBaseType_t {
        GetTCB_read!(self).GetMutexHeldCount()
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn SetMutexHeldCount(&self, new_count: UBaseType_t) {
        GetTCB_write!(self).SetMutexHeldCount(new_count)
    }

    pub fn from_arc(arc: Arc<RwLock<TCB>>) -> Self {
        TaskHandle(arc)
    }
    pub fn from(tcb: TCB) -> Self {
        TaskHandle(Arc::new(RwLock::new(tcb)))
    }

    pub fn as_raw(self) -> xTaskHandle {
        Arc::into_raw(self.0) as *mut _
    }

    pub fn GetEventListItem(&self) -> ItemLink {
        GetTCB_read!(self).GetEventListItem()
    }

    pub fn GetStateListItem(&self) -> ItemLink {
        GetTCB_read!(self).GetStateListItem()
    }

    pub fn GetName(&self) -> String {
        GetTCB_read!(self).GetName()
    }

    pub fn GetPriority(&self) -> UBaseType_t {
        self.0.read().unwrap().GetPriority()
    }

    pub fn SetPriority(&self, new_priority: UBaseType_t) {
        GetTCB_write!(self).SetPriority(new_priority);
    }

    pub fn SetPriorityInDetail(&mut self, NewPriority: UBaseType_t) {
        let mut NewPriority = NewPriority;
        let mut YieldRequired: bool = false;
        let mut CurrentBasePriority: UBaseType_t = 0;
        let mut PriorityUsedOnEntry: UBaseType_t = 0;
    
        //首先检查NewPriority是否大于最大限定，并对其进行修改
        if NewPriority >= configMAX_PRIORITIES!() as UBaseType_t {
            NewPriority = configMAX_PRIORITIES!() as UBaseType_t - 1 as UBaseType_t;
        } else {
            DebugPrint!();
        }
    
        taskENTER_CRITICAL!();
        {
            let mut pTCB = GetTCB_write!(self);

            // let lock = RwLock::new(TCB { /* ... */ });
            // let guard = lock.write().unwrap();
            let pxTCB: &TCB = &*pTCB;
            traceTASK_PRIORITY_SET!(&pxTCB, &NewPriority); 
    
    //基础优先级（base priority）是指任务在创建时分配的优先级
    //当使用互斥锁时，任务的实际运行优先级可能会高于其基础优先级，
    //以避免优先级反转问题
    //当不再需要避免优先级反转问题时，任务的运行优先级将恢复为其基础优先级
    
    //优先级反转：通常发生在多个任务共享资源时
    //例如，假设有三个任务：高优先级任务 H、中优先级任务 M 和低优先级任务 L
    //假设 L 正在使用一个共享资源 R，而 H 需要使用该资源。此时，H 会被阻塞，直到 L 释放资源 R
    //然而，在 L 释放资源 R 之前，如果 M 变为可运行状态，则 M 会抢占 L（因为 M 的优先级高于 L）
    //导致 L 无法及时释放资源 R，从而导致 H 无法运行。
            {
                #![cfg(feature = "configUSE_MUTEXES")]
                CurrentBasePriority = pxTCB.GetBasePriority();
            }
    
            {
                #![cfg(not(feature = "configUSE_MUTEXES"))]
                CurrentBasePriority = pxTCB.GetPriority();
            }
    
            if CurrentBasePriority != NewPriority {
                //如果该任务不是正在执行的任务
                //且优先级比现在正在执行的任务优先级高，Yield = 1
                if self != &mut GetCurrentTaskHandle!() {
                    if NewPriority >= GetCurrentTaskPriority!() {
                        YieldRequired = true;
                    } else {
                        DebugPrint!();
                    }
                }
            } else if self == &mut GetCurrentTaskHandle!() {
                //如果该任务是正在执行的任务
                //而且优先级改变了，Yield = 1
                YieldRequired = true;
            } 
    
            {
                #![cfg(feature = "configUSE_MUTEXES")]
                if pxTCB.GetBasePriority() == pxTCB.GetPriority() {
                    pxTCB.SetPriority(NewPriority);
                } else {
                    DebugPrint!();
                }
                pxTCB.SetBasePriority(NewPriority);
            }
            #[cfg(not(feature = "configUSE_MUTEXS"))]
            // pxTCB.SetPriority(NewPriority);
    
            let EventListItem = pxTCB.GetEventListItem();
            let StateListItem = pxTCB.GetStateListItem();
    
            unsafe {
                if (list::listGET_LIST_ITEM_VALUE(&EventListItem) & taskEVENT_LIST_ITEM_VALUE_IN_USE) == 0 {
                    //检查事件列表项的值是否未被使用
                    //如果未被使用，则更新事件列表项的值，将其设置为 configMAX_PRIORITIES!() - NewPriority
                    list::listSET_LIST_ITEM_VALUE(
                        &EventListItem,
                        (configMAX_PRIORITIES!() as TickType_t - NewPriority as TickType_t),
                    );
                } else {
                    DebugPrint!();
                }
            }
    
            if list::is_contained_within(
                //检查该任务是否包含在就绪任务列表中
                &READY_TASK_LISTS[PriorityUsedOnEntry as usize],
                &StateListItem,
            ) {
                //如果包含，则从就绪任务列表中移除该状态列表项
                if list::uxListRemove(StateListItem) == 0 as UBaseType_t {
                    //如果移除后就绪任务列表为空，则使用 portRESET_READY_PRIORITY!() 宏重置就绪任务优先级
                    portRESET_READY_PRIORITY!(PriorityUsedOnEntry, uxTopReadyPriority);
                } else {
                    DebugPrint!();
                }
                //将任务重新添加到就绪任务列表
                self.AddTaskToReadyList();
            } else {
                DebugPrint!();
            }
    
            if YieldRequired != false {
                //Yield = 1时，如果支持抢占，那就抢占
                taskYIELD_IF_USING_PREEMPTION!();
            } else {
                DebugPrint!();
            }
        }
    
        taskEXIT_CRITICAL!();
    }

    pub fn GetBasePriority(&self) -> UBaseType_t {
        GetTCB_read!(self).GetBasePriority()
    }

    pub fn SetBasePriority(&self, new_val: UBaseType_t) {
        GetTCB_write!(self).SetBasePriority(new_val)
    }
    

    // pub fn GetTCB_read!() -> Result<(TCB), FreeRtosError>{
    //     match self.0.try_read() {
    //         Ok(handle) => handle,
    //         Err(_) => {
    //             warn!("TCB was locked, read failed");
    //             panic!("Task handle locked!");
    //         }
    //     }
    // }

    // pub fn GetTCB_write!(&self) -> Result<(TCB), FreeRtosError>{
    //     match self.0.try_write() {
    //         Ok(handle) => handle,
    //         Err(_) => {
    //             warn!("TCB was locked, write failed");
    //             panic!("Task handle locked!");
    //         }
    //     }
    // }

    pub fn AddTaskToReadyList(&self) -> Result<(), FreeRtosError> {
        //该任务本来就已经存在，只是可能从其他地方（比如阻塞队列）中移动到ReadyList
        //从Handle获取TCB
        let tcb = GetTCB_read!(self);
        let priority = self.GetPriority();
        //和当前最高Priority的任务比较,如果该taskhandle更高，就更新
        traceMOVED_TASK_TO_READY_STATE!(&tcb);
        RecordReadyPriority(priority);
        //插入链表end
        list::vListInsertEnd(
            &READY_TASK_LISTS[priority as usize],
            &Arc::clone(&Arc::clone(&tcb.StateListItem)),
        );
        tracePOST_MOVED_TASK_TO_READY_STATE!(&tcb);
        Ok(())
    }

    fn AddNewTaskToReadyList(&self) -> Result<(), FreeRtosError> {
        let newtcb = GetTCB_read!(self);
    
        //C语言宏接口
        taskENTER_CRITICAL!();
        {
            //通过全局变量获取并设定当前所有列表Task总数数量
            let CurrentNumber = GetCurrentNumberOfTasks!() + 1;
            SetCurrentNumberOfTasks!(CurrentNumber);

            if crate::global::CURRENT_TCB.read().unwrap().is_none() {
                //如果现在没有currenttskhandle
                //那就把newtcb这个tskhandle设定为currenttskhandle
                SetCurrentTaskHandle!(self.clone());
                if GetCurrentNumberOfTasks!() != 1 {
                    DebugPrint!();
                }
            } else {
                //如果现在有currenttskhandle
                let tskhandle = GetCurrentTaskHandle!();
                if !GetSchedulerRunning!() {
                    //如果现在调度器没启动，并且newtcb优先就高于currenttsk，那就用这个newtcb作为启动task
                    if tskhandle.GetPriority() <= newtcb.Priority {
                        SetCurrentTaskHandle!(self.clone());
                    } else {
                        DebugPrint!();
                    }
                }
            }
            //获取task总数
            SetTaskNumber!(GetTaskNumber!() + 1);
            traceTASK_CREATE!(self.clone());
            self.AddTaskToReadyList()?;
        }
        //C语言宏接口
        taskEXIT_CRITICAL!();
        if GetSchedulerRunning!() {
            //如果调度器启动了，而currenttsk优先级低于newtcb，那就中断抢占
            let current_task_priority = GetCurrentTaskHandle!().GetPriority();
            if current_task_priority < newtcb.Priority {
                //C语言接口
                taskYIELD_IF_USING_PREEMPTION!();
            } else {
                DebugPrint!();
            }
        } else {
            DebugPrint!();
        }
    
        Ok(())
    }
}

//INCLUDE_xTaskAbortDelay用于启用xTaskAbortDelay()函数。将其定义为1即可启用此功能。
//xTaskAbortDelay()强制任务离开阻塞状态并进入就绪状态。
//即使任务处于阻塞状态等待的事件未发生，且任何指定的超时时间未过期，也将离开阻塞
pub fn AddCurrentTaskToDelayedList(ticks_to_wait: TickType_t, can_block_indefinitely: bool) {
    let curtskhandle = GetCurrentTaskHandle!();
    info!("Remove succeeded");

    {//设置标志位，表示成功abort
        #![cfg(feature = "INCLUDE_xTaskAbortDelay")]
        curtskhandle.SetDelayAborted(false);
    }
    info!("Abort succeeded");

    //为了将该任务加入Blocked List，首先要从Ready List中移除
    if list::uxListRemove(curtskhandle.GetStateListItem()) == 0 {
        info!("Returned 0");
        //成功移除
        //重新设置Ready List的最高优先级
        portRESET_READY_PRIORITY!(curtskhandle.GetPriority(), GetTopReadyPriority!());
    } else {
        info!("Returned not 0");
        DebugPrint!();
    }

    info!("Remove succeeded");
    {
        //INCLUDE_vTaskSuspend被定义的情况：
        #![cfg(feature = "INCLUDE_vTaskSuspend")]
        //INCLUDE_vTaskSuspend光被定义还没用，还需要判断portMAX_DELAY和can_block_indefinitely
        if ticks_to_wait == crate::port::portMAX_DELAY && can_block_indefinitely {
            //将任务添加到Suspended List而不是Delayed List
            //确保它永久阻塞而不会被时钟唤醒
            let cur_state_list_item = curtskhandle.GetStateListItem();
            list::vListInsertEnd(&SUSPENDED_TASK_LIST, &cur_state_list_item);
        } else {
            //否则，函数会计算任务应该在什么时候被唤醒（如果事件没有发生）
            //并根据唤醒时间将任务添加到不同的延迟任务列表中
            let time_to_wake = GetTickCount!() + ticks_to_wait;

            let cur_state_list_item = curtskhandle.GetStateListItem();
            list::listSET_LIST_ITEM_VALUE(&cur_state_list_item, time_to_wake);

            if time_to_wake < GetTickCount!() {
                //如果唤醒时间小于当前时间，则将任务添加到溢出延迟任务列表中
                list::vListInsert(&OVERFLOW_DELAYED_TASK_LIST, &cur_state_list_item);
            } else {
                //否则，将任务添加到当前延迟任务列表中
                list::vListInsert(&DELAYED_TASK_LIST, &curtskhandle.GetStateListItem());

                //如果任务被添加到了延迟任务列表的头部
                //那么xNextTaskUnblockTime需要被更新
                if time_to_wake < GetNextTaskUnblockTime!() {
                    SetNextTaskUnblockTime!(time_to_wake);
                } else {
                    DebugPrint!();
                }
            }
        }
    }

    {
        //INCLUDE_vTaskSuspend压根没被定义的情况：
        #![cfg(not(feature = "INCLUDE_vTaskSuspend"))]
        //计算任务应该在什么时候被唤醒（如果事件没有发生）
        //并根据唤醒时间将任务添加到适当的延迟任务列表中
        let time_to_wake = GetTickCount!() + ticks_to_wait;

        let cur_state_list_item = curtskhandle.GetStateListItem();
        list::listSET_LIST_ITEM_VALUE(&cur_state_list_item, time_to_wake);

        if time_to_wake < GetTickCount!() {
           //如果唤醒时间小于当前时间，则将任务添加到溢出延迟任务列表中
            list::vListInsert(&OVERFLOW_DELAYED_TASK_LIST, &cur_state_list_item);
        } else {
            //否则，将任务添加到当前延迟任务列表中
            list::vListInsert(&DELAYED_TASK_LIST, &curtskhandle.GetStateListItem());

            //如果任务被添加到了延迟任务列表的头部
            //那么xNextTaskUnblockTime需要被更新
            if time_to_wake < GetNextTaskUnblockTime!() {
                SetNextTaskUnblockTime!(time_to_wake);
            } else {
                DebugPrint!();
            }
        }
    }
    info!("Place succeeded");
}

pub fn ResetNextTaskUnblockTime() {
    if listLIST_IS_EMPTY(&DELAYED_TASK_LIST) {
        //检查DELAYED_TASK_LIST是否为空
        //如果为空，则将 xNextTaskUnblockTime 设置为最大可能值
        SetNextTaskUnblockTime!(portMAX_DELAY);
    } else {
        //如果DELAYED_TASK_LIST不为空
        //就把xNextTaskUnblockTime设置为该列表头结点的等待时间
        let mut temp = get_owner_of_head_entry(&DELAYED_TASK_LIST);
        SetNextTaskUnblockTime!(listGET_LIST_ITEM_VALUE(&temp.GetStateListItem()));
    }
}

#[cfg(feature = "INCLUDE_vTaskDelete")]
pub fn TaskDelete(task_to_delete: Option<TaskHandle>) {
    //如果NULL被传入，就删除calling task
    let pxtcb = GetTCB_write!(task_to_delete);

    taskENTER_CRITICAL!();
    {
        //将任务从Ready List移除,前提是它在这个List
        if list::uxListRemove(pxtcb.GetStateListItem()) == 0 {
            //重新设置优先级
            taskRESET_READY_PRIORITY!(pxtcb.GetPriority());
        } else {
            DebugPrint!();
        }

        //接下来检查任务是否在等待事件
        //如果是，则从事件列表中移除该任务；否则，执行测试标记。
        if list::get_list_item_container(&pxtcb.GetEventListItem()).is_some() {
            list::uxListRemove(pxtcb.GetEventListItem());
        } else {
            DebugPrint!();
        }

        //递增全局变量uxTaskNumber，以便内核感知调试器能够检测到任务列表需要重新生成。
        SetTaskNumber!(GetTaskNumber!() + 1);

        if pxtcb == GetCurrentTaskHandle!() {
            //检查要删除的任务是否是当前正在运行的任务
            //如果是，则将该任务插入到等待终止的任务列表中
            //以便空闲任务能够检查该列表并释放调度器为TCB和堆栈分配的内存。
            list::vListInsertEnd(&TASKS_WAITING_TERMINATION, pxtcb.GetStateListItem());

            //递增全局变量ucTasksDeleted
            //以便空闲任务知道有一个任务已被删除，因此应检查xTasksWaitingTermination列表
            set_deleted_tasks_waiting_clean_up!(get_deleted_tasks_waiting_clean_up!() + 1);

            //接下来调用宏portPRE_TASK_DELETE_HOOK!
            //该宏主要用于Windows模拟器，在其中执行特定于Windows的清理操作
            //在此之后，无法从此任务中产生YIELD，因此使用xYieldPending来锁定需要进行上下文切换
            portPRE_TASK_DELETE_HOOK!(pxtcb, get_yield_pending!());
        } else {
            //如果要删除的任务不是当前正在运行的任务，则递减全局变量uxCurrentNumberOfTasks
            SetCurrentNumberOfTasks!(GetCurrentNumberOfTasks!() - 1);

            //释放堆栈所占用的内存
            let StackPointer = GetTCB_read!(pxtcb).StackPointer;
            port::port_free(StackPointer as *mut _);

            //调用函数reset_next_task_unblock_time()重置下一个预期解除阻塞时间
            ResetNextTaskUnblockTime();
        }
    }
    taskEXIT_CRITICAL!();

    //如果删除的任务是当前执行的任务，需要reschedule
    if get_scheduler_suspended!() > 0 {
        if pxtcb == GetCurrentTaskHandle!() {
            assert!(get_scheduler_suspended!() == 0);
            portYIELD_WITHIN_API!();
        } else {
            DebugPrint!();
        }
    }
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn SuspendTask(task_to_suspend: TaskHandle) {
    info!("SuspendTask called!");
    
    let mut tcb = GetTCB_read!(task_to_suspend);
    taskENTER_CRITICAL!();
    {
        traceTASK_SUSPEND!(&tcb);

        //从原列表移除，并修改优先级
        if list_remove(tcb.GetStateListItem()) == 0 {
            taskRESET_READY_PRIORITY!(tcb.GetPriority());
        } else {
            DebugPrint!();
        }

        //从事件列表中移除
        if get_list_item_container(&tcb.GetEventListItem()).is_some() {
            list_remove(tcb.GetEventListItem());
        } else {
            DebugPrint!();
        }
        //加入SUSPEND列表
        list_insert_end(&SUSPENDED_TASK_LIST, tcb.GetStateListItem());
    }
    taskEXIT_CRITICAL!();

    if GetSchedulerRunning!() {
        //修改next_task_unblock_time
        taskENTER_CRITICAL!();
        {
            ResetNextTaskUnblockTime();
        }
        taskEXIT_CRITICAL!();
    } else {
        DebugPrint!();
    }

    if task_to_suspend == GetCurrentTaskHandle!() {
        if GetSchedulerRunning!() {
            //如果Scheduler开启了，且该任务为正在执行的任务
            assert!(get_scheduler_suspended!() == 0);
            portYIELD_WITHIN_API!();
        } else {
            //如果Scheduler没有开启，而且pxCurrentTCB所指向的任务被suspend了
            //需要重新设定pxCurrentTCB
            if current_list_length(&SUSPENDED_TASK_LIST) != GetCurrentNumberOfTasks!() {
                //如果不是所有任务都被suspend，那就进行上下文切换
                task_switch_context();
            }
        }
    } else {
        DebugPrint!();
    }
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn IsTaskSuspended(task: &TaskHandle) -> bool {
    //检查给定任务是否处于暂停状态。
    let mut xreturn: bool = false;
    let tcb = GetTCB_read!(task);

    //检查给定任务是否包含在暂停任务列表中
    if is_contained_within(&SUSPENDED_TASK_LIST, &tcb.GetStateListItem()) {
        //如果是，则检查该任务是否正在从中断服务例程（ISR）中恢复
        //即检查在不在PENDING_READY_LIST里
        if !is_contained_within(&PENDING_READY_LIST, &tcb.GetEventListItem()) {
            //如果没有，则检查该任务是否因为处于暂停状态或因为阻塞且没有超时而被包含在暂停列表中
            //即检查是不是事件驱动而待在暂停任务列表
            if get_list_item_container(&tcb.GetEventListItem()).is_none() {
                xreturn = true;
            } else {
                DebugPrint!();
            }
        } else {
            DebugPrint!();
        }
    } else {
        DebugPrint!();
    }

    xreturn
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn ResumeTask(task_to_resume: TaskHandle) {
    //函数目的是恢复一个任务
    info!("resume task called!");
    let mut tcb = GetTCB_read!(task_to_resume);

    if task_to_resume != GetCurrentTaskHandle!() {
        //检查要恢复的任务是否为当前正在执行的任务，如果不是的情况：
        taskENTER_CRITICAL!();
        {
            if IsTaskSuspended(&task_to_resume) {
                //调用task_is_tasksuspended函数检查给定任务是否处于暂停状态
                traceTASK_RESUME!(&tcb);

                //如果是,从暂停列表中删除该任务并将其添加到准备列表中
                list_remove(tcb.GetStateListItem());
                task_to_resume.AddTaskToReadyList();

                let current_task_priority = GetCurrentTaskHandle!().GetPriority();
                if tcb.GetPriority() >= current_task_priority {
                    //检查要恢复的任务的优先级是否大于或等于当前任务的优先级
                    //如果是，则使用宏taskYIELD_IF_USING_PREEMPTION!()触发上下文切换
                    taskYIELD_IF_USING_PREEMPTION!();
                } else {
                    DebugPrint!();
                }
            } else {
                DebugPrint!();
            }
        }
        taskEXIT_CRITICAL!();
    } else {
        //检查要恢复的任务是否为当前正在执行的任务，如果是，那根本不用resume
        DebugPrint!();
    }
}

// #[macro_export]
// macro_rules! GetHandleFromOption {
//     ($option: expr) => {
//         match $option {
//             Some(handle) => handle,
//             None => GetCurrentTaskHandle!(),
//         }
//     };
// }