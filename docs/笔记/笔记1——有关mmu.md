# mmu

__笔记1——有关mmu__

#### mmu的两大功能：

1. 对内存地址的访问进行控制：代码段只读权限控制，多线程的栈内存之间的空洞页隔离可以防止栈溢出后改写其他线程的栈内存，不同进程之间的地址隔离等等。

2. 将进程的地址空间隔离（地址映射）：进程具有空间独立性，每一个进程对应一个进程空间，在进程眼里会以为自己拥有所有整个内存空间，实际上这个只是虚拟内存空间，经过mmu被映射到真正的物理内存空间。只有存在mmu时，才有进程这个概念，只有线程。比如没有mmu的单片机中（即使有操作系统，比如FreeRTOS）,就不存在进程这个说法，只有线程或着叫做task（任务）。因为没有mmu就没办法实现内存空间的隔离。以上可参考视频[CPU眼里的：进程、线程 | MMU | 空间独立性_bilibili](https://www.bilibili.com/video/BV12r4y1e7uM/?spm_id_from=333.1007.top_right_bar_window_history.content.click&vd_source=d0dc82d5ac6356787d64b7062e49109c)

   <img src="address mapping.png">

#### 概念辨析

1. 多线程：一个进程中有多个线程。
2. 线程：一个进程可包含多个线程。在没有mmu的操作系统中也称为task。
3. 进程：有mmu后才有的概念，一个进程有自己的进程空间（实际上是虚拟空间），被mmu映射到物理空间上。
4. MPU（Memory Protection Unit）：可以将MMU（Memory Management Unit）理解为MPU的升级版。MPU主要用于内存保护（protect/restrict memory to tasks），但是不具备地址映射能力（remapping the address space）

#### 一些说明

1. Cotex-M系列（STM32就属于Cotex-M系列）没有MMU，但是Cotex-A系列有。关于Cotex、Arm等的区分记录在另外的笔记上。
2. 有人称有MMU的处理器叫CPU，没有MMU的称为MCU。当然这个说法其实不太准确，说法不一。