use crate::list;
use crate::port::*;
use crate::projdefs::pdFALSE;
use crate::tasks::{TaskHandle, TCB};
use crate::global::{PENDING_READY_LIST, DELAYED_TASK_LIST, READY_TASK_LISTS, OVERFLOW_DELAYED_TASK_LIST};
use crate::*;
// use crate::{GetSchedulerSuspended};
// use crate::{mtCOVERAGE_TEST_MARKER, traceTASK_INCREMENT_TICK, traceTASK_SWITCHED_OUT, traceTASK_SWITCHED_IN};

//use log::{trace};

// returned by xTaskGetSchedulerState()
pub enum SchedulerState {
    NotStarted,
    Suspended,
    Running,
}

// 宏定义：上下文切换
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

// 宏定义：进入临界区
#[macro_export]
macro_rules! taskENTER_CRITICAL {
    () => {
        portENTER_CRITICAL!()
    };
}

// 宏定义：退出临界区
/// Nothing
#[macro_export]
macro_rules! taskEXIT_CRITICAL {
    () => {
        portEXIT_CRITICAL!();
    };
}

#[macro_export]
macro_rules! taskENTER_CRITICAL_FROM_ISR {
    () => {
        portSET_INTERRUPT_MASK_FROM_ISR!()
    };
}



#[macro_export]
macro_rules! taskEXIT_CRITICAL_FROM_ISR {
    ($x: expr) => {
        portCLEAR_INTERRUPT_MASK_FROM_ISR!($x)
    };
}

// 宏定义：禁中断
#[macro_export]
macro_rules! taskDISABLE_INTERRUPTS {
    () => {
        portDISABLE_INTERRUPTS!()
    };
}

// 宏定义：使能中断
#[macro_export]
macro_rules! taskENABLE_INTERRUPTS {
    () => {
        portENABLE_INTERRUPTS!()
    };
}

// 开始调度任务
pub fn TaskStartScheduler() {
    CreateIdleTask();

    #[cfg(feature = "configUSE_TIMERS")]
    CreateTimerTask();

    InitializeScheduler();
}

