## QEMU

##### 什么是qemu

按官方解释，qemu是一个处理器仿真器（processor emulator），我们可以简单的理解为一种虚拟机。当提到虚拟机时很多人都会想到VMware,这个软件可以让我们在一个系统中运行另一个操作系统，并且能同时运行多个，比如在windows下运行一个linux系统，从而帮助我们完成一些只有在linux中可以完成的工作。既然VMware这么好用，为什么还要提到QEMU呢？那是因为VMware只能虚拟一个与主机有着同样指令集架构的系统，比如上面提到的windows和linux均是x86架构的，然而，我们本次项目希望能在自己的电脑上运行一个arm或riscv架构的系统，那么这个时候就不能使用VMware了。而QEMU则可以在x86、x86-64和PowerPC的系统中运行，并且模拟的机器可以是x86、 x86-64、ARM、 SPARC、 PowerPC和MIPS。比如说，通过QEMU我们可以在自己电脑上模拟一个树莓派或者一款STM32单片机，从而可以做到在没有硬件的情况下测试我们自己改进优化的操作系统的运行是否符合我们的预期。这种方式不仅可以缩小成本，也大大增加了验证工作的效率。

##### QEMU的两种运行模式

QEMU为用户提供了两种不同的运行模式。其一是User-mode，在这种模式下，我们可以在我们自己的机器上面运行另一个架构的编译得到的二进制文件，实际上，此时QEMU为我们做的事情是把模拟的架构指令转化为我们自己机器上架构的指令，然后在真实的CPU上执行；第二种模式叫做Full-system，这种模式下，QEMU能够完全模拟一个完整的机器出来，从BIOS到CPU、内存再到各种外设（硬盘、声卡……），均通过纯软件模拟出来。因此这个模拟出来的虚拟机器与真实的机器行为几乎完全一致，运行在上面的操作系统是无法得知自己是运行在虚拟机器上的。这对我们来说无疑是个好消息，因为我们正希望模拟出来的机器能尽量贴近物理机。

##### QEMU的加速

###### KQEMU

KQEMU即QEMU加速器，是一个已经过时的QEMU加速驱动程序，官方声称有了它的加持，模拟机器的速度可以接近原生态机器（真实机器）。KQEMU可以运行在x86架构的宿主机上，但是当今最新版本的QEMU（0.11及更高版本）已经不再被KQEMU支持，新版本的QEMU多数采用KVM来进行加速。

###### KVM

KVM（Kernel-based Virtual Machine）是Linux内核的一个模块，即当编译Linux内核时如果没有裁剪掉虚拟化模块，就能在Linux上使用这个模块的功能。KVM和QEMU最大的区别就是KVM能够利用CPU和内存硬件来辅助虚拟化，相比于纯软件模拟的QEMU，它的效率就相当地高，KVM一般和QEMU配合使用，KVM实现CPU与内存虚拟化，而QEMU实现IO的虚拟化。

##### 怎么使用QEMU运行一个操作系统

以FreeDOS这个操作系统为例，它属于i386架构，虽然i386理论上也属于x86的一种，但是由于比较老，所以在QEMU里和x86架构是两个类别。

1. 选择架构：首先使用QEMU开始模拟一台机器需要指定它的__架构__，在这里需要用到的指令参数是

   ``` bash
   $ qemu-system-i386 #这条指令最终会和下面提到的一些参数配合使用
   ```

2. 创建硬盘：接着我们需要给机器一个硬盘，我们用下面指令创造一个虚拟的__硬盘__（virtual disk）,创造出来的实际是一个image文件，我们命名为image.img，并指定其大小为200M,这些都可以自己决定怎么调节

   ``` bash
   $ qemu-img create image.img 200M
   ```

3. 指定硬盘：然后我们就在开启虚拟机的指令里添加参数指定__硬盘__文件imagefile

   ```bash
   -hda imagefile #imagefile指的是虚拟硬盘的image文件，比如在这个例子中应该换成image.img
   ```

