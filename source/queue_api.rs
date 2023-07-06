use crate::port::*;
use crate::queue::*;
use crate::queue_h::*;
use std::cell::UnsafeCell;

pub struct QueueHandle_t<T>(UnsafeCell<QueueDefinition<T>>)
    where
        T: Default + Clone;
// send, sync is used for sharing queue among threads
unsafe impl<T: Default + Clone> Send for QueueHandle_t<T> {}
unsafe impl<T: Default + Clone> Sync for QueueHandle_t<T> {}

impl<T> QueueHandle_t<T>
    where
        T: Default + Clone,
{
    pub fn xQueueCreate(length: UBaseType_t) -> Self {
        QueueHandle_t(UnsafeCell::new(QueueDefinition::new(
            length,
            QueueType::Base,
        )))
    }
    pub fn xQueueSend(&self, pvItemToQueue: T, xTicksToWait: TickType) -> Result<(), QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSend(pvItemToQueue, xTicksToWait, queueSEND_TO_BACK)
        }
    }

    pub fn xQueueSendToFront(
        &self,
        pvItemToQueue: T,
        xTicksToWait: TickType,
    ) -> Result<(), QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSend(pvItemToQueue, xTicksToWait, queueSEND_TO_FRONT)
        }
    }
    pub fn xQueueSendToBack(&self, pvItemToQueue: T, xTicksToWait: TickType) -> Result<(), QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSend(pvItemToQueue, xTicksToWait, queueSEND_TO_BACK)
        }
    }
    pub fn xQueueOverwrite(&self, pvItemToQueue: T) -> Result<(), QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSend(pvItemToQueue, 0, queueOVERWRITE)
        }
    }
    pub fn xQueueSendToFrontFromISR(&self, pvItemToQueue: T) -> (Result<(), QueueError>, bool) {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSendFromISR(pvItemToQueue, queueSEND_TO_FRONT)
        }
    }
    pub fn xQueueSendToBackFromISR(&self, pvItemToQueue: T) -> (Result<(), QueueError>, bool) {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSendFromISR(pvItemToQueue, queueSEND_TO_BACK)
        }
    }
    pub fn xQueueOverwriteFromISR(&self, pvItemToQueue: T) -> (Result<(), QueueError>, bool) {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericSendFromISR(pvItemToQueue, queueOVERWRITE)
        }
    }
    pub fn xQueueReceive(&self, xTicksToWait: TickType) -> Result<T, QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericReceive(xTicksToWait, false)
        }
    }
    pub fn xQueuePeek(&self, xTicksToWait: TickType) -> Result<T, QueueError> {
        unsafe {
            let inner = self.0.get();
            (*inner).xQueueGenericReceive(xTicksToWait, true)
        }
    }
}