// 创建 idle task.
pub fn CreateIdleTask() -> TaskHandle {
    println!("number: {}", GetCurrentNumberOfTasks!());
    let idle_task_fn = || {
        loop {
            trace!("Idle Task running");

            // 检测是否有任务结束；若有，则释放其 TCB 与栈
            CheckTasksWaitingTermination();

            // 不使能抢占时，应在此处切换任务
            #[cfg(not(feature = "configUSE_PREEMPTION"))]
            taskYIELD!();

            {
                #![cfg(all(feature = "configUSE_PREEMPTION", feature = "configIDLE_SHOULD_YIELD"))]
                // 当使用抢占式调度时，相同优先级的任务会进行时间片轮转。
                // 如果一个与空闲优先级共享的任务准备好运行，那么空闲任务应该在时间片结束之前主动让出CPU
                if list::listCURRENT_LIST_LENGTH(&READY_TASK_LISTS[0]) > 1 {
                    taskYIELD!();
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            }

            {
                #![cfg(feature = "configUSE_IDLE_HOOK")]

                // 在空闲任务中调用用户定义的函数。可用于添加后台功能
                // vApplicationIdleHook();
                trace!("Idle Task running");
            }
        }
    };
    TCB::new().INITIALIZE(idle_task_fn).unwrap_or_else(|err| panic!("Idle task creation failed with error: {:?}", err))
}
//     TCB::new().SetPriority(0).SetName("Idle").INITIALIZE(idle_task_fn).unwrap_or_else(|err| panic!("Idle task creation failed with error: {:?}", err))
// }

fn CheckTasksWaitingTermination() {
    // TODO: Wait for task_delete.
}

// 创建 timer task
fn CreateTimerTask() {
    // TODO: This function relies on the software timer, which we may not implement.
    // timer::CreateTimerTask()
    // On fail, panic!("No enough heap space to allocate timer task.");
}

/// 开始调度，调用 port_start_scheduler() ，不会返回
/// vTaskStartScheduler()
fn InitializeScheduler() {
    // 禁中断
    // 第一个任务开始时，会自动启用中断
    portDISABLE_INTERRUPTS!();

    SetNextTaskUnblockTime!(crate::port::portMAX_DELAY);
    SetSchedulerRunning!(true);
    SetTickCount!(0);

    
    // 如果定义了configGENERATE_RUN_TIME_STATS，则必须定义以下宏来配置用于生成运行时间计数器时间基准的定时器/计数器
    portCONFIGURE_TIMER_FOR_RUN_TIME_STATS!();

    // 启动调度器，调用外部接口（涉及硬件操作）
    if crate::port::port_start_scheduler() != pdFALSE {
        // 不应运行到这里
    } else {
        // 当有任务调用 xTaskEndScheduler() 后运行到这里
    }
}

// 终止
pub fn TaskEndScheduler() {
    // 停止调度器中断，并调用外部调度器结束接口
    portDISABLE_INTERRUPTS!();
    SetSchedulerRunning!(false);
    crate::port::port_end_scheduler();
}

// 暂停调度器，此时不发生上下文切换
pub fn TaskSuspendAll() {
    // SCHEDULER_SUSPENDED 加 1
    SetSchedulerSuspended!(GetSchedulerSuspended!() + 1);
}

// 恢复调度器
// 若恢复调度器引起了上下文切换，则返回 true
pub fn TaskResumeAll() -> bool {
    trace!("resume_all called!");
    let mut already_yielded = false;

    assert!(
        GetSchedulerSuspended!() > pdFALSE as UBaseType_t,
        "The call to TaskResumeAll() does not match \
         a previous call to vTaskSuspendAll()."
    );

    // 在调度器被暂停期间，可能会发生中断服务程序（ISR）导致任务从事件列表中被移除的情况。如果是这种情况，被移除的任务将会被添加到 xPendingReadyList 中。一旦调度器被恢复，就可以安全地将所有处于等待状态的任务从该列表中移动到相应的就绪列表中。
    taskENTER_CRITICAL!();
    {
        // SCHEDULER_SUSPENDED 减 1
        SetSchedulerSuspended!(GetSchedulerSuspended!() - 1);
        println!(
            "GetCurrentNumberOfTasks: {}",
            GetCurrentNumberOfTasks!()
        );
        if GetSchedulerSuspended!() == pdFALSE as UBaseType_t {
            if GetCurrentNumberOfTasks!() > 0 {
                trace!(
                    "Current number of tasks is: {}, move tasks to ready list.",
                    GetCurrentNumberOfTasks!()
                );
                // 将任何已准备好的任务从等待列表中移动到就绪列表中
                if MoveTasksToReadyList() {
                    // 在调度器挂起期间，一个任务被解除阻塞，可能会阻止下一个解除阻塞时间被重新计算，在这种情况下，现在重新计算它
                    ResetNextTaskUnblockTime();
                }

                // 如果在调度器挂起期间发生了任何滴答，则应立即处理它们
                ProcessPendedTicks();

                if GetYieldPending!() {
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
        if task_handle.GetPriority() >= GetCurrentTaskPriority!() {
            SetYieldPending!(true);
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
        SetNextTaskUnblockTime!(crate::port::portMAX_DELAY);
    } else {
        //否则，它获取延迟列表头部的任务句柄并设置下一个任务解锁时间
        let task_handle = list::get_owner_of_head_entry(&DELAYED_TASK_LIST);
        SetNextTaskUnblockTime!(list::listGET_LIST_ITEM_VALUE(
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
                SetYieldPending!(true);
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
    let next_task_unblock_time = GetNextTaskUnblockTime!();
    assert!(cur_tick_count + ticks_to_jump <= next_task_unblock_time);
    SetTickCount!(cur_tick_count + ticks_to_jump);
    traceINCREASE_TICK_COUNT!(xTicksToJump);
}

pub fn TaskSwitchContext() {
    //用于切换上下文
    if GetSchedulerSuspended!() > pdFALSE as UBaseType_t {
        //首先检查调度程序是否被挂起
        //如果是，则执行 yield
        SetYieldPending!(true);
    } else {
        //否则，它将 yield 设置为 false
        SetYieldPending!(false);
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
    let mut top_priority: UBaseType_t = GetTopReadyPriority!();

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

    trace!("SCHEDULER_SUSP is {}", GetSchedulerSuspended!());
    //检查调度程序是否被挂起
    if GetSchedulerSuspended!() == pdFALSE as UBaseType_t {
        //如果没有被挂起，则递增滴答计数并检查滴答计数是否等于零
        let const_tick_count = GetTickCount!() + 1;
        SetTickCount!(const_tick_count);

        //如果是，则切换延迟列表
        if const_tick_count == 0 {
            SwitchDelayedList!();
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }

        //检查滴答计数是否大于或等于下一个任务解锁时间
        if const_tick_count >= GetNextTaskUnblockTime!() {
            trace!("UNBLOCKING!");
            loop {
                //如果是，则使用循环检查延迟列表是否为空
                if list::listLIST_IS_EMPTY(&DELAYED_TASK_LIST) {
                    //如果为空，则将下一个任务解锁时间设置为最大可能值并退出循环
                    SetNextTaskUnblockTime!(crate::port::portMAX_DELAY);
                    break;
                } else {
                    //否则，它获取延迟列表头部的任务句柄并设置下一个任务解锁时间
                    let delay_head_entry_owner = list::get_owner_of_head_entry(&DELAYED_TASK_LIST);
                    let task_handle = delay_head_entry_owner;
                    let state_list_item = task_handle.GetStateListItem();
                    let event_list_item = task_handle.GetEventListItem();
                    let item_value = list::listGET_LIST_ITEM_VALUE(&state_list_item);

                    if const_tick_count < item_value {
                        SetNextTaskUnblockTime!(item_value);
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