4. 在自己电脑上装过windows或linux操作系统的同学应该都知道，装系统时需要一个已经写入目标操作系统的光盘或者U盘之类的存储器用于安装操作系统的电脑硬盘上。在这里，我们也用一个参数来指定这个__光盘__。同样，这里的光盘也是一个虚拟的镜像文件，这种文件称为ISO镜像文件，可以从相应的网站上下载，比如说这个例子中就从FreeDOS官网中下载了这个CD-ROM(光盘的一种，除此之外还有DVD)iso，文件名为FD12CD.iso

   ```bash
   -cdrom isofile #在这个例子中isofile替换为FD12CD.iso
   ```

5. 为机器分配一个内存

   ```bash
   -m 16M #分配了16m内存，如果没有这个参数默认是128m
   ```

6. 配置boot顺序，格式为-boot [*options*]

   ```bash
   -boot order=dc #这里告诉QEMU先尝试从CDROM（d）启动，再尝试从硬盘hard drive（c）启动。
   ```

7. 最后把所有参数合为长指令，最终就剩下两条指令

   ```bash
   $ qemu-img create image.img 200M
   $ qemu-system-i386 -hda image.img -cdrom FD12CD.iso -m 16M -boot order=dc
   ```

   这个时候就成功启动了QEMU，跳出一个与给真实机器安装系统时一样的界面，接着就能像操作真正的电脑一样安装操作系统并使用这台机器。

QEMU的机器配置完全靠参数来指定，这导致上手的时候比较困难，但是这种方式能够十分灵活地配置机器。

##### QEMU的ARM架构机器模拟

QEMU可以模拟ARM架构的机器是其一大亮点，也是本项目我们很看重QEMU的原因。而ARM架构的机器比较特殊，值得我们单独讨论。

QEMU能够模拟32位和64位的ARM CPU，对应的指令为

``` bash
$ qemu-system-aarch64 #用于模拟32位或64位的arm机器
$ qemu-system-arm #用于模拟32位的机器
```

与其他架构不同，ARM架构的CPU大多数是集成在片上系统（system-on-chip或SoC）上的，这些SoC都由许多不同的公司厂家开发，搭载的外设都十分的不同，即使是同一款SoC，不同厂家在生产整个机器（板子）时也大相径庭。这使得QEMU支持50多种ARM机器，却还只能覆盖现有ARM架构机器中很小的一部分。一块ARM板子上跑的操作系统在另一款ARM机器上就大概率完全跑不起来，从而用户程序和内核程序很难完全分开（比如STM32上如果要用FreeRTOS就得把操作系统和用户程序放在一起编译），而不像x86架构那样每台机器上跑的操作系统都像是标准的一套，一旦操作系统内核启动，用户程序就很少去关心操作系统程序。

QEMU支持ARM架构中有三种，分为A类、M类和R类（也称Cotex-A、Cotex-M、Cotex-R）。A类CPU是较高性能的CPU（拥有MMU，可以运行Linux这种需要MMU的系统）；M类包括Cortex-M0、Cortex-M3和Cortex-M4，不含有MMU，这类的CPU属于嵌入式板上的的微控制器。但是无论是哪一类，大多数的ARM架构的板子的硬件是固定的，因此不需要我们自己去设定它的RAM大小之类的东西，只需要直接指定特定的机器就行了。

当我们要模拟某一款ARM板子（也就是所谓的机器），我们需要先用以下命令查一下QEMU是否支持它

```bash
$ qemu-system-aarch64 --machine help 
```

其中包含一些我们比较熟悉的机器，比如

+ Raspberry Pi boards (`raspi0`, `raspi1ap`, `raspi2b`, `raspi3ap`, `raspi3b`）
+ Orange Pi PC (`orangepi-pc`)
+ STMicroelectronics STM32 boards (`netduino2`, `netduinoplus2`, `stm32vldiscovery`)

这里的树莓派和STM32正是我们展开讨论的两类机器。

##### 参考资料

[Documentation/KQemu - QEMU](https://wiki.qemu.org/Documentation/KQemu#Introduction)

QEMU: a Multihost, Multitarget Emulator——Daniel Bartholomew

kvm: the Linux Virtual Machine Monitor——Avi Kivity、Yaniv Kamay、Dor Laor、Uri Lublin、Anthony Liguori

[How to use QEMU to boot another OS——JIM HALL 2020](https://www.howtogeek.com/devops/how-to-use-qemu-to-boot-another-os/)

[Arm System emulator — QEMU documentation](https://www.qemu.org/docs/master/system/target-arm.html)