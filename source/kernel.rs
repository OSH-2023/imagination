// kernel.rs, FreeRTOS scheduler control APIs.
// This file is created by Fan Jinhao.
// Functions defined in this file are explained in Chapter 9 and 10.
use crate::list;
use crate::port::UBaseType_t;
use crate::projdefs::pdFALSE;
use crate::task::{TaskHandle, TCB};
use crate::task_global::*;
use crate::*; // TODO: Is this line necessary?
              // use crate::task_control::TCB;

/* Definitions returned by xTaskGetSchedulerState().
 * The originial definitons are C constants, we changed them to enums.
 */

pub enum SchedulerState {
    NotStarted,
    Suspended,
    Running,
}

/// Macro for forcing a context switch.
///
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
#[macro_export]
macro_rules! taskYIELD {
    () => {
        portYIELD!()
    };
}

#[macro_export]
macro_rules! taskYIELD_IF_USING_PREEMPTION {
    () => {
        #[cfg(feature = "configUSE_PREEMPTION")]
        portYIELD_WITHIN_API!();
    };
}

/// Macro to mark the start of a critical code region.  Preemptive context
/// switches cannot occur when in a critical region.
///
/// NOTE: This may alter the stack (depending on the portable implementation)
/// so must be used with care!
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
#[macro_export]
macro_rules! taskENTER_CRITICAL {
    () => {
        portENTER_CRITICAL!()
    };
}

#[macro_export]
macro_rules! taskENTER_CRITICAL_FROM_ISR {
    () => {
        portSET_INTERRUPT_MASK_FROM_ISR!()
    };
}

/// Macro to mark the end of a critical code region.  Preemptive context
/// switches cannot occur when in a critical region.
///
/// NOTE: This may alter the stack (depending on the portable implementation)
/// so must be used with care!
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
#[macro_export]
macro_rules! taskEXIT_CRITICAL {
    () => {
        portEXIT_CRITICAL!()
    };
}

#[macro_export]
macro_rules! taskEXIT_CRITICAL_FROM_ISR {
    ($x: expr) => {
        portCLEAR_INTERRUPT_MASK_FROM_ISR!($x)
    };
}

/// Macro to disable all maskable interrupts.
/// * Implemented by: Fan Jinhao.
/// * C implementation: task.h
///
/// # Arguments
///
/// # Return
///
/// Nothing

#[macro_export]
macro_rules! taskDISABLE_INTERRUPTS {
    () => {
        portDISABLE_INTERRUPTS!()
    };
}

/// Macro to enable microcontroller interrupts.
///
/// * Implemented by: Fan Jinhao.
/// * C implementation: task.h
///
/// # Arguments
///
/// # Return
///
/// Nothing

#[macro_export]
macro_rules! taskENABLE_INTERRUPTS {
    () => {
        portENABLE_INTERRUPTS!()
    };
}

///
/// Starts the real time kernel tick processing.  After calling the kernel
/// has control over which tasks are executed and when.
///
/// See the demo application file main.c for an example of creating
/// tasks and starting the kernel.
///
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
///
pub fn TaskStartScheduler() {
    CreateIdleTask();

    #[cfg(feature = "configUSE_TIMERS")]
    CreateTimerTask();

    InitializeScheduler();
}

