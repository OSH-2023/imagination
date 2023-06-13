### 单机版ray的性能测试及优化

##### 下面进行任务执行时间的测试及优化

选取矩阵乘法作为测试程序，在没有ray的情况下，测试程序为**程序1**：

```python
import numpy as np
import time

def matrix_multiplication(matrix_a, matrix_b):
    # 获取矩阵 A 和 B 的维度
    rows_a, cols_a = matrix_a.shape
    rows_b, cols_b = matrix_b.shape
    
    # 确保矩阵可以相乘
    assert cols_a == rows_b
    
    # 创建一个用于存储结果的矩阵
    result = np.zeros((rows_a, cols_b))
    
    # 计算矩阵乘积
    for i in range(rows_a):
        for j in range(cols_b):
            for k in range(cols_a):
                result[i, j] += matrix_a[i, k] * matrix_b[k, j]
    
    return result
    
def generate_random_square_matrix(n):
    # 生成 n x n 的随机方阵
    matrix = np.random.randint(0, 1000, size=(n, n))
    return matrix
    
n = 100
matrix1 = generate_random_square_matrix(n)
matrix2 = generate_random_square_matrix(n)
tic = time.time()
matrix_multiplication(matrix1, matrix1)
duration = (time.time() - tic)
print("duration = "+ str(duration))
```

运行结果：

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\4.png">

使用单机版ray，程序修改为并行计算的结构，**程序2**的代码如下：

```python
import ray
import numpy as np
import time

ray.init()

@ray.remote
def multiply_row_column(row, column):
    # 计算矩阵的一行和一列的乘积
    return np.dot(row, column)

def parallel_matrix_multiplication(matrix_a, matrix_b):
    # 获取矩阵的维度
    rows_a, cols_a = matrix_a.shape
    rows_b, cols_b = matrix_b.shape
    
    # 确保矩阵可以相乘
    assert cols_a == rows_b
    
    # 创建一个存储结果的矩阵
    result = np.zeros((rows_a, cols_b))
    
    # 并行计算矩阵乘积
    for i in range(rows_a):
        for j in range(cols_b):
            # 提取矩阵的一行和一列
            row = matrix_a[i]
            column = matrix_b[:, j]
            
            # 在 Ray 上并行计算乘积
            result[i, j] = ray.get(multiply_row_column.remote(row, column))
    
    return result


def generate_random_square_matrix(n):
    # 生成 n x n 的随机方阵
    matrix = np.random.randint(0, 1000, size=(n, n))
    return matrix

n = 100
matrix1 = generate_random_square_matrix(n)
matrix2 = generate_random_square_matrix(n)
tic = time.time()
parallel_matrix_multiplication(matrix1, matrix1)
duration = (time.time() - tic)
print("duration = "+ str(duration))
```

运行结果：

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\5.png">

结果令人意想不到，使用并行计算之后，程序的运行时间暴涨，通过查阅资料，得知创建任务也需要一定的开销，程序2会创建$n^2$个任务，当$n=100$时任务的数量就是10000个，但每个任务实际上只执行了100个乘法运算（加法忽略），这说明任务创建的开销甚至比任务执行的开销还要大，导致了程序2的执行时间要远高于程序1。将调整，将并行计算的任务修改为**程序3：**

```python
import ray
import numpy as np
import time
import string
#ray.init(object_store_memory = 1*1024*1024*1024, num_cpus = 4)
ray.init()

@ray.remote
def multiply_row(row, matrix_b):
    result = np.dot(row, matrix_b)
    return result

def parallel_matrix_multiply(matrix_a, matrix_b):
    # 将矩阵B转置，以便按行并行计算
    transposed_b = np.transpose(matrix_b)

    results = []
    for row_a in matrix_a:
        results.append(multiply_row.remote(row_a, transposed_b))

    # 获取并汇总所有并行计算的结果
    results = ray.get(results)

    # 将结果组合成最终的乘积矩阵
    result_matrix = np.vstack(results)

    return result_matrix

def generate_random_square_matrix(n):
    # 生成 n x n 的随机方阵
    matrix = np.random.randint(0, 1000, size=(n, n))
    return matrix

n = 100
matrix1 = generate_random_square_matrix(n)
matrix2 = generate_random_square_matrix(n)
tic = time.time()
parallel_matrix_multiply(matrix1, matrix1)
duration = (time.time() - tic)
print("duration = "+ str(duration))
```

