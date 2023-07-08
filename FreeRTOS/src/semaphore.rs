use crate::port::*;
use crate::queue::*;
use crate::queue_h::*;
use crate::tasks::*;
use crate::*;
use std::cell::UnsafeCell;
use log::{trace};

pub struct Semaphore(UnsafeCell<QueueDefinition<Option<TaskHandle>>>);
unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {

    pub fn xSemaphoreCreateMutex() -> Self {
        Semaphore(UnsafeCell::new(QueueDefinition::prvQueueGenericCreate(1, QueueType::Mutex)))
    }


    #[cfg(all(
        feature = "configUSE_MUTEXES",
        feature = "INCLUDE_xSemaphoreGetMutexHolder"
    ))]
    pub fn xSemaphoreGetMutexHolder(&self) -> Option<task_control::TaskHandle> {
        let mut mutex_holder: Option<task_control::TaskHandle>;
        taskENTER_CRITICAL!();
        {
            unsafe {
                let inner = self.0.get();
                mutex_holder = (*inner).queue_generic_receive(0, true).unwrap();
            }
        }
        taskEXIT_CRITICAL!();
        mutex_holder
    }


    pub fn xSemaphoreGive(&self) -> Result<Option<TaskHandle>, QueueError> {
        unsafe {
            trace!("Semaphore up runs!");
            let inner = self.0.get();
            trace!("Semaphore up get finished!");
            (*inner).xQueueGenericReceive(semGIVE_BLOCK_TIME, false)
        }
    }


    pub fn xSemaphoreTake(&self, xBlockTime: TickType) -> Result<(), QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSend(None, xBlockTime, queueSEND_TO_BACK)
        }
    }


    pub fn xSemaphoreCreateBinary() -> Self {
        Semaphore(UnsafeCell::new(QueueDefinition::prvQueueGenericCreate(

            1,
            QueueType::BinarySemaphore,
        )))
    }


    pub fn xSemaphoreCreateCounting (max_count: UBaseType /*,initial_count:UBaseType*/) -> Self {
        let mut counting_semphr = Semaphore(UnsafeCell::new(QueueDefinition::prvQueueGenericCreate(
            max_count,
            QueueType::CountingSemaphore,
        )));
        unsafe {
            let inner = counting_semphr.0.get();
            (*inner).vQueueCreateCountingSemaphore(0);
        }
        //traceCREATE_COUNTING_SEMAPHORE!();
        counting_semphr
    }


    pub fn xSemaphoreCreateRecursiveMutex() -> Self {
        Semaphore(UnsafeCell::new(QueueDefinition::prvQueueGenericCreate(
            1,
            QueueType::RecursiveMutex,
        )))
    }

    pub fn xSemaphoreGiveRecursive(&self) -> bool {
        unsafe {
            let inner = self.0.get();
            if (*inner).transed_task_handle_for_mutex().unwrap().clone()
                == GetCurrentTaskHandle!()
            {
                traceGIVE_MUTEX_RECURSIVE!(*inner);
                (*inner).vQueueUnionDecrease();
                if (*inner).IsQueueUnionZero() {
                    (*inner).xQueueGenericReceive(semGIVE_BLOCK_TIME, false);
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
                return true;
            } else {
                traceGIVE_MUTEX_RECURSIVE_FAILED!(*inner);
                return false;
            }
        }
    }

    pub fn xSemaphoreTakeRecursive(&self, ticks_to_wait: TickType) -> bool {
        let mut xReturn: bool = false;
        unsafe {
            let inner = self.0.get();
            traceTAKE_MUTEX_RECURSIVE!(*inner);
            trace!("Ready to get recursive mutex holder");
            let mutex_holder = (*inner).transed_task_handle_for_mutex();
            trace!("Get recursive mutex holder successfully");
            if mutex_holder.is_some()
            {
                if mutex_holder.unwrap().clone() == GetCurrentTaskHandle!(){
                    trace!("Not First Time get this mutex");
                    (*inner).vQueueUnionIncrease();
                    xReturn = false;
                }
            } 
             else {
                trace!("First Time get this mutex");
                match (*inner).xQueueGenericSend(None, ticks_to_wait, queueSEND_TO_BACK) {
                    Ok(x) => {
                        (*inner).vQueueUnionIncrease();
                        xReturn = true;
                    }
                    Err(x) => {
                        traceTAKE_MUTEX_RECURSIVE_FAILED!(*inner);
                        xReturn = false;
                    }
                }
            }
        }
        return xReturn;
    }

    pub fn uxSemaphoreGetCount (&self) -> UBaseType {
        unsafe {
            let inner = self.0.get();
            (*inner).uxGetRecursiveCount()
        }
    }
}
