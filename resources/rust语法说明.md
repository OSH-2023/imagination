#### Rust规则宏

Rust里面没有实用的#define，但是有功能极为强大的宏，规则宏是里面最简单的一类，可以用来替代原先的`#define MAX 100`这样的宏。例如：

```rust
#[macro_export]
macro_rules! MAX {
    () => 100
}

fn main() {
    println!("{}", MAX!());
}
```

如果想要实现更复杂一点的宏结构也类似：

```rust
#[macro_export]
macro_rules! sqr {
    ($x:expr) => {$x * $x}
}

fn main() {
    println!("{}", sqr!(1 + 1));
}
```

`#[macro_export]`表示该宏可以该源代码(假设为test1.rs)的任何位置使用，

如果我想要在其他源代码(test2.rs)中使用，可以在test2.rs的开头添加如下代码，即可进行使用

```rust
#[marco_use]
extern crate::test1
```

关于`extern crate::test1`解释在**模块**部分详细解释。



#### Rust 条件编译

条件编译是指根据某些条件来决定部分代码是否被视为源代码的一部分,即如果满足条件，这部分就会被编译，否则会被编译器无视，相当于不存在。

接下来通过一个例子直观感受一下，下面这段代码展示了如何使用`cfg`属性来有条件地编译函数：

```rust
#[cfg(target_os = "windows")]
fn are_you_on_windows() {
    println!("You are running windows!");
}

#[cfg(not(target_os = "windows"))]
fn are_you_on_windows() {
    println!("You are not running windows!");
}

fn main() {
    are_you_on_windows();
}
```

在上面的例子中，如果目标操作系统是Windows，那么第一个函数会被编译，否则第二个函数会被编译。这与C语言中使用预处理器指令（例如`#ifdef`和`#ifndef`）进行条件编译类似。



以上条件编译就是通过Rust的一个特殊的属性：#[cfg]来实现的。它有两种形式，相当于if：

```rust
     #[cfg(foo)]

     #[cfg(bar = "baz")]
```

它们还有一些辅助形式：

```rust
     #[cfg(any(unix, windows))] //类似“或”

     #[cfg(all(unix, target_pointer_width = "32"))]	//类似“且”

     #[cfg(not(foo))] //类似“非”
```

可以随意嵌套：

```rust
     #[cfg(any(not(unix), all(target_os="macos", target_arch = "powerpc")))]
```

除了#cfg这种语法，还有!cfg的用法，总体类似，很容易理解：

```rust
if cfg!(target_arch = "x86") { } 
else if cfg!(target_arch = "x86_64") { } 
else if cfg!(target_arch = "mips") { } 
else { }
```



##### 配置选项

以上cfg后面括号内的部分叫做配置选项，简单来说就是一个键值对，例如target_arch = "x86_64" 是一个配置选项。

##### 设置配置选项

既然条件编译需要知道配置选项，那么怎么设置配置选项呢？

1. 一部分配置选项是默认的，例如target_os = "windows"和target_arch = "x86_64"，它是编译器根据你自己电脑的配置默认设置的，你可以直接利用cfg判断。

2.  另外一些配置选项是你可以自己定义的，需要在源代码之外，使用Cargo进行设置。具体方法是在Cargo.toml中的[features]进行设置，这样一来foo就相当于被设置成了1。

   ```
   [features]
   foo=[]
   ```

   当我们这样设置时，Cargo传递给rustc一个标记，你可以理解为Cargo.toml类似于Makefile:

   ```
   --cfg feature="${feature_name}"
   ```

   编译时，如果直接编译：cargo build会发现，报错。

   正确的编译方式如下：

   ```
   cargo build --features="foo"
   ```

   编译，这样你使用`#[cfg(foo)]` 就能成功进行条件编译了。

#### 属性

上述条件编译介绍的cfg属于一种属性。
Rust属性的基本格式如下：`#[feature(arg1, arg2 = "param")] `

包括了两种写法：

1. 单个**标识符**代表的**属性名**，如#[unix]

2. 单个**标识符**代表**属性名**，后面紧跟着一个=，然后再跟着一个字面量（Literal），组成一个键值对，

   如`#[link(name = “openssl”)]`。

##### 属性的作用范围

比如#![feature(box_syntax)]，这表示这个属性是应用于它所在的这个整个源文件。而如果没有!，

#[feature(box_syntax)]则表示这个属性仅应用于紧接着的那个模块。



##### 几个基本的标识符(feature)

1. **条件编译**：首先就是上述介绍过的cfg就是一个标识符，通过#[cfg]属性，可以进行条件编译。

2. **派生**：通过 #[derive] 属性，编译器能够提供某些 trait 的基本实现。

   下面是可以自动派生的 trait：

   1. 比较 trait: `Eq`, `PartialEq`, `Ord`, `PartialOrd`

   - Eq：完全等价关系
   - PartialEq： 部分等价关系
   - Ord：全序关系
   - PartialOrd：比较排序顺序的值

   2. `Clone`, 用来从 &T 创建副本 T。
   3. `Copy`，使类型具有 “复制语义”（copy semantics）而非 “移动语义”（move semantics）。
   4. `Hash`，从 &T 计算哈希值（hash）。
   5. `Default`, 创建数据类型的一个空实例。
   6. `Debug`，允许使用 {:?} 来进行输出。