/// The fist part of TaskStartScheduler(), creates the idle task.
/// Will panic if task creation fails.
/// * Implemented by: Fan Jinhao.
/// * C implementation: tasks.c 1831-1866
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
pub fn CreateIdleTask() -> TaskHandle {
    println!("number: {}", get_current_number_of_tasks!());
    let idle_task_fn = || {
        loop {
            trace!("Idle Task running");
            /* THIS IS THE RTOS IDLE TASK - WHICH IS CREATED AUTOMATICALLY WHEN THE
            SCHEDULER IS STARTED. */

            /* See if any tasks have deleted themselves - if so then the idle task
            is responsible for freeing the deleted task's TCB and stack. */
            CheckTasksWaitingTermination();

            /* If we are not using preemption we keep forcing a task switch to
            see if any other task has become available.  If we are using
            preemption we don't need to do this as any task becoming available
            will automatically get the processor anyway. */
            #[cfg(not(feature = "configUSE_PREEMPTION"))]
            taskYIELD!();

            {
                #![cfg(all(feature = "configUSE_PREEMPTION", feature = "configIDLE_SHOULD_YIELD"))]
                /* When using preemption tasks of equal priority will be
                timesliced.  If a task that is sharing the idle priority is ready
                to run then the idle task should yield before the end of the
                timeslice.

                A critical region is not required here as we are just reading from
                the list, and an occasional incorrect value will not matter.  If
                the ready list at the idle priority contains more than one task
                then a task other than the idle task is ready to execute. */
                if list::listCURRENT_LIST_LENGTH(&READY_TASK_LISTS[0]) > 1 {
                    taskYIELD!();
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            }

            {
                #![cfg(feature = "configUSE_IDLE_HOOK")]
                // TODO: Use IdleHook
                // extern void vApplicationIdleHook( void );

                /* Call the user defined function from within the idle task.  This
                allows the application designer to add background functionality
                without the overhead of a separate task.
                NOTE: vApplicationIdleHook() MUST NOT, UNDER ANY CIRCUMSTANCES,
                CALL A FUNCTION THAT MIGHT BLOCK. */
                // vApplicationIdleHook();
                trace!("Idle Task running");
            }
        }
    };

    TCB::new()
        .priority(0)
        .name("Idle")
        .initialise(idle_task_fn)
        .unwrap_or_else(|err| panic!("Idle task creation failed with error: {:?}", err))
}

fn CheckTasksWaitingTermination() {
    // TODO: Wait for task_delete.
}

/// The second (optional) part of TaskStartScheduler(),
/// creates the timer task. Will panic if task creation fails.
/// * Implemented by: Fan Jinhao.
/// * C implementation: tasks.c 1868-1879
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
fn CreateTimerTask() {
    // TODO: This function relies on the software timer, which we may not implement.
    // timer::CreateTimerTask()
    // On fail, panic!("No enough heap space to allocate timer task.");
}

/// The third part of task_step_scheduler, do some initialziation
/// and call port_start_scheduler() to set up the timer tick.
/// vTaskStartScheduler()
fn InitializeScheduler() {
    /* Interrupts are turned off here, to ensure a tick does not occur
    before or during the call to xPortStartScheduler().  The stacks of
    the created tasks contain a status word with interrupts switched on
    so interrupts will automatically get re-enabled when the first task
    starts to run. */
    portDISABLE_INTERRUPTS!();

    // TODO: NEWLIB

    set_next_task_unblock_time!(port::portMAX_DELAY);
    set_scheduler_running!(true);
    set_tick_count!(0);

    /* If configGENERATE_RUN_TIME_STATS is defined then the following
    macro must be defined to configure the timer/counter used to generate
    the run time counter time base. */
    portCONFIGURE_TIMER_FOR_RUN_TIME_STATS!();

    // 启动调度器 
    /* Setting up the timer tick is hardware specific and thus in the
    portable interface. */
    if port::port_start_scheduler() != pdFALSE {
        /* Should not reach here as if the scheduler is running the
        function will not return. */
    } else {
        // TODO: Maybe a trace here?
        /* Should only reach here if a task calls xTaskEndScheduler(). */
    }
}

/// NOTE:  At the time of writing only the x86 real mode port, which runs on a PC
/// in place of DOS, implements this function.
///
/// Stops the real time kernel tick.  All created tasks will be automatically
/// deleted and multitasking (either preemptive or cooperative) will
/// stop.  Execution then resumes from the point where vTaskStartScheduler ()
/// was called, as if vTaskStartScheduler () had just returned.
///
/// See the demo application file main. c in the demo/PC directory for an
/// example that uses vTaskEndScheduler ().
///
/// vTaskEndScheduler () requires an exit function to be defined within the
/// portable layer (see vPortEndScheduler () in port. c for the PC port).  This
/// performs hardware specific operations such as stopping the kernel tick.
///
/// vTaskEndScheduler () will cause all of the resources allocated by the
/// kernel to be freed - but will not free resources allocated by application
/// tasks.
///
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing

pub fn TaskEndScheduler() {
    /* Stop the scheduler interrupts and call the portable scheduler end
    routine so the original ISRs can be restored if necessary.  The port
    layer must ensure interrupts enable bit is left in the correct state. */
    portDISABLE_INTERRUPTS!();
    set_scheduler_running!(false);
    port::port_end_scheduler();
}

/// Suspends the scheduler without disabling interrupts.  Context switches will
/// not occur while the scheduler is suspended.
///
/// After calling vTaskSuspendAll () the calling task will continue to execute
/// without risk of being swapped out until a call to xTaskResumeAll () has been
/// made.
///
/// API functions that have the potential to cause a context switch (for example,
/// vTaskDelayUntil(), xQueueSend(), etc.) must not be called while the scheduler
/// is suspended.
///
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// Nothing
pub fn TaskSuspendAll() {
    /* A critical section is not required as the variable is of type
    BaseType_t.  Please read Richard Barry's reply in the following link to a
    post in the FreeRTOS support forum before reporting this as a bug! -
    http://goo.gl/wu4acr */

    // Increment SCHEDULER_SUSPENDED.
    set_scheduler_suspended!(get_scheduler_suspended!() + 1);
}

/// Resumes scheduler activity after it was suspended by a call to
/// vTaskSuspendAll().
///
/// xTaskResumeAll() only resumes the scheduler.  It does not unsuspend tasks
/// that were previously suspended by a call to vTaskSuspend().
/// * Implemented by: Fan Jinhao.
/// * C implementation:
///
/// # Arguments
///
///
/// # Return
///
/// If resuming the scheduler caused a context switch then true is
/// returned, otherwise false is returned.
pub fn TaskResumeAll() -> bool {
    trace!("resume_all called!");
    let mut already_yielded = false;

    // TODO: This is a recoverable error, use Result<> instead.
    assert!(
        get_scheduler_suspended!() > pdFALSE as UBaseType_t,
        "The call to TaskResumeAll() does not match \
         a previous call to vTaskSuspendAll()."
    );

    /* It is possible that an ISR caused a task to be removed from an event
    list while the scheduler was suspended.  If this was the case then the
    removed task will have been added to the xPendingReadyList.  Once the
    scheduler has been resumed it is safe to move all the pending ready
    tasks from this list into their appropriate ready list. */
    taskENTER_CRITICAL!();
    {
        // Decrement SCHEDULER_SUSPENDED.
        set_scheduler_suspended!(get_scheduler_suspended!() - 1);
        println!(
            "get_current_number_of_tasks: {}",
            get_current_number_of_tasks!()
        );
        if get_scheduler_suspended!() == pdFALSE as UBaseType_t {
            if get_current_number_of_tasks!() > 0 {
                trace!(
                    "Current number of tasks is: {}, move tasks to ready list.",
                    get_current_number_of_tasks!()
                );
                /* Move any readied tasks from the pending list into the
                appropriate ready list. */
                if MoveTasksToReadyList() {
                    /* A task was unblocked while the scheduler was suspended,
                    which may have prevented the next unblock time from being
                    re-calculated, in which case re-calculate it now.  Mainly
                    important for low power tickless implementations, where
                    this can prevent an unnecessary exit from low power
                    state. */
                    ResetNextTaskUnblockTime();
                }

                /* If any ticks occurred while the scheduler was suspended then
                they should be processed now.  This ensures the tick count does
                not slip, and that any delayed tasks are resumed at the correct
                time. */
                ProcessPendedTicks();

                if get_yield_pending!() {
                    {
                        #![cfg(feature = "configUSE_PREEMPTION")]
                        already_yielded = true;
                    }

                    taskYIELD_IF_USING_PREEMPTION!();
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            }
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    }

    trace!("Already yielded is {}", already_yielded);
    already_yielded
}



// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //
// ***************************************************************************************** //



fn MoveTasksToReadyList() -> bool {
    //从 PENDING_READY_LIST 中移动到就绪列表中
    let mut has_unblocked_task = false;
    while !list::listLIST_IS_EMPTY(&PENDING_READY_LIST) {
        //使用 while 循环检查 PENDING_READY_LIST 是否为空
        //如果不为空，它将 has_unblocked_task 设置为 true 并获取列表头部的任务句柄
        trace!("PEDING_LIST not empty");
        has_unblocked_task = true;
        let task_handle = list::get_owner_of_head_entry(&PENDING_READY_LIST);
        //获取任务句柄的事件列表项和状态列表项，并从列表中删除它们。然后，它将任务添加到就绪列表中
        let event_list_item = task_handle.GetEventListItem();
        let state_list_item = task_handle.GetStateListItem();

        list::uxListRemove(state_list_item);
        list::uxListRemove(event_list_item);

        task_handle.AddTaskToReadyList().unwrap();

        //检查移动的任务的优先级是否高于当前任务的优先级。如果是，则执行 yield
        if task_handle.get_priority() >= GetCurrentTaskPriority!() {
            set_yield_pending!(true);
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    }
    has_unblocked_task
}

fn ResetNextTaskUnblockTime() {
    //重置下一个任务解锁时间
    if list::listLIST_IS_EMPTY(&DELAYED_TASK_LIST) {
        //首先检查 DELAYED_TASK_LIST 是否为空
        //如果为空，则将 xNextTaskUnblockTime 设置为最大可能值
        set_next_task_unblock_time!(port::portMAX_DELAY);
    } else {
        //否则，它获取延迟列表头部的任务句柄并设置下一个任务解锁时间
        let task_handle = list::get_owner_of_head_entry(&DELAYED_TASK_LIST);
        set_next_task_unblock_time!(list::listGET_LIST_ITEM_VALUE(
            &task_handle.GetStateListItem()
        ));
    }
}

fn ProcessPendedTicks() {
    //用于处理pended_ticks
    trace!("Processing pended ticks");
    let mut pended_counts = GetPendedTicks!();
    //首先获取pended_ticks并检查是否大于零
    if pended_counts > 0 {
        //如果是，则使用循环递增计时器计数
        //并递减挂起计数
        loop {
            if TaskIncrementTick() {
                //执行 yield
                set_yield_pending!(true);
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }

            pended_counts -= 1;

            if pended_counts <= 0 {
                break;
            }
        }
        SetPendedTicks!(0);
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }
}

#[cfg(feature = "configUSE_TICKLESS_IDLE")]
pub fn TaskStepTick(ticks_to_jump: TickType) {
   //用于在使用无滴答模式或低功耗模式时更新调度程序维护的滴答计数值
    let cur_tick_count = get_tick_count!(); 
    let next_task_unblock_time = get_next_task_unblock_time!();
    assert!(cur_tick_count + ticks_to_jump <= next_task_unblock_time);
    set_tick_count!(cur_tick_count + ticks_to_jump);
    traceINCREASE_TICK_COUNT!(xTicksToJump);
}

pub fn TaskSwitchContext() {
    //用于切换上下文
    if GetSchedulerSuepended!() > pdFALSE as UBaseType {
        //首先检查调度程序是否被挂起
        //如果是，则执行 yield
        set_yield_pending!(true);
    } else {
        //否则，它将 yield 设置为 false
        set_yield_pending!(false);
        traceTASK_SWITCHED_OUT!();

        #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
        GenerateContextSwitchStats();

        //检查栈溢出
        taskCHECK_FOR_STACK_OVERFLOW!();

        //选择最高优先级的任务来运行
        TaskSelectHighestPriorityTask();//选择最高优先级的任务
        traceTASK_SWITCHED_IN!();
    }
}

#[cfg(not(feature = "configUSE_PORT_OPTIMISED_TASK_SELECTION"))]
fn TaskSelectHighestPriorityTask() {
    //用于选择最高优先级的任务
    let mut top_priority: UBaseType = GetTopReadyPriority!();

    //首先获取最高优先级,使用 while 循环查找包含就绪任务的最高优先级队列
    while list::listLIST_IS_EMPTY(&READY_TASK_LISTS[top_priority as usize]) {
        assert!(top_priority > 0, "No task found with a non-zero priority");
        top_priority -= 1;
    }

    //获取下一个任务并设置当前任务句柄和最高就绪优先级
    let next_task = list::get_owner_of_next_entry(&READY_TASK_LISTS[top_priority as usize]);

    trace!("Next task is {}", next_task.GetName());
    SetCurrentTaskHandle!(next_task);

    SetTopReadyPriority!(top_priority);
}

#[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
fn GenerateContextSwitchStats() {
    //用于生成上下文切换统计信息
    //首先获取总运行时间并设置总运行时间
    let total_run_time = portGET_RUN_TIME_COUNTER_VALUE!() as u32;
    trace!("Total runtime: {}", total_run_time);
    set_total_run_time!(total_run_time);

    //将任务运行时间增加到累计时间中
    let task_switched_in_time = GetTaskSwitchInTime!();
    if total_run_time > task_switched_in_time {
        let current_task = GetCurrentTaskHandle!();
        let old_run_time = current_task.GetRunTime();
        current_task.set_run_time(old_run_time + total_run_time - task_switched_in_time);
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }
    set_task_switch_in_time!(total_run_time);
}

pub fn TaskIncrementTick() -> bool {
    //用于递增计时器计数并检查新的计时器计数值是否会导致任何任务被解除阻塞
    let mut switch_required = false;
    traceTASK_INCREMENT_TICK!(get_tick_count!());

    trace!("SCHEDULER_SUSP is {}", GetSchedulerSuepended!());
    //检查调度程序是否被挂起
    if GetSchedulerSuspended!() == pdFALSE as UBaseType {
        //如果没有被挂起，则递增滴答计数并检查滴答计数是否等于零
        let const_tick_count = get_tick_count!() + 1;
        set_tick_count!(const_tick_count);

        //如果是，则切换延迟列表
        if const_tick_count == 0 {
            switch_delayed_lists!();
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }

        //检查滴答计数是否大于或等于下一个任务解锁时间
        if const_tick_count >= get_next_task_unblock_time!() {
            trace!("UNBLOCKING!");
            loop {
                //如果是，则使用循环检查延迟列表是否为空
                if list::listLIST_IS_EMPTY(&DELAYED_TASK_LIST) {
                    //如果为空，则将下一个任务解锁时间设置为最大可能值并退出循环
                    set_next_task_unblock_time!(port::portMAX_DELAY);
                    break;
                } else {
                    //否则，它获取延迟列表头部的任务句柄并设置下一个任务解锁时间
                    let delay_head_entry_owner = list::get_owner_of_head_entry(&DELAYED_TASK_LIST);
                    let task_handle = delay_head_entry_owner;
                    let state_list_item = task_handle.GetStateListItem();
                    let event_list_item = task_handle.GetEventListItem();
                    let item_value = list::listGET_LIST_ITEM_VALUE(&state_list_item);

                    if const_tick_count < item_value {
                        set_next_task_unblock_time!(item_value);
                        break;
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }

                    //将任务从block list移除
                    list::uxListRemove(state_list_item.clone());

                    //检查任务是否也在等待事件
                    //如果在等待事件，就从event_list_item删除
                    if list::get_list_item_container(&event_list_item).is_some() {
                        list::uxListRemove(event_list_item.clone());
                    }
                    //将解除阻塞的任务添加到适当的就绪列表中
                    task_handle.AddTaskToReadyList().unwrap();

                    //检查是否启用了抢占,如果启用了抢占
                    //则仅当解除阻塞的任务的优先级等于或高于当前执行任务的优先级时才执行上下文切换。
                    {
                        #![cfg(feature = "configUSE_PREEMPTION")]
                        if task_handle.GetPriority() >= GetCurrentTaskPriority!() {
                            switch_required = true;
                        } else {
                            mtCOVERAGE_TEST_MARKER!();
                        }
                    }
                }
            }
        }

        //检查当前运行任务的优先级是否与其他就绪任务相同
        //如果是，则在启用抢占和时间片时执行上下文切换
        {
            #![cfg(all(feature = "configUSE_PREEMPTION", feature = "configUSE_TIME_SLICING"))]
            let cur_task_pri = GetCurrentTaskPriority!();

            if list::listCURRENT_LIST_LENGTH(&READY_TASK_LISTS[cur_task_pri as usize]) > 1 {
                switch_required = true;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }

        {
            #![cfg(feature = "configUSE_TICK_HOOK")]
            //检查挂起的计数是否为零
            if GetPendedTicks!() == 0 {
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }
    } else {
        //如果调度程序被挂起了
        //递增挂起计数
        SetPendedTicks!(GetPendedTicks!() + 1);

        #[cfg(feature = "configUSE_TICK_HOOK")]
        vApplicationTickHook();

        #[cfg(feature = "configUSE_PREEMPTION")]
        {
            //检查是否启用了抢占,如果启用了抢占,则检查是否需要执行 yield
            //如果是，则执行上下文切换
            if GetYieldPending!() {
                switch_required = true;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }
    }
    switch_required
}

#[cfg(any(
    feature = "INCLUDE_xTaskGetSchedulerState",
    feature = "configUSE_TIMERS"
))]
pub fn TaskGetSchedulerState() -> SchedulerState {
    //用于获取调度程序状态
    if !get_scheduler_running!() {
        //首先检查调度程序是否正在运行
        SchedulerState::NotStarted
    } else {
        if GetSchedulerSuepended!() == pdFALSE as UBaseType {
            //否则，它检查调度程序是否被挂起
            SchedulerState::Running
        } else {
            SchedulerState::Suspended
        }
    }
}

#[cfg(not(feature = "configUSE_PORT_OPTIMISED_TASK_SELECTION"))]
#[macro_export]
macro_rules! taskRESET_READY_PRIORITY {
    //用于重置就绪优先级
    //仅在未使用端口优化的任务选择方法时才需要，因此在这种情况下被定义为空
    ($uxPriority: expr) => {};
}

#[cfg(not(feature = "configUSE_PORT_OPTIMISED_TASK_SELECTION"))]
#[macro_export]
macro_rules! taskRECORD_READY_PRIORITY {
    //用于记录就绪优先级
    ($uxPriority: expr) => {
        //接受一个参数 $uxPriority，表示要set的优先级
        //如果 $uxPriority 大于当前的最高就绪优先级，则将最高就绪优先级设置为 $uxPriority
        if $uxPriority > GetTopReadyPriority!() {
            SetTopReadyPriority!($uxPriority);
        }
    };
}