### FreeRTOS命名惯例

RTOS 内核和演示应用程序源代码使用以下惯例:

- 变量
  - *uint32_t* 类型变量以 *ul* 为前缀，其中“u”表示“unsigned” ，“l”表示“long”。
  - *uint16_t* 类型变量以 *us* 为前缀，其中“u”表示“unsigned” ， “s”表示“short”。
  - *uint8_t* 类型变量以 *uc* 为前缀，其中“u”表示“unsigned” ， “c”表示“char ”。
  - 非 stdint 类型的变量以 *x* 为前缀。例如，BaseType_t 和 TickType_t，二者分别是可移植层定义的定义类型，主要架构的自然类型或最有效类型，以及用于保存 RTOS ticks 计数的类型。
  - 非 stdint 类型的无符号变量存在附加前缀 *u*。例如，UBaseType_t（无符号 BaseType_t）类型变量以 *ux* 为前缀。
  - *size_t* 类型变量也带有 *x* 前缀。
  - 枚举变量以 *e* 为前缀
  - 指针以附加 *p* 为前缀，例如，指向 uint16_t 的指针将以 *pus* 为前缀。
  - 根据 MISRA 指南，未限定标准 *char* 类型仅可包含 ASCII 字符，并以 *c* 为前缀。
  - 根据 MISRA 指南，char * 类型变量仅可包含指向 ASCII 字符串的指针，并以 *pc* 为前缀。
- 函数
  - 文件作用域静态（私有）函数以 *prv* 为前缀。
  - 根据变量定义的相关规定，API 函数以其返回类型为前缀，并为 *void* 添加前缀 *v*。
  - API 函数名称以定义 API 函数文件的名称开头。例如，在 tasks.c 中定义 vTaskDelete，并且具有 void 返回类型。
- 宏
  - 宏以定义宏的文件为前缀。前缀为小写。例如，在 FreeRTOSConfig.h 中定义 configUSE_PREEMPTION。
  - 除前缀外，所有宏均使用大写字母书写，并使用下划线来分隔单词。

 

------



### 数据类型

仅使用 stdint.h 类型和 RTOS 自带的 typedef，但以下情况除外：

- char

  根据 MISRA 指南，仅在未限定字符类型包含 ASCII 字符方可使用未限定字符类型。

- char *

  根据 MISRA 指南，仅在未限定字符指针指向 ASCII 字符串时方可使用未限定字符指针。使用需要 char * 参数的标准库函数时，无需抑制良性编译器警告，此举尤其考虑到将一些编译器默认为未限定 char 类型是签名的，而其他编译器默认未限定 char 类型是未签名的。


针对每个移植定义四种类型。即：

- TickType_t

  如果 configUSE_16_BIT_TICKS 设置为非零 (true) ，则将 TickType_t 定义为无符号的 16 位类型（uint16_t）。如果 configUSE_16_BIT_TICKS 设置为零 (false)，则将 TickType_t 定义为无符号的 32 位类型（uint32_t）。请参阅 API 文档的[自定义](https://www.freertos.org/zh-cn-cmn-s/a00110.html)章节查看完整信息。

  32 位架构应始终将 configUSE_16_BIT_TICKS 设置为 0。

- BaseType_t

  架构中最有效、最自然的类型。例如，在 32 位架构上，BaseType_t 会被定义为 32 位类型。在 16 位架构上，BaseType_t 会被定义为 16 位类型。如果将 BaseType_t 定义为 char，则须特别注意确保将有符号字符用于可能为负的函数返回值来指示错误。

- UBaseType_t

  无符号的 BaseType_t。

- StackType_t

  意指架构用于存储堆栈项目的类型。通常是 16 位架构上的 16 位类型和 32 位架构上的 32 位类型，但也有例外情况。供 FreeRTOS 内部使用。