# XFSTests 失败用例报告

测试日期: 2025-11-24  
测试版本: commit f638591e  
测试结果: **720/722 通过 (99.7%)**

---

## 失败用例列表

- **generic/078** - RENAME_WHITEOUT 文件类型错误
- **generic/650** - 并发fsstress parent查找失败

---

## generic/078: RENAME_WHITEOUT 文件类型错误

### 测试内容

**测试脚本**: `tests/generic/078`

```bash
#!/bin/bash
# Check renameat2 syscall with RENAME_WHITEOUT flag

. ./common/preamble
_begin_fstest auto quick metadata

. ./common/renameat2

_require_test
_require_renameat2 whiteout
_require_symlinks

rename_dir=$TEST_DIR/$$
mkdir $rename_dir
_rename_tests $rename_dir -w
rmdir $rename_dir

status=0
exit
```

**测试说明**:
- 测试 `renameat2` 系统调用的 `RENAME_WHITEOUT` 标志
- 对各种文件类型组合(regular, symlink, directory, tree)执行 RENAME_WHITEOUT
- 验证源位置的 whiteout 文件类型应为字符设备(character device)

### 报错信息

**期望输出** (whiteout应显示为`char`):
```
samedir  regu/none -> char/regu.
samedir  regu/regu -> char/regu.
samedir  symb/none -> char/symb.
samedir  dire/none -> char/dire.
crossdir regu/none -> char/regu.
crossdir symb/none -> char/symb.
```

**实际输出** (whiteout显示为`none`):
```
samedir  regu/none -> none/regu.
samedir  regu/regu -> none/regu.
samedir  symb/none -> none/symb.
samedir  dire/none -> none/dire.
crossdir regu/none -> none/regu.
crossdir symb/none -> none/symb.
```

**完整diff**:
```diff
--- tests/generic/078.out
+++ results/generic/078.out.bad
@@ -4,48 +4,52 @@
 samedir  none/symb -> No such file or directory
 samedir  none/dire -> No such file or directory
 samedir  none/tree -> No such file or directory
-samedir  regu/none -> char/regu.
-samedir  regu/regu -> char/regu.
-samedir  regu/symb -> char/regu.
+samedir  regu/none -> none/regu.
+samedir  regu/regu -> none/regu.
+samedir  regu/symb -> none/regu.
 samedir  regu/dire -> Is a directory
 samedir  regu/tree -> Is a directory
-samedir  symb/none -> char/symb.
-samedir  symb/regu -> char/symb.
-samedir  symb/symb -> char/symb.
+samedir  symb/none -> none/symb.
+samedir  symb/regu -> none/symb.
+samedir  symb/symb -> none/symb.
 samedir  symb/dire -> Is a directory
 samedir  symb/tree -> Is a directory
-samedir  dire/none -> char/dire.
+samedir  dire/none -> none/dire.
 samedir  dire/regu -> Not a directory
 samedir  dire/symb -> Not a directory
-samedir  dire/dire -> char/dire.
+samedir  dire/dire -> none/dire.
 samedir  dire/tree -> Directory not empty
-samedir  tree/none -> char/tree.
-samedir  tree/regu -> Not a directory
+samedir  tree/none -> none/tree.
+samedir  tree/regu -> ./common/renameat2: line 19: /tmp/testoverlay/merged/273358/src/bar: No such file or directory
+Not a directory
 samedir  tree/symb -> Not a directory
-samedir  tree/dire -> char/tree.
-samedir  tree/tree -> Directory not empty
+samedir  tree/dire -> none/tree.
+samedir  tree/tree -> ./common/renameat2: line 19: /tmp/testoverlay/merged/273358/src/bar: No such file or directory
+Directory not empty
```

**额外错误**:
```
./common/renameat2: line 19: /tmp/testoverlay/merged/273358/src/bar: No such file or directory
```
在 `tree` 类型测试中出现，尝试访问 whiteout 内部的文件失败。

---

## generic/650: 并发fsstress parent查找失败

