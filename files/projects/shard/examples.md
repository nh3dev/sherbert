<link href="/style/style.css" rel="stylesheet"/>
<include "header.html">

The following examples assume `libc` is available on the system.

# Hello World
`extern` is used to declare the `puts` libc function.  
`&[T]` is by default a fat pointer, the `raw` keyword is used to convert it to a thin one.
```
let puts = extern "puts" |str: raw &[u8]|: u32;
puts("Hello, World!\0");
```

# Average
```
let average = |nums: &[u32]|: u32 {
	let sum = loop let (sum, i) = (0, 0) {
		if i == nums.len { break sum };
		(sum + nums[i], i + 1)
	}.0;

	sum / nums.len
};

average(&[1, 2, 3, 4, 5]) // => 3
```

# Fibonacci
**Shard** doesn't have a way to define variadic functions, so we
have to cheat a *little* by defining `printf` with fixed arguments.  
`self` is used to so we can recursively call the function we're in.
```
let printf = extern "printf" |str: raw &[u8], i: u32|: u32;

let fibonacci = |n: u32|: u32 {
   if n <= 1 { return n };
   self(n - 1) + self(n - 2)
};

let terms = 10;

loop let i = 0 {
	if i == terms { break };
	printf("%d\n\0", fibonacci(i));
	i + 1
}
```


# Bubble Sort
```
let bubble_sort = |array: &mut [u32]| {
	loop let i = 0 {
		if i == array.len { break };
		loop let j = 0 {
			if j == array.len - i - 1 { break };
			if array[j] > array[j + 1] {
				let temp = array[j];
				array[j] = array[j + 1];
				array[j] = temp;
			}
			j + 1
		}
		i + 1
	}
};

let array = mut [2, 8, 9, 7, 4, 3, 6, 5, 1, 0];

bubble_sort(&mut array);

// print the array
loop let i = 0 {
	if i == array.len { break };
	printf("%d\n\0", array[i]);
	i + 1
}
```

# Newtypes
```
let Box = |t: type| raw &mut t;

let<T> box = |v: T|: Box(T) {
	let<T> malloc = extern "malloc" |size: usize|: raw &mut T;
	let ptr = malloc(core::tyinfo(T).size);
	*ptr = v;
	Box(T)(ptr)
};

impl<T> core::drop |self: Box(T)| {
	let<T> free = extern "free" |ptr: raw &mut T|;
	free(self);
};

let x: Box(i32) = box(5);
core::drop(x);
```

<include "footer.html">
