use std::collections::VecDeque;
use crate::port::*;
use crate::list::*;
use crate::queue_h::*;
use crate::*;
use crate::task_queue::*;
pub const queueQUEUE_IS_MUTEX: UBaseType = 0;
pub const queueUNLOCKED: i8 = -1;
pub const queueLOCKED_UNMODIFIED: i8 = 0;
pub const queueSEMAPHORE_QUEUE_ITEM_LENGTH: UBaseType = 0;
pub const queueMUTEX_GIVE_BLOCK_TIME: TickType = 0;
use log::{error, warn, info, debug, trace};

#[derive(Default)]
pub struct QueueDefinition<T>
    where
        T: Default + Clone
{
    pcQueue: VecDeque<T>,

    pcHead: UBaseType,
    pcTail: UBaseType,
    pcWriteTo: UBaseType,

    QueueUnion: UBaseType,

    xTasksWaitingToSend: ListLink,
    xTasksWaitingToReceive: ListLink,

    uxMessagesWaiting: UBaseType,
    uxLength: UBaseType,
    cRxLock: i8,
    cTxLock: i8,

    ucStaticallyAllocated: u8,


    // pxQueueSetContainer: Option<Box<QueueDefinition>>,


    uxQueueNumber: UBaseType,
    ucQueueType: QueueType,
}
impl<T> QueueDefinition<T>
    where
        T: Default + Clone,
{
   
    #[cfg(feature = "configSUPPORT_DYNAMIC_ALLOCATION")]
    pub fn xQueueGenericCreate(uxQueueLength: UBaseType, ucQueueType: QueueType) -> Self {
        let mut queue: QueueDefinition<T> = Default::default();
        queue.pcQueue = VecDeque::with_capacity(uxQueueLength as usize);
        queue.prvInitialiseNewQueue(uxQueueLength, ucQueueType);
        queue
    }

    pub fn prvQueueGenericCreate(uxQueueLength: UBaseType, ucQueueType: QueueType) -> Self {
        let mut queue: QueueDefinition<T> = Default::default();
        queue.pcQueue = VecDeque::with_capacity(uxQueueLength as usize);
        queue.prvInitialiseNewQueue(uxQueueLength, ucQueueType);
        queue
    }

    pub fn prvInitialiseNewQueue(&mut self, uxQueueLength: UBaseType, ucQueueType: QueueType) {
        self.pcHead = 0;
        self.uxLength = uxQueueLength;
        self.xQueueGenericReset(true);

        self.ucQueueType = ucQueueType;
        /*
        {
            #[cfg(feature = "configUSE_QUEUE_SETS")]
            self.pxQueueSetContainer = None;
        }
*/
        // traceQUEUE_CREATE!(&self);
    }
    pub fn xQueueGenericReset(&mut self, xNewQueue: bool) -> Result<(), QueueError> {
        taskENTER_CRITICAL!();
        {
            self.pcTail = self.pcHead + self.uxLength;
            self.uxMessagesWaiting = 0 as UBaseType;
            self.pcWriteTo = self.pcHead;
            self.QueueUnion = self.pcHead + self.uxLength - (1 as UBaseType); //QueueUnion represents pcReadFrom
            self.cRxLock = queueUNLOCKED;
            self.cTxLock = queueUNLOCKED;
            self.pcQueue.clear();
            if xNewQueue == false {
                if list::listLIST_IS_EMPTY(&self.xTasksWaitingToSend) == false {
                    if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToSend) != false {
                        queueYIELD_IF_USING_PREEMPTION!();
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            } else {
                self.xTasksWaitingToSend = Default::default();
                self.xTasksWaitingToReceive = Default::default();
            }
        }
        taskEXIT_CRITICAL!();
        Ok(())
    }
    pub fn xQueueGenericSend(
        &mut self,
        pvItemToQueue: T,
        xTicksToWait: TickType,
        xCopyPosition: BaseType,
    ) -> Result<(), QueueError> {
        let mut xEntryTimeSet: bool = false;
        let mut xTimeOut: time_out = Default::default();
        let mut xTicksToWait = xTicksToWait;

        assert!(!((xCopyPosition == queueOVERWRITE) && self.uxLength == 1));

        #[cfg(all(feature = "xTaskGetSchedulerState", feature = "configUSE_TIMERS"))]
        assert!(
            !((kernel::task_get_scheduler_state() == SchedulerState::Suspended)
                && (xTicksToWait != 0))
        );
        trace!("Enter function queue_generic_send! TicksToWait: {}, uxMessageWaiting: {}, xCopyPosition: {}", xTicksToWait ,self.uxMessagesWaiting, xCopyPosition);
        loop {
            taskENTER_CRITICAL!();
            {
                if self.uxMessagesWaiting < self.uxLength || xCopyPosition == queueOVERWRITE {
                    // traceQUEUE_SEND!(&self);
                    self.prvCopyDataToQueue(pvItemToQueue, xCopyPosition);
                    trace!("Queue can be sent");
                    #[cfg(feature = "configUSE_QUEUE_SETS")]
                    match self.pxQueueSetContainer {
                        Some => {
                            if notify_queue_set_container(&self, &xCopyPosition) != false {
                                queueYIELD_IF_USING_PREEMPTION!();
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        }
                        None => {
                            if list::list_is_empty(&self.xTasksWaitingToReceive) == false {
                                if task_queue::task_remove_from_event_list(
                                    &self.xTasksWaitingToReceive,
                                ) {
                                    queueYIELD_IF_USING_PREEMPTION!();
                                } else {
                                    mtCOVERAGE_TEST_MARKER!();
                                }
                            }
                        }
                    }

                    {
                        #[cfg(not(feature = "configUSE_QUEUE_SETS"))]
                        if !list::listLIST_IS_EMPTY(&self.xTasksWaitingToReceive) {
                            if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToReceive)
                            {
                                queueYIELD_IF_USING_PREEMPTION!();
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                    taskEXIT_CRITICAL!();
                    return Ok(());
                } else {
                    {
                        #[cfg(feature = "configUSE_MUTEXES")]
                        if self.ucQueueType == QueueType::Mutex || self.ucQueueType == QueueType::RecursiveMutex {
                            taskENTER_CRITICAL!();
                            {
                                let task_handle = self.transed_task_handle_for_mutex();
                                task_queue::task_priority_inherit(task_handle);
                            }
                            taskEXIT_CRITICAL!();
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                    if xTicksToWait == 0 as TickType {
                        taskEXIT_CRITICAL!();
                        traceQUEUE_SEND_FAILED!(&self);
                        trace!("Queue Send: QueueFull");
                        return Err(QueueError::QueueFull);
                    } else if !xEntryTimeSet {
                        task_queue::vTaskSetTimeOutState(&mut xTimeOut);
                        xEntryTimeSet = true;
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }
                }
            }
            taskEXIT_CRITICAL!();
            kernel::TaskSuspendAll();
            self.prvLockQueue();
            if !task_queue::xTaskCheckForTimeOut(&mut xTimeOut, &mut xTicksToWait) {
                if self.prvIsQueueFull() {
                    // traceBLOCKING_ON_QUEUE_SEND!(&self);
                    trace!("queue_generic_send place on event list");
                    task_queue::vTaskPlaceOnEventList(&self.xTasksWaitingToSend, xTicksToWait);
                    self.prvUnlockQueue();
                    if !kernel::TaskResumeAll() {
                        portYIELD_WITHIN_API!();
                    }
                } else {
                    self.prvUnlockQueue();
                    kernel::TaskResumeAll();
                }
            } else {
                self.prvUnlockQueue();
                kernel::TaskResumeAll();

                traceQUEUE_SEND_FAILED!(self);
                return Err(QueueError::QueueFull);
            }
        }
    }
    pub fn xQueueGenericSendFromISR(
        &mut self,
        pvItemToQueue: T,
        xCopyPosition: BaseType,
    ) -> (Result<(), QueueError>, bool) {
        let mut xReturn: Result<(), QueueError> = Ok(());
        let mut pxHigherPriorityTaskWoken: bool = false; //默认为false,下面一些情况改为true

        portASSERT_IF_INTERRUPT_PRIORITY_INVALID!();
        let uxSavedInterruptStatus: UBaseType = 
        // portSET_INTERRUPT_MASK_FROM_ISR!() 
        32 as UBaseType;
        {
            if self.uxMessagesWaiting < self.uxLength || xCopyPosition == queueOVERWRITE {
                let cTxLock: i8 = self.cTxLock;
                traceQUEUE_SEND_FROM_ISR!(&self);
                self.prvCopyDataToQueue(pvItemToQueue, xCopyPosition);

                if cTxLock == queueUNLOCKED {
                    #[cfg(feature = "configUSE_QUEUE_SETS")]
                    match self.pxQueueSetContainer {
                        Some => {
                            if notify_queue_set_container(self, xCopyPosition) != false {
                                pxHigherPriorityTaskWoken = true
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        }
                        None => {
                            if list::list_is_empty(&self.xTasksWaitingToReceive) == false {
                                if task_queue::task_remove_from_event_list(
                                    &self.xTasksWaitingToReceive,
                                ) != false
                                {
                                    pxHigherPriorityTaskWoken = true;
                                } else {
                                    mtCOVERAGE_TEST_MARKER!();
                                }
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        }
                    }

                    {
                        #[cfg(not(feature = "configUSE_QUEUE_SETS"))]
                        if list::listLIST_IS_EMPTY(&self.xTasksWaitingToReceive) == false {
                            if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToReceive)
                                != false
                            {
                                pxHigherPriorityTaskWoken = true;
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                } else {
                    self.cTxLock = (cTxLock + 1) as i8;
                }
                xReturn = Ok(());
            } else {
                traceQUEUE_SEND_FROM_ISR_FAILED!(&self);
                xReturn = Err(QueueError::QueueFull);
            }
        }
        portCLEAR_INTERRUPT_MASK_FROM_ISR!(uxSavedInterruptStatus);
        (xReturn, pxHigherPriorityTaskWoken)
    }
    pub fn prvLockQueue(&mut self) {
        //源码中为宏，改为Queue的方法
        taskENTER_CRITICAL!();
        {
            if self.cRxLock == queueUNLOCKED {
                self.cRxLock = queueLOCKED_UNMODIFIED;
            }
            if self.cTxLock == queueUNLOCKED {
                self.cTxLock = queueLOCKED_UNMODIFIED;
            }
        }
        taskEXIT_CRITICAL!()
    }
    fn prvUnlockQueue(&mut self) {
        taskENTER_CRITICAL!();
        {
            let mut cTxLock: i8 = self.cTxLock;
            while cTxLock > queueLOCKED_UNMODIFIED {
                #[cfg(feature = "configUSE_QUEUE_SETS")]
                match self.pxQueueSetContainer {
                    Some => {
                        if notify_queue_set_container(self, queueSEND_TO_BACK) != false {
                            task_queue::task_missed_yield();
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                    None => {
                        if list::list_is_empty(&self.xTasksWaitingToReceive) == false {
                            if task_queue::task_remove_from_event_list(&self.xTasksWaitingToReceive)
                                != false
                            {
                                task_queue::task_missed_yield();
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        } else {
                            break;
                        }
                    }
                }
                {
                    #[cfg(not(feature = "configUSE_QUEUE_SETS"))]
                    if list::listLIST_IS_EMPTY(&self.xTasksWaitingToReceive) == false {
                        if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToReceive)
                            != false
                        {
                            task_queue::vTaskMissedYield();
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    } else {
                        break;
                    }
                }

                cTxLock = cTxLock - 1;
            }
            self.cTxLock = queueUNLOCKED;
        }
        taskEXIT_CRITICAL!();

        taskENTER_CRITICAL!();
        {
            let mut cRxLock: i8 = self.cRxLock;
            while cRxLock > queueLOCKED_UNMODIFIED {
                if list::listLIST_IS_EMPTY(&self.xTasksWaitingToReceive) == false {
                    if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToReceive)
                        != false
                    {
                        task_queue::vTaskMissedYield();
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }

                    cRxLock = cRxLock - 1;
                } else {
                    break;
                }
            }
            self.cRxLock = queueUNLOCKED;
        }
        taskEXIT_CRITICAL!();
    }
    pub fn xQueueGenericReceive(
        &mut self,
        mut xTicksToWait: TickType,
        xJustPeeking: bool,
    ) -> Result<T, QueueError> {
        let mut xEntryTimeSet: bool = false;
        let mut xTimeOut: time_out = Default::default();
        let mut xYieldRequired: bool = false;
        let mut buffer: Option<T>;
        #[cfg(all(feature = "xTaskGetSchedulerState", feature = "configUSE_TIMERS"))]
        assert!(
            !((kernel::task_get_scheduler_state() == SchedulerState::Suspended)
                && (xTicksToWait != 0))
        );
        loop {
            trace!(
                "Enter function queue_generic_receive, TicksToWait:{}, Peeking: {}!",
                xTicksToWait,
                xJustPeeking
            );
            taskENTER_CRITICAL!();
            {
                let uxMessagesWaiting: UBaseType = self.uxMessagesWaiting;
                trace!(
                    "queue_generic_receive: uxMessageWaiting: {}",
                    uxMessagesWaiting
                );
                if uxMessagesWaiting > 0 as UBaseType {
                    let pcOriginalReadPosition: UBaseType = self.QueueUnion; //QueueUnion represents pcReadFrom
                    buffer = self.prvCopyDataFromQueue(); //
                    if xJustPeeking == false {
                        // traceQUEUE_RECEIVE!(&self);
                        self.uxMessagesWaiting = uxMessagesWaiting - 1;

                        {
                            #[cfg(feature = "configUSE_MUTEXES")]
                            /*if uxQueueType == queueQUEUE_IS_MUTEX*/
                            if self.ucQueueType == QueueType::Mutex
                                || self.ucQueueType == QueueType::RecursiveMutex
                            {
                                let task_handle = self.transed_task_handle_for_mutex();
                                xYieldRequired = task_queue::vTaskPriorityInherit(task_handle);
                                self.pcQueue.pop_front();
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        }
                        trace!("queue_generic_receive -- line 498");
                        if list::listLIST_IS_EMPTY(&self.xTasksWaitingToSend) == false {
                            if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToSend)
                                != false
                            {
                                queueYIELD_IF_USING_PREEMPTION!();
                            } else {
                                trace!("queue_generic_receive -- line 504");
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        } else if xYieldRequired == true {
                            queueYIELD_IF_USING_PREEMPTION!();
                        } else {
                            trace!("queue_generic_receive -- line 508");
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    } else {
                        // traceQUEUE_PEEK!(&self);
                        self.QueueUnion = pcOriginalReadPosition;
                        if list::listLIST_IS_EMPTY(&self.xTasksWaitingToReceive) != false {
                            if task_queue::xTaskRemoveFromEventList(&self.xTasksWaitingToReceive)
                                != false
                            {
                                queueYIELD_IF_USING_PREEMPTION!();
                            } else {
                                mtCOVERAGE_TEST_MARKER!();
                            }
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                    taskEXIT_CRITICAL!();
                    trace!("queue_generic_receive -- line 529");
                    return Ok(buffer.unwrap_or_else(|| panic!("buffer is empty!")));
                } else {
                    if xTicksToWait == 0 as TickType {
                        taskEXIT_CRITICAL!();
                        traceQUEUE_RECEIVE_FAILED!(&self);
                        return Err(QueueError::QueueEmpty);
                    } else if xEntryTimeSet == false {
                        task_queue::vTaskSetTimeOutState(&mut xTimeOut);
                        xEntryTimeSet = true;
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }
                }
            }
            taskEXIT_CRITICAL!();
            trace!("queue_generic_receive -- line 553");
            kernel::TaskSuspendAll();
            self.prvLockQueue();
            trace!("queue_generic_receive -- line 556");
            if task_queue::xTaskCheckForTimeOut(&mut xTimeOut, &mut xTicksToWait) == false {
                if self.prvIsQueueEmpty() != false {
                    // traceBLOCKING_ON_QUEUE_RECEIVE!(&self);
                    task_queue::vTaskPlaceOnEventList(
                        &self.xTasksWaitingToReceive,
                        xTicksToWait,
                    );
                    self.prvUnlockQueue();
                    if kernel::TaskResumeAll() == false {
                        portYIELD_WITHIN_API!();
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }
                } else {
                    self.prvUnlockQueue();
                    kernel::TaskResumeAll();
                }
                trace!("queue_generic_receive -- line 589");
            } else {
                self.prvUnlockQueue();
                kernel::TaskResumeAll();
                if self.prvIsQueueEmpty() != false {
                    traceQUEUE_RECEIVE_FAILED!(&self);
                    return Err(QueueError::QueueEmpty);
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            }
        }
    }
    pub fn prvCopyDataFromQueue(&mut self) -> Option<T> {
        self.QueueUnion += 1;
        if self.QueueUnion >= self.pcTail {
            self.QueueUnion = self.pcHead;
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
        let ret_val = self.pcQueue.get(self.QueueUnion as usize).cloned();
        Some(ret_val.unwrap())
    }
    pub fn prvCopyDataToQueue(&mut self, pvItemToQueue: T, xPosition: BaseType)
    {
        let mut uxMessagesWaiting: UBaseType = self.uxMessagesWaiting;
        {
            #[cfg(feature = "configUSE_MUTEXES")]
            if self.ucQueueType == QueueType::Mutex || self.ucQueueType == QueueType::RecursiveMutex
            {
                let mutex_holder = transed_task_handle_to_T(task_increment_mutex_held_count());
                self.pcQueue.insert(0, mutex_holder);
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }

        if xPosition == queueSEND_TO_BACK {
            if self.ucQueueType != QueueType::Mutex && self.ucQueueType != QueueType::RecursiveMutex {
                self.pcQueue.insert(self.pcWriteTo as usize, pvItemToQueue);
            } else {}
            self.pcWriteTo = self.pcWriteTo + 1;

            if self.pcWriteTo >= self.pcTail {
                self.pcWriteTo = self.pcHead;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        } else {
            if self.ucQueueType != QueueType::Mutex && self.ucQueueType != QueueType::RecursiveMutex {
                self.pcQueue.insert(self.QueueUnion as usize, pvItemToQueue);
            } else {}
            self.QueueUnion = self.QueueUnion - 1;
            if self.QueueUnion < self.pcHead {
                self.QueueUnion = self.pcTail - 1;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }

            if xPosition == queueOVERWRITE {
                if uxMessagesWaiting > 0 as UBaseType {
                    uxMessagesWaiting = uxMessagesWaiting - 1;
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }
        self.uxMessagesWaiting = uxMessagesWaiting + 1;
    }
    pub fn prvIsQueueEmpty(&self) -> bool {
        let mut xReturn: bool = false;
        taskENTER_CRITICAL!();
        {
            if self.uxMessagesWaiting == 0 as UBaseType {
                xReturn = true;
            }
        }
        taskEXIT_CRITICAL!();
        xReturn
    }
    pub fn prvIsQueueFull(&self) -> bool {
        let mut xReturn: bool = false;
        taskENTER_CRITICAL!();
        {
            if self.uxMessagesWaiting == self.uxLength {
                xReturn = true;
            }
        }
        taskEXIT_CRITICAL!();
        xReturn
    }
    pub fn vQueueCreateCountingSemaphore(&mut self, initial_count: UBaseType) {
        self.uxMessagesWaiting = initial_count;
    }
    pub fn vQueueUnionDecrease(&mut self) {
        self.QueueUnion = self.QueueUnion - 1;
    }
    pub fn vQueueUnionIncrease(&mut self) {
        self.QueueUnion = self.QueueUnion + 1;
    }
    pub fn IsQueueUnionZero(&self) -> bool {
        if self.QueueUnion == 0 as UBaseType {
            return true;
        } else {
            return false;
        }
    }
    pub fn uxGetRecursiveCount(&self) -> UBaseType {
        self.QueueUnion
    }
    #[cfg(feature = "configUSE_TRACE_FACILITY")]
    pub fn get_queue_number(&self) -> UBaseType {
        self.uxQueueNumber
    }

    #[cfg(feature = "configUSE_QUEUE_SETS")]
    fn notify_queue_set_container(&self, xCopyPosition: BaseType) {
        unimplemented!();
    }
    pub fn transed_task_handle_for_mutex(&self) -> Option<tasks::TaskHandle> {
        if self.pcQueue.get(0).cloned().is_some() {
            let untransed_task_handle = self.pcQueue.get(0).cloned().unwrap();
            trace!("successfully get the task handle");
            let untransed_task_handle = Box::new(untransed_task_handle);
            let mut task_handle: Option<tasks::TaskHandle>;
            unsafe {
                let transed_task_handle = std::mem::transmute::<
                    Box<T>,
                    Box<Option<tasks::TaskHandle>>,
                >(untransed_task_handle);
                task_handle = *transed_task_handle
            }
            task_handle
        } else {
            None
        }
    }

    
}
fn transed_task_handle_to_T<T>(task_handle: Option<tasks::TaskHandle>) -> T {
    /* use unsafe to transmute Option<TaskHandle> to T type*/
    let mut T_type: T;
    let task_handle = Box::new(task_handle);
    unsafe {
        let transed_T =
            std::mem::transmute::<Box<Option<tasks::TaskHandle>>, Box<T>>(task_handle);
        T_type = *transed_T;
    }
    T_type
}

#[macro_export]
macro_rules! queueYIELD_IF_USING_PREEMPTION {
    () => {
        #[cfg(feature = "configUSE_PREEMPTION")]
        portYIELD_WITHIN_API!();
    };
}

