use crate::list;
use crate::list::ListLink;
use crate::port::*;
use crate::kernel::*;
use crate::projdefs::pdFALSE;
use crate::tasks::*;
use crate::global::*;
use crate::*;
use crate::projdefs;
use log::{error, warn, info, debug, trace};

#[cfg(not(feature = "configUSE_16_BIT_TICKS"))]
pub const taskEVENT_LIST_ITEM_VALUE_IN_USE: TickType_t = 0x80000000;

pub fn xTaskRemoveFromEventList(event_list: &ListLink) -> bool {
    let unblocked_tcb = list::get_owner_of_head_entry(event_list);
    let mut xreturn: bool = false;

    list::uxListRemove(unblocked_tcb.GetEventListItem());

    if GetSchedulerSuspended!() == 0{
        list::uxListRemove(unblocked_tcb.GetStateListItem());
        unblocked_tcb.AddTaskToReadyList().unwrap();
    } else {
        list::vListInsertEnd(&PENDING_READY_LIST, &unblocked_tcb.GetEventListItem());
    }

    if unblocked_tcb.GetPriority() > GetCurrentTaskPriority!() {
        xreturn = true;
        SetYieldPending!(true);
    } else {
        xreturn = false;
    }

    {
        #[cfg(feature = "configUSE_TICKLESS_IDLE")]
        ResetNextTaskUnblockTime();
    }

    trace!("xreturn is {}", xreturn);
    xreturn
}

pub fn vTaskMissedYield() {
    SetYieldPending!(false);
}

#[derive(Debug, Default)]
pub struct time_out {
    overflow_count: BaseType_t,
    time_on_entering: TickType_t,
}

pub fn vTaskSetTimeOutState(pxtimeout: &mut time_out)
{
    pxtimeout.overflow_count = GetNumOfOverflows!();
    pxtimeout.time_on_entering = GetTickCount!();
}

pub fn xTaskCheckForTimeOut(pxtimeout: &mut time_out, ticks_to_wait: &mut TickType_t) -> bool {
    trace!("time_out is {:?}", pxtimeout);
    trace!("ticks_to_wait is {}", ticks_to_wait);
    let mut xreturn: bool = false;
    taskENTER_CRITICAL!();
    {
        let const_tick_count: TickType_t = GetTickCount!();
        trace!("Tick_count is {}", const_tick_count);
        let unwrapped_cur = GetCurrentTaskHandle!();
        let mut cfglock1 = false;
        let mut cfglock2 = false;
/*
        {
            #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
            cfglock1 = true;
        }

        {
            #[cfg(feature = "INCLUDE_vTaskSuspend")]
            cfglock2 = true;
        }*/

        // if cfglock1 && unwrapped_cur.GetDelayAborted() {
        //     unwrapped_cur.SetDelayAborted(false);
        //     xreturn = true;
        // }

        if cfglock2 && *ticks_to_wait == crate::port::portMAX_DELAY {
            xreturn = false;
        }

        if GetNumOfOverflows!() != pxtimeout.overflow_count
            && const_tick_count >= pxtimeout.time_on_entering
        {
            trace!("IF");
            xreturn = true;
        } else if const_tick_count - pxtimeout.time_on_entering < *ticks_to_wait {
            trace!("ELSE IF");
            *ticks_to_wait -= const_tick_count - pxtimeout.time_on_entering;
            vTaskSetTimeOutState(pxtimeout);
            xreturn = false;
        } else {
            trace!("ELSE");
            xreturn = true;
        }
    }
    taskEXIT_CRITICAL!();
    xreturn
}

pub fn vTaskPlaceOnEventList(event_list: &ListLink, ticks_to_wait: TickType_t) {
    let unwrapped_cur = GetCurrentTaskHandle!();
    trace!("INSERT");
    list::vListInsert(event_list, &unwrapped_cur.GetEventListItem());
    trace!("INSERT SUCCEEDED");
    AddCurrentTaskToDelayedList(ticks_to_wait, true);
    trace!("ADD SUCCEEDED");
}


#[cfg(feature = "configUSE_MUTEXES")]
pub fn pvTaskIncrementMutexHeldCount() -> Option<TaskHandle> {
    match GetCurrentTaskHandleWrapped!() {
        Some(current_task) => {
            let new_val = current_task.GetMutexHeldCount() + 1;
            current_task.SetMutexHeldCount(new_val);
            Some(current_task.clone())
        }
        None => None,
    }
}

#[cfg(feature = "configUSE_MUTEXES")]
pub fn vTaskPriorityInherit(mutex_holder: Option<TaskHandle>)
{
    trace!("Enter function 'TaskPriorityInherit'");
    if mutex_holder.IsSome() {
        trace!("Mutex holder exists!");
        let task = mutex_holder.Unwrap();
        let current_task_priority = GetCurrentTaskPriority!();
        let this_task_priority = task.GetPriority();

        if this_task_priority < current_task_priority {
            trace!("change priority!");
            let event_list_item = task.GetEventListItem();
            if (list::listGET_LIST_ITEM_VALUE(&event_list_item) & taskEVENT_LIST_ITEM_VALUE_IN_USE) == 0
            {
                let new_item_val = (configMAX_PRIORITIES!() - current_task_priority) as TickType_t;
                list::listSET_LIST_ITEM_VALUE(&event_list_item, new_item_val);
            } else {
                mtCOVERAGE_TEST_MARKER!()
            }
            let state_list_item = task.GetStateListItem();
            if list::is_contained_within(
                &READY_TASK_LISTS[this_task_priority as usize],
                &state_list_item,
            ) {
                if list::uxListRemove(state_list_item) == 0 {
                    taskRESET_READY_PRIORITY!(this_task_priority);
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
                task.SetPriority(current_task_priority);
                task.AddTaskToReadyList().Unwrap();
            }
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }
}

#[cfg(feature = "configUSE_MUTEXES")]
pub fn xTaskPriorityDisinherit(mutex_holder: Option<TaskHandle>) -> bool {
    let mut ret_val: bool = false;
    trace!("Enter function 'xTaskPriorityDisinherit'");
    if let Some(task) = mutex_holder {

        assert!(task == get_current_task_handle!());

        let mutex_held = task.GetMutexHeldCount();
        assert!(mutex_held > 0);
        let mutex_held = mutex_held - 1;
        task.SetMutexHeldCount(mutex_held);

        let this_task_priority = task.GetPriority();
        let this_task_base_priority = task.GetBasePriority();
        if this_task_priority != this_task_base_priority {

            if mutex_held == 0 {
                let state_list_item = task.GetStateListItem();
                if list::uxListRemove(state_list_item) == 0 {
                    taskRESET_READY_PRIORITY!(this_task_priority);
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
                traceTASK_PRIORITY_DISINHERIT!(&task, this_task_base_priority);
                task.SetPriority(this_task_base_priority);

                let new_item_val = (configMAX_PRIORITIES!() - this_task_priority) as TickType_t;
                list::listSET_LIST_ITEM_VALUE(&task.get_event_list_item(), new_item_val);
                task.AddTaskToReadyList().Unwrap();

                ret_val = true;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }

    ret_val
}