运行结果如下

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\6.png">

相较于程序2，程序3只会创建100个任务，且每个任务会执行10000个乘法运算，任务创建开销远小于任务开销，效果也是非常显著，程序3的运行速度是程序2的十几倍，程序1的2倍以上。并行计算任务的性能得到了极大的优化（远远大于20%）。

##### 使用程序3进行任务运行时间和吞吐量的测试和分析

调整矩阵大小，任务执行时间和吞吐量如下：

| 矩阵大小    | 100   | 200   | 400   | 800   | 1000  | 1200  |
| ----------- | ----- | ----- | ----- | ----- | ----- | ----- |
| 执行时间(s) | 0.186 | 0.366 | 0.779 | 2.05  | 3.21  | 4.61  |
| 吞吐量      | 5.38  | 11.90 | 20.53 | 31.21 | 31.15 | 31.23 |

由数据可知，任务执行时间随着矩阵大小的增加而持续增加，吞吐量在矩阵较小时随矩阵大小的增加而增加，但当矩阵大于800时，吞吐量趋于恒定。分析其原因，可将程序运行时间分为矩阵运算时间和其他时间，矩阵较小时其他时间不可忽略，随着矩阵大小的增加，其他时间所占的比重越来越小，所以吞吐量增加，当矩阵大到一定程度时，其他时间可以忽略不计，吞吐量趋于恒定。

### 分布式ray性能测试

ray集群部署如下：

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\10.png">

##### 使用程序3进行任务运行时间和吞吐量的测试和分析

| 矩阵大小    | 100  | 200  | 400  | 500  | 600  | 700  |
| ----------- | ---- | ---- | ---- | ---- | ---- | ---- |
| 执行时间(s) | 1.63 | 2.52 | 3.87 | 5.07 | 7.00 | 9.66 |
| 吞吐量      | 0.61 | 1.58 | 2.16 | 4.93 | 5.14 | 5.07 |

结论与单机测试结论相同，矩阵大于500时，吞吐量趋于恒定。

注：由于单机和分布式采用了不同的虚拟机，所以他们之间执行时间没有可比性。

### 基于Docker的分布式ray性能测试

##### 使用如下命令启动head节点：

```shell
sudo docker run -it rayproject/ray
```

##### 在Docker的dash中创建头节点

```shell
ray start --head --port=6379 --include-dashboard=true --dashboard-host=0.0.0.0 --dashboard-port=8265
```

结果如下：

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\7.png">

##### 打开另一个Docker创建从节点

```shell
ray start --address='172.17.0.2:6379'
```

查看节点状态：

```shell
ray status
```

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\9.png">

显示有两个节点，说明基于Docker的Ray部署成功。

##### 单节点程序运行时间（$800  \times 800$）

<img src="C:\Users\DELL\Desktop\ray部署及测试\src\8.png">

##### 使用程序3进行任务运行时间和吞吐量的测试和分析（两个节点）

| 矩阵大小    | 100   | 200  | 400  | 600  | 800   | 1000  |
| ----------- | ----- | ---- | ---- | ---- | ----- | ----- |
| 执行时间(s) | 2.16  | 2.58 | 3.63 | 6.03 | 10.80 | 17.08 |
| 吞吐量      | 0.463 | 1.55 | 4.40 | 5.97 | 5.92  | 5.85  |

结论与单机测试结论相同，矩阵大于600时，吞吐量趋于恒定。