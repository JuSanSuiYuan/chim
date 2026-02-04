# Chim 3.1 编译器示例程序

## 1. 斐波那契数列 (fib.chim)

```Chim
# 文件: fib.chim
# 计算斐波那契数列

fn fib(n: int): int =
  if n < 2:
    n
  else:
    fib(n - 1) + fib(n - 2)

fn sum_to(n: int): int =
  var sum = 0
  for i in 1..n:
    sum = sum + i
  sum

# 顶层代码按顺序执行
let f = fib(10)              # 55
let s = sum_to(10)           # 55
println("Fib(10) = " + str(f))
println("Sum(10) = " + str(s))
```

### 编译和运行

```bash
chimc fib.chim -o fib
./fib
```

### 预期输出

```
Fib(10) = 55
Sum(10) = 55
```

---

## 2. 素数判断 (prime.chim)

```Chim
# 文件: prime.chim
# 判断素数

fn is_prime(n: int): bool =
  if n < 2:
    false
  elif n == 2:
    true
  else:
    var is_prime = true
    var i = 2
    while i * i <= n:
      if n % i == 0:
        is_prime = false
        break
      i = i + 1
    is_prime

# 测试多个数字
let test_numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]

for n in test_numbers:
  if is_prime(n):
    println(str(n) + " 是素数")
  else:
    println(str(n) + " 不是素数")
```

### 编译和运行

```bash
chimc prime.chim -o prime
./prime
```

### 预期输出

```
1 不是素数
2 是素数
3 是素数
4 不是素数
5 是素数
...
17 是素数
18 不是素数
19 是素数
20 不是素数
```

---

## 3. 列表操作 (list.chim)

```Chim
# 文件: list.chim
# 列表处理示例

fn map(f: fn(int) -> int, list: [int]): [int] =
  var result = []
  for item in list:
    result = result + [f(item)]
  result

fn filter(p: fn(int) -> bool, list: [int]): [int] =
  var result = []
  for item in list:
    if p(item):
      result = result + [item]
  result

fn reduce(f: fn(int, int) -> int, list: [int], initial: int): int =
  var acc = initial
  for item in list:
    acc = f(acc, item)
  acc

# 使用
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# 映射: 每个元素乘以 2
let doubled = map(fn(x): int = x * 2, numbers)
println("Doubled: " + str(doubled))

# 过滤: 保留偶数
let evens = filter(fn(x): bool = x % 2 == 0, numbers)
println("Evens: " + str(evens))

# 过滤: 保留奇数
let odds = filter(fn(x): bool = x % 2 == 1, numbers)
println("Odds: " + str(odds))

# 归约: 求和
let sum = reduce(fn(acc: int, x: int): int = acc + x, numbers, 0)
println("Sum: " + str(sum))

# 归约: 求积
let product = reduce(fn(acc: int, x: int): int = acc * x, numbers, 1)
println("Product: " + str(product))
```

### 编译和运行

```bash
chimc list.chim -o list
./list
```

### 预期输出

```
Doubled: [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
Evens: [2, 4, 6, 8, 10]
Odds: [1, 3, 5, 7, 9]
Sum: 55
Product: 3628800
```

---

## 4. 模式匹配 (match.chim)

```Chim
# 文件: match.chim
# 模式匹配示例

fn describe_number(n: int): string =
  match n:
    0 => "零"
    1 => "一"
    2 => "二"
    _ if n < 0 => "负数: " + str(n)
    _ if n < 10 => "个位数: " + str(n)
    _ if n < 100 => "两位数: " + str(n)
    _ => "大数字: " + str(n)

# 测试
for i in -2..12:
  println(str(i) + " -> " + describe_number(i))
```

### 编译和运行

```bash
chimc match.chim -o match
./match
```

### 预期输出

```
-2 -> 负数: -2
-1 -> 负数: -1
0 -> 零
1 -> 一
2 -> 二
3 -> 个位数: 3
...
9 -> 个位数: 9
10 -> 两位数: 10
11 -> 两位数: 11
```

---

## 5. 快速排序 (quicksort.chim)

```Chim
# 文件: quicksort.chim
# 快速排序算法

fn partition(arr: [int], low: int, high: int): [int] =
  var pivot = arr[high]
  var i = low - 1
  var j = low
  while j < high:
    if arr[j] < pivot:
      i = i + 1
      var temp = arr[i]
      arr[i] = arr[j]
      arr[j] = temp
    j = j + 1
  var temp2 = arr[i + 1]
  arr[i + 1] = arr[high]
  arr[high] = temp2
  arr

fn quicksort_helper(arr: [int], low: int, high: int): [int] =
  if low < high:
    var pi = partition(arr, low, high)
    var left = quicksort_helper(arr, low, pi - 1)
    var right = quicksort_helper(left, pi + 1, high)
    right
  else:
    arr

fn quicksort(arr: [int]): [int] =
  if length(arr) == 0:
    []
  else:
    quicksort_helper(arr, 0, length(arr) - 1)

# 测试
let unsorted = [64, 25, 12, 22, 11, 90, 45, 33]
let sorted = quicksort(unsorted)
println("Original: " + str(unsorted))
println("Sorted: " + str(sorted))
```

---

## 6. 阶乘和组合数 (combinatorics.chim)

```Chim
# 文件: combinatorics.chim
# 阶乘和组合数

fn factorial(n: int): int =
  if n <= 1:
    1
  else:
    n * factorial(n - 1)

fn combination(n: int, k: int): int =
  factorial(n) / (factorial(k) * factorial(n - k))

fn permutation(n: int, k: int): int =
  factorial(n) / factorial(n - k)

# 计算排列组合
let n = 10
let k = 3

println("n = " + str(n))
println("k = " + str(k))
println("n! = " + str(factorial(n)))
println("C(n,k) = " + str(combination(n, k)))
println("P(n,k) = " + str(permutation(n, k)))

# 打印杨辉三角
println("\nPascal's Triangle:")
for i in 0..n:
  var line = ""
  for j in 0..i:
    line = line + " " + str(combination(i, j))
  println(line)
```

---

## 运行所有示例

```bash
# 编译所有示例
for file in examples/*.chim; do
    name=$(basename "$file" .chim)
    chimc "$file" -o "$name"
done

# 运行
for exe in fib prime list match; do
    echo "=== $exe ==="
    ./"$exe"
    echo ""
done
```