3. **指定数据类型**：通过\#[repr]属性。

   例如：

   ```rust
   #[repr(u8)]
   pub enum task_state {
       running = 0,
       ready = 1,
       blocked = 2,
       suspended = 3,
       deleted = 4,
   }
   ```

   上述代码在没有`#[repr(u8)]`时里面每一个变量的类型是编译器自动判断并生成的，这就导致外界在使用时不能很好地确认数据类型，有的时候不得不使用强**制类型转**换来进行转换。

   `#[repr(u8)]`的含义是讲enum中的所有变量强制声明为u8类型。

#### 模块（Module）

这些先进的语言的组织单位可以层层包含，就像文件系统的目录结构一样。Rust 中的组织单位是模（Module）。

```
mod nation {
    mod government {
        fn govern() {}
    }
    mod congress {
        fn legislate() {}
    }
    mod court {
        fn judicial() {}
    }
}
```

这是一段描述法治国家的程序：国家（nation）包括政府（government）、议会（congress）和法院（court），分别有行政、立法和司法的功能。我们可以把它转换成树状结构：

```
nation
 ├── government
 │ └── govern
 ├── congress
 │ └── legislate
 └── court
   └── judicial
```

在文件系统中，目录结构往往以斜杠在路径字符串中表示对象的位置，Rust 中的路径分隔符是 **::** 。

路径分为绝对路径和相对路径。绝对路径从 crate 关键字开始描述。相对路径从 self 或 super 关键字或一个标识符开始描述。例如：

```
crate::nation::government::govern();
```

是描述 govern 函数的绝对路径，相对路径可以表示为：

```
nation::government::govern();
```

##### 访问权限

Rust 中有两种简单的访问权：公共（public）和私有（private）。

默认情况下，如果不加修饰符，模块中的成员访问权将是私有的。

如果想使用公共权限，需要使用 pub 关键字。

对于私有的模块，只有在与其平级的位置或下级的位置才能访问，不能从其外部访问。



##### use: 引用外部模块

例如：

这是test1.rs

```rust
pub mod fnnn
{
 pub fo()
}
```

我想要在test2.rs中调用fo()，需要在前面加上如下代码。

这一部分可以理解为c语言的#include "test1.h"。

```rust
 use crate::test1::fnnn
 use fnnn::fo
```

值得注意的是，`use::* `表示将该文件下所有模块都加入。

##### extern “C”

`extern “C” fn fo() -> ret;`表示导入C语言的函数fo(),

使用时使用unsafe关键字：`unsafe(fo())；`



使用举例: 

```rust
extern “C”  fn add(a: i32 , b: i32)->i32;
let c: i32 = unsafe {
 add(a,b);
}
```

 

##### use 和 extern

use: 将该库放入当前的范围

extern: 链接外部的库

现在大部分情况下，不再需要使用extern crate 



#### Self和self

##### Self

我也不是很好解释，它相当于指向变量本身，经常出现在`trait`或`impl`中。

例如对于clone，

```rust
	let s1 = String::from("hello");
    let s2 = s1.clone();
```

它的特性是这样声明的：

```
trait Clone {
    fn clone(&self) -> Self;
}
```

##### self

1. 当`self`用作函数的第一个参数时，它等价于`self: Self`。

2. `&self`参数等价于`self: &Self`。

3. `&mut self`参数等价于`self: &mut Self`。

   

#### Typedef

没什么好说的，一眼看懂。

```rust
type StackType_t = i32;
type UBaseType_t = u32;
```

#### NULL

##### Option 枚举类

Option 是 Rust 标准库中的枚举类，这个类用于填补 Rust 不支持 null 引用的空白。

Rust 在语言层面彻底不允许空值 null 的存在，但无奈null 可以高效地解决少量的问题，所以 Rust 引入了 Option 枚举类：

```
enum Option<T> {
    Some(T),
    None,
}
```

如果你想定义一个可以为空值的类，你可以这样：

```
let opt = Option::Some("Hello");
```

如果你想针对 opt 执行某些操作，你必须先判断它是否是 **Option::None**：

##### 实例1

```rust
fn main() {
  let opt = Option::Some("Hello");
  match opt {
    Option::Some(something) => {
      println!("{}", something);
    },
    Option::None => {
      println!("opt is nothing");
    }
  }
}
```

运行结果：

```
Hello
```

如果你的变量刚开始是空值，你体谅一下编译器，它怎么知道值不为空的时候变量是什么类型的呢？

所以初始值为空的 Option 必须明确类型：

##### 实例2

```rust
fn main() {
  let opt: Option<&str> = Option::None;
  match opt {
    Option::Some(something) => {
      println!("{}", something);
    },
    Option::None => {
      println!("opt is nothing");
    }
  }
}
```

运行结果：

```cmd
opt is nothing
```