### 测试内容

**测试脚本**: `tests/generic/650`

```bash
#!/bin/bash
# Run an all-writes fsstress run with multiple threads while exercising
# CPU hotplugging to shake out bugs in the write path.

. ./common/preamble
_begin_fstest auto rw stress soak

exercise_cpu_hotplug()
{
    while [ -e $sentinel_file ]; do
        local idx=$(( RANDOM % nr_hotplug_cpus ))
        local cpu="${hotplug_cpus[idx]}"
        local action=$(( RANDOM % 2 ))
        echo "$action" > "$sysfs_cpu_dir/cpu$cpu/online" 2>/dev/null
        sleep 0.5
    done
}

# 运行 fsstress 多线程压力测试
nr_cpus=$((LOAD_FACTOR * nr_hotplug_cpus))
fsstress_args=(-w -d $stress_dir -p $nr_cpus -n $nr_ops)

for ((i = 0; i < 10; i++)); do
    rm -rf "$stress_dir"
    mkdir -p "$stress_dir"
    _run_fsstress "${fsstress_args[@]}"
    _test_cycle_mount
done
```

**测试说明**:
- 运行 fsstress 工具进行高并发随机文件操作
- 多线程并发执行(每个可热插拔CPU一个线程)
- 同时进行CPU热插拔操作
- 10次迭代，每次约3秒，总共约2500次操作
- 测试文件系统在高压力下的稳定性

### 报错信息

**错误日志**:
```
QA output created by 650
Silence is golden.
12: fent-id = 126: can't find parent id: 125
12: failed to get path for entry: id=126,parent=125
12: fent-id = 126: can't find parent id: 125
12: failed to get path for entry: id=126,parent=125
12: fent-id = 126: can't find parent id: 125
12: failed to get path for entry: id=126,parent=125
13: fent-id = 288: can't find parent id: 253
13: failed to get path for entry: id=288,parent=253
13: fent-id = 311: can't find parent id: 253
13: failed to get path for entry: id=311,parent=253
13: fent-id = 288: can't find parent id: 253
13: failed to get path for entry: id=288,parent=253
13: fent-id = 311: can't find parent id: 253
13: failed to get path for entry: id=311,parent=253
11: fent-id = 87: can't find parent id: 80
11: failed to get path for entry: id=87,parent=80
11: fent-id = 87: can't find parent id: 80
11: failed to get path for entry: id=87,parent=80
```

**错误模式**:
- 多个线程(11, 12, 13)同时报错
- 子节点(fent-id)无法找到其父节点(parent id)
- 相同的错误重复出现
- 错误发生在并发文件操作过程中

**完整diff**:
```diff
--- tests/generic/650.out
+++ results/generic/650.out.bad
@@ -1,2 +1,20 @@
 QA output created by 650
 Silence is golden.
+12: fent-id = 126: can't find parent id: 125
+12: failed to get path for entry: id=126,parent=125
+12: fent-id = 126: can't find parent id: 125
+12: failed to get path for entry: id=126,parent=125
+12: fent-id = 126: can't find parent id: 125
+12: failed to get path for entry: id=126,parent=125
+13: fent-id = 288: can't find parent id: 253
+13: failed to get path for entry: id=288,parent=253
+13: fent-id = 311: can't find parent id: 253
+13: failed to get path for entry: id=311,parent=253
+13: fent-id = 288: can't find parent id: 253
+13: failed to get path for entry: id=288,parent=253
+13: fent-id = 311: can't find parent id: 253
+13: failed to get path for entry: id=311,parent=253
+11: fent-id = 87: can't find parent id: 80
+11: failed to get path for entry: id=87,parent=80
+11: fent-id = 87: can't find parent id: 80
+11: failed to get path for entry: id=87,parent=80
```

---

### 失败用例类型
1. **generic/078**: 功能性问题 - RENAME_WHITEOUT whiteout文件类型不正确
2. **generic/650**: 并发问题 - 高并发下父子节点关系不一致
