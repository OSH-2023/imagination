#### ray的单机版部署

##### 安装pip

```shell
cd     #切换到根目录
sudo apt install pip
```

#####  安装pytest-runner

```shell
pip install pytest-runner
```

##### 安装ray(可能会失败，可以多尝试几次)

```shell
sudo apt-get update
```

```shell
sudo pip3 install -U ray
```

##### 安装ray[default]

```shell
sudo pip3 install 'ray[default]'
```

至此ray的单机部署完成

#### ray的集群部署

##### 运行以下指令创建head节点

```shell
ray start --head --port=6379 --include-dashboard=true --dashboard-host=0.0.0.0 --dashboard-port=8265
```

出现信息中含有：

<img src = ".\src\3.png">

这说明要创建从节点需要运行

```shell
ray start --address='172.31.175.228:6379'
```

##### 查看Ray Dashboard界面

可以在web看到当前Ray Dashboard界面，其中VM6629-CS2023为刚创建的头节点，VM6549-qiu是刚建立的从节点

<img src=".\src\1.png" style="zoom:100%;" />

##### 测试ray集群是否可用

尝试python代码

```python
import ray

ray.init(address = "172.31.175.228:6379")

@ray.remote
def hello():
    return "Hello"

@ray.remote
def world():
    return "world!"

@ray.remote
def hello_world(a, b):
    return a + " " + b

a_id = hello.remote()
b_id = world.remote()
c_id = hello_world.remote(a_id, b_id)

hello = ray.get(c_id)

print(hello)
```

运行：

```shell
python hello_world.py
```

结果如下：

<img src=".\src\2.png">

说明ray的集群部署成功

### 基于Docker的Ray部署

##### 安装Docker

```shell
sudo apt install docker.io
```

##### 使用如下命令下载Docker镜像：

```shell
sudo docker pull rayproject/ray
```

##### 使用如下命令启动head节点：

```shell
sudo docker run -it rayproject/ray
```

##### 在Docker的dash中创建头节点

```shell
ray start --head --port=6379 --include-dashboard=true --dashboard-host=0.0.0.0 --dashboard-port=8265
```

结果如下：

<img src=".\src\7.png">

##### 打开另一个Docker创建从节点

```shell
ray start --address='172.17.0.2:6379'
```

查看节点状态：

```shell
ray status
```

<img src=".\src\9.png">

显示有两个节点，说明基于Docker的Ray部署成功。
