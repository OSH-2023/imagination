use std::boxed::FnBox;
use std::mem;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct TCB {
    //* basic information
    StateListItem: ItemLink,
    EventListItem: ItemLink,
    pcTaskName: String,
    StackDepth: UBaseType,
    Priority: UBaseType,
    StackPointer: StackType,

    #[cfg(feature = "portCRITICAL_NESTING_IN_TCB")]
    critical_nesting: UBaseType,
    #[cfg(feature = "configUSE_MUTEXES")]
    base_priority: UBaseType,
    #[cfg(feature = "configUSE_MUTEXES")]
    mutexes_held: UBaseType,
    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    runtime_counter: TickType,
    #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
    notified_value: u32,
    #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
    notify_state: u8,
    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    delay_aborted: bool,
}

impl PartialEq for TCB {
    fn eq(&self, other: &Self) -> bool {
        self.stack_pos == other.stack_pos
    }
}

extern "C" fn run_wrapper(func_to_run: CVoidPointer) {
    info!(
        "Run_wrapper: The function is at position: {:X}",
        func_to_run as u64
    );
    unsafe {
        let func_to_run = Box::from_raw(func_to_run as *mut Box<FnBox() + 'static>);
        func_to_run();
    }
}

impl TCB {
    pub fn new() -> Self {
        TCB {
            StateListItem: Default::default(),
            EventListItem: Default::default(),
            task_priority: 1,       //initialized with 1
            StackDepth: configMINIMAL_STACK_SIZE!(),
            pcTaskName: String::from("Unnamed"),
            StackPointer: 0,

            #[cfg(feature = "portCRITICAL_NESTING_IN_TCB")]
            critical_nesting: 0,
            #[cfg(feature = "configUSE_MUTEXES")]
            base_priority: 0,
            #[cfg(feature = "configUSE_MUTEXES")]
            mutexes_held: 0,
            #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
            runtime_counter: 0,
            #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
            notified_value: 0,
            #[cfg(feature = "configUSE_TASK_NOTIFICATIONS")]
            notify_state: 0,
            #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
            delay_aborted: false,
        }
    }

    pub fn SetName(&mut self, name: &str) -> Self {
        self.pcTaskName = name.to_owned().to_string();
        Self
    }

    pub fn SetStackDepth(&mut self, stacksize: UBaseType) -> Self {
        self.ulStackDepth = stacksize;
        Self
    }

    pub fn SetPriority(mut self, priority: UBaseType) -> Self {
        if priority >= configMAX_PRIORITIES!() {
            warn!("Specified priority larger than system maximum priority, will be reduced.");
            info!(
                "MAX_PRIORITY is {}, but got {}",
                configMAX_PRIORITIES!() - 1,
                priority
            );
            self.task_priority = configMAX_PRIORITIES!() - 1;
        } else {
            self.task_priority = priority;
        }
        self
    }
    
    pub fn get_name(&self) -> String {
        self.task_name.clone()
    }

    pub fn get_priority(&self) -> UBaseType {
        self.task_priority.clone()
    }

    pub fn set_priority(&mut self, new_priority: UBaseType) {
        self.task_priority = new_priority;
    }

    pub fn get_state_list_item(&self) -> ItemLink {
        Arc::clone(&self.state_list_item)
    }

    pub fn get_event_list_item(&self) -> ItemLink {
        Arc::clone(&self.event_list_item)
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn get_run_time(&self) -> TickType {
        self.runtime_counter
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn set_run_time(&mut self, next_val: TickType) -> TickType {
        let prev_val: u32 = self.runtime_counter;
        self.runtime_counter = next_val;
        prev_val
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn get_delay_aborted(&self) -> bool {
        self.delay_aborted
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn set_delay_aborted(&mut self, next_val: bool) -> bool {
        let prev_val: bool = self.delay_aborted;
        self.delay_aborted = next_val;
        prev_val
    }
    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn get_mutex_held_count(&self) -> UBaseType {
        self.mutexes_held
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn set_mutex_held_count(&mut self, new_count: UBaseType) {
        self.mutexes_held = new_count;
    }

    pub fn get_base_priority(&self) -> UBaseType {
        self.base_priority
    }

    pub fn set_base_priority(&mut self, new_val: UBaseType) {
        self.base_priority = new_val
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

pub fn record_ready_priority(priority: UBaseType) {
    if priority > get_top_ready_priority!() {
        set_top_ready_priority!($priority);
    }
}

impl TaskHandle {
    pub fn get_event_list_item(&self) -> ItemLink {
        GetTCB_read(self).get_event_list_item()
    }

    pub fn get_state_list_item(&self) -> ItemLink {
        GetTCB_read(self).get_state_list_item()
    }

    pub fn get_name(&self) -> String {
        GetTCB_read(self).get_name()
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn get_run_time(&self) -> TickType {
        GetTCB_read(self).get_run_time()
    }

    #[cfg(feature = "configGENERATE_RUN_TIME_STATS")]
    pub fn set_run_time(&self, next_val: TickType) -> TickType {
        GetTCB_write(self).set_run_time(next_val)
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn get_delay_aborted(&self) -> bool {
        GetTCB_read(self).get_delay_aborted()
    }

    #[cfg(feature = "INCLUDE_xTaskAbortDelay")]
    pub fn set_delay_aborted(&self, next_val: bool) -> bool {
        GetTCB_write(self).set_delay_aborted(next_val)
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn get_mutex_held_count(&self) -> UBaseType {
        GetTCB_read(self).get_mutex_held_count()
    }

    #[cfg(feature = "configUSE_MUTEXES")]
    pub fn set_mutex_held_count(&self, new_count: UBaseType) {
        GetTCB_write(self).set_mutex_held_count(new_count)
    }

    pub fn get_base_priority(&self) -> UBaseType {
        GetTCB_read(self).get_base_priority()
    }

    pub fn set_base_priority(&self, new_val: UBaseType) {
        GetTCB_write(self).set_base_priority(new_val)
    }

    pub fn from_arc(arc: Arc<RwLock<TCB>>) -> Self {
        TaskHandle(arc)
    }
    pub fn from(tcb: TCB) -> Self {
        TaskHandle(Arc::new(RwLock::new(tcb)))
    }

    pub fn as_raw(self) -> ffi::xTaskHandle {
        Arc::into_raw(self.0) as *mut _
    }

    pub fn get_priority(&self) -> UBaseType {
        self.0.read().unwrap().get_priority()
    }

    pub fn set_priority(&self, new_priority: UBaseType) {
        self.GetTCB_write().set_priority(new_priority);
    }

    pub fn GetTCB_read(&self) -> Result<(), FreeRtosError>{
        match self.0.try_read() {
            Ok(handle) => handle,
            Err(_) => {
                warn!("TCB was locked, read failed");
                panic!("Task handle locked!");
            }
        }
    }

    pub fn GetTCB_write(&self) -> Result<(), FreeRtosError>{
        match self.0.try_write() {
            Ok(handle) => handle,
            Err(_) => {
                warn!("TCB was locked, write failed");
                panic!("Task handle locked!");
            }
        }
    }

    pub fn add_task_to_ready_list(&self) -> Result<(), FreeRtosError> {
        //从Handle获取TCB
        let tcb = self.GetTCB_read(self);
        let priority = self.get_priority();
        //和当前最高Priority的任务比较,如果该taskhandle更高，就更新
        traceMOVED_TASK_TO_READY_STATE!(&tcb);
        record_ready_priority(priority);
        //插入链表end
        list::list_insert_end(
            &READY_TASK_LISTS[priority as usize],
            Arc::clone(&tcb.StateListItem),
        );
        tracePOST_MOVED_TASK_TO_READY_STATE!(&tcb);
        Ok(())
    }

    fn add_new_task_to_ready_list(&self) -> Result<(), FreeRtosError> {
        let newtcb = self.GetTCB_read();
    
        //C语言宏接口
        taskENTER_CRITICAL!();
        {
            //通过全局变量获取并设定current_task数量
            let CurrentNumber = get_current_number_of_tasks!() + 1;
            set_current_number_of_tasks!(CurrentNumber);

            
            if task_global::CURRENT_TCB.read().unwrap().is_none() {
                //如果现在没有currenttskhandle
                //那就把newtcb这个tskhandle设定为currenttskhandle
                set_current_task_handle!(self.clone());
                if get_current_number_of_tasks!() != 1 {
                    mtCOVERAGE_TEST_MARKER!(); // What happened?
                }
            } else {
                //如果现在有currenttskhandle
                let tskhandle = get_current_task_handle!();
                if !get_scheduler_running!() {
                    //如果现在调度器没启动，并且newtcb优先就高于currenttsk，那就用这个newtcb作为启动task
                    if tskhandle.get_priority() <= newtcb.task_priority {
                        set_current_task_handle!(self.clone());
                    } else {
                        mtCOVERAGE_TEST_MARKER!();
                    }
                }
            }
            //获取task总数
            set_task_number!(get_task_number!() + 1);
            traceTASK_CREATE!(self.clone());
            self.add_task_to_ready_list()?;
        }
        //C语言宏接口
        taskEXIT_CRITICAL!();
        if get_scheduler_running!() {
            //如果调度器启动了，而currenttsk优先级低于newtcb，那就中断抢占
            let current_task_priority = get_current_task_handle!().get_priority();
            if current_task_priority < newtcb.task_priority {
                //C语言接口
                taskYIELD_IF_USING_PREEMPTION!();
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    
        Ok(())
    }
}

//INCLUDE_xTaskAbortDelay用于启用xTaskAbortDelay()函数。将其定义为1即可启用此功能。
//xTaskAbortDelay()强制任务离开阻塞状态并进入就绪状态。
//即使任务处于阻塞状态等待的事件未发生，且任何指定的超时时间未过期，也将离开阻塞
pub fn add_current_task_to_delayed_list(ticks_to_wait: TickType, can_block_indefinitely: bool) {
    let curtskhandle = get_current_task_handle!();
    trace!("Remove succeeded");

    {
        #![cfg(feature = "INCLUDE_xTaskAbortDelay")]
        curtskhandle.set_delay_aborted(false);
    }
    trace!("Abort succeeded");

    //为了将该任务加入Blocked List，首先要从Ready List中移除
    if list::list_remove(curtskhandle.get_state_list_item()) == 0 {
        trace!("Returned 0");
        //成功移除
        //重新设置Ready List的最高优先级
        portRESET_READY_PRIORITY!(curtskhandle.get_priority(), get_top_ready_priority!());
    } else {
        trace!("Returned not 0");
        mtCOVERAGE_TEST_MARKER!();
    }

    trace!("Remove succeeded");
    {
        //INCLUDE_vTaskSuspend被定义的情况：
        #![cfg(feature = "INCLUDE_vTaskSuspend")]
        //INCLUDE_vTaskSuspend光被定义还没用，还需要判断portMAX_DELA和can_block_indefinitely
        if ticks_to_wait == portMAX_DELAY && can_block_indefinitely {
            //将任务添加到Suspended List而不是Delayed List
            //确保它永久阻塞而不会被时钟唤醒
            let cur_state_list_item = curtskhandle.get_state_list_item();
            list::list_insert_end(&SUSPENDED_TASK_LIST, cur_state_list_item);
        } else {
            //否则，函数会计算任务应该在什么时候被唤醒（如果事件没有发生）
            //并根据唤醒时间将任务添加到不同的延迟任务列表中
            let time_to_wake = get_tick_count!() + ticks_to_wait;

            let cur_state_list_item = curtskhandle.get_state_list_item();
            list::set_list_item_value(&cur_state_list_item, time_to_wake);

            if time_to_wake < get_tick_count!() {
                //如果唤醒时间小于当前时间，则将任务添加到溢出延迟任务列表中
                list::list_insert(&OVERFLOW_DELAYED_TASK_LIST, cur_state_list_item);
            } else {
                //否则，将任务添加到当前延迟任务列表中
                list::list_insert(&DELAYED_TASK_LIST, curtskhandle.get_state_list_item());

                //如果任务被添加到了延迟任务列表的头部
                //那么xNextTaskUnblockTime需要被更新
                if time_to_wake < get_next_task_unblock_time!() {
                    set_next_task_unblock_time!(time_to_wake);
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            }
        }
    }

    {
        //INCLUDE_vTaskSuspend压根没被定义的情况：
        #![cfg(not(feature = "INCLUDE_vTaskSuspend"))]
        //计算任务应该在什么时候被唤醒（如果事件没有发生）
        //并根据唤醒时间将任务添加到适当的延迟任务列表中
        let time_to_wake = get_tick_count!() + ticks_to_wait;

        let cur_state_list_item = curtskhandle.get_state_list_item();
        list::set_list_item_value(&cur_state_list_item, time_to_wake);

        if time_to_wake < get_tick_count!() {
           //如果唤醒时间小于当前时间，则将任务添加到溢出延迟任务列表中
            list::list_insert(&OVERFLOW_DELAYED_TASK_LIST, cur_state_list_item);
        } else {
            //否则，将任务添加到当前延迟任务列表中
            list::list_insert(&DELAYED_TASK_LIST, curtskhandle.get_state_list_item());

            //如果任务被添加到了延迟任务列表的头部
            //那么xNextTaskUnblockTime需要被更新
            if time_to_wake < get_next_task_unblock_time!() {
                set_next_task_unblock_time!(time_to_wake);
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }
    }
    trace!("Place succeeded");
}

pub fn reset_next_task_unblock_time() {
    if list_is_empty(&DELAYED_TASK_LIST) {
        //检查DELAYED_TASK_LIST是否为空
        //如果为空，则将 xNextTaskUnblockTime 设置为最大可能值
        set_next_task_unblock_time!(portMAX_DELAY);
    } else {
        //如果DELAYED_TASK_LIST不为空
        //就把xNextTaskUnblockTime设置为该列表头结点的等待时间
        let mut temp = get_owner_of_head_entry(&DELAYED_TASK_LIST);
        set_next_task_unblock_time!(get_list_item_value(&temp.get_state_list_item()));
    }
}

#[macro_export]
macro_rules! get_handle_from_option {
    ($option: expr) => {
        match $option {
            Some(handle) => handle,
            None => get_current_task_handle!(),
        }
    };
}

#[cfg(feature = "INCLUDE_vTaskDelete")]
pub fn task_delete(task_to_delete: Option<TaskHandle>) {
    //如果NULL被传入，就删除calling task
    let pxtcb = get_handle_from_option!(task_to_delete);

    taskENTER_CRITICAL!();
    {
        //将任务从Ready List移除,前提是它在这个List
        if list::list_remove(pxtcb.get_state_list_item()) == 0 {
            //重新设置优先级
            taskRESET_READY_PRIORITY!(pxtcb.get_priority());
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }

        //接下来检查任务是否在等待事件
        //如果是，则从事件列表中移除该任务；否则，执行测试标记。
        if list::get_list_item_container(&pxtcb.get_event_list_item()).is_some() {
            list::list_remove(pxtcb.get_event_list_item());
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }

        //递增全局变量uxTaskNumber，以便内核感知调试器能够检测到任务列表需要重新生成。
        set_task_number!(get_task_number!() + 1);

        if pxtcb == get_current_task_handle!() {
            //检查要删除的任务是否是当前正在运行的任务
            //如果是，则将该任务插入到等待终止的任务列表中
            //以便空闲任务能够检查该列表并释放调度器为TCB和堆栈分配的内存。
            list::list_insert_end(&TASKS_WAITING_TERMINATION, pxtcb.get_state_list_item());

            //递增全局变量ucTasksDeleted
            //以便空闲任务知道有一个任务已被删除，因此应检查xTasksWaitingTermination列表
            set_deleted_tasks_waiting_clean_up!(get_deleted_tasks_waiting_clean_up!() + 1);

            //接下来调用宏portPRE_TASK_DELETE_HOOK!
            //该宏主要用于Windows模拟器，在其中执行特定于Windows的清理操作
            //在此之后，无法从此任务中产生YIELD，因此使用xYieldPending来锁定需要进行上下文切换
            portPRE_TASK_DELETE_HOOK!(pxtcb, get_yield_pending!());
        } else {
            //如果要删除的任务不是当前正在运行的任务，则递减全局变量uxCurrentNumberOfTasks
            set_current_number_of_tasks!(get_current_number_of_tasks!() - 1);

            //释放堆栈所占用的内存
            let stack_pos = pxtcb.GetTCB_read().stack_pos;
            port::port_free(stack_pos as *mut _);

            //调用函数reset_next_task_unblock_time()重置下一个预期解除阻塞时间
            reset_next_task_unblock_time();
        }
    }
    taskEXIT_CRITICAL!();

    //如果删除的任务是当前执行的任务，需要reschedule
    if get_scheduler_suspended!() > 0 {
        if pxtcb == get_current_task_handle!() {
            assert!(get_scheduler_suspended!() == 0);
            portYIELD_WITHIN_API!();
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    }
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn suspend_task(task_to_suspend: TaskHandle) {
    trace!("suspend_task called!");
    
    let mut tcb = task_to_suspend.GetTCB_read();
    taskENTER_CRITICAL!();
    {
        traceTASK_SUSPEND!(&tcb);

        //从原列表移除，并修改优先级
        if list_remove(tcb.get_state_list_item()) == 0 {
            taskRESET_READY_PRIORITY!(tcb.get_priority());
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }

        //从事件列表中移除
        if get_list_item_container(&tcb.get_event_list_item()).is_some() {
            list_remove(tcb.get_event_list_item());
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
        //加入SUSPEND列表
        list_insert_end(&SUSPENDED_TASK_LIST, tcb.get_state_list_item());
    }
    taskEXIT_CRITICAL!();

    if get_scheduler_running!() {
        //修改next_task_unblock_time
        taskENTER_CRITICAL!();
        {
            reset_next_task_unblock_time();
        }
        taskEXIT_CRITICAL!();
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }

    if task_to_suspend == get_current_task_handle!() {
        if get_scheduler_running!() {
            //如果Scheduler开启了，且该任务为正在执行的任务
            assert!(get_scheduler_suspended!() == 0);
            portYIELD_WITHIN_API!();
        } else {
            //如果Scheduler没有开启，而且pxCurrentTCB所指向的任务被suspend了
            //需要重新设定pxCurrentTCB
            if current_list_length(&SUSPENDED_TASK_LIST) != get_current_number_of_tasks!() {
                //如果不是所有任务都被suspend，那就进行上下文切换
                task_switch_context();
            }
        }
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn task_is_tasksuspended(task: &TaskHandle) -> bool {
    //检查给定任务是否处于暂停状态。
    let mut xreturn: bool = false;
    let tcb = task.GetTCB_read();

    //检查给定任务是否包含在暂停任务列表中
    if is_contained_within(&SUSPENDED_TASK_LIST, &tcb.get_state_list_item()) {
        //如果是，则检查该任务是否正在从中断服务例程（ISR）中恢复
        //即检查在不在PENDING_READY_LIST里
        if !is_contained_within(&PENDING_READY_LIST, &tcb.get_event_list_item()) {
            //如果没有，则检查该任务是否因为处于暂停状态或因为阻塞且没有超时而被包含在暂停列表中
            //即检查是不是事件驱动而待在暂停任务列表
            if get_list_item_container(&tcb.get_event_list_item()).is_none() {
                xreturn = true;
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        } else {
            mtCOVERAGE_TEST_MARKER!();
        }
    } else {
        mtCOVERAGE_TEST_MARKER!();
    }

    xreturn
}

#[cfg(feature = "INCLUDE_vTaskSuspend")]
pub fn resume_task(task_to_resume: TaskHandle) {
    //函数目的是恢复一个任务
    trace!("resume task called!");
    let mut tcb = task_to_resume.GetTCB_read();

    if task_to_resume != get_current_task_handle!() {
        //检查要恢复的任务是否为当前正在执行的任务，如果不是的情况：
        taskENTER_CRITICAL!();
        {
            if task_is_tasksuspended(&task_to_resume) {
                //调用task_is_tasksuspended函数检查给定任务是否处于暂停状态
                traceTASK_RESUME!(&tcb);

                //如果是,从暂停列表中删除该任务并将其添加到准备列表中
                list_remove(tcb.get_state_list_item());
                task_to_resume.add_task_to_ready_list();

                let current_task_priority = get_current_task_handle!().get_priority();
                /* We may have just resumed a higher priority task. */
                if tcb.get_priority() >= current_task_priority {
                    //检查要恢复的任务的优先级是否大于或等于当前任务的优先级
                    //如果是，则使用宏taskYIELD_IF_USING_PREEMPTION!()触发上下文切换
                    taskYIELD_IF_USING_PREEMPTION!();
                } else {
                    mtCOVERAGE_TEST_MARKER!();
                }
            } else {
                mtCOVERAGE_TEST_MARKER!();
            }
        }
        taskEXIT_CRITICAL!();
    } else {
        //检查要恢复的任务是否为当前正在执行的任务，如果是，那根本不用resume
        mtCOVERAGE_TEST_MARKER!();
    }
}