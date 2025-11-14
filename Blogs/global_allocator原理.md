alloc.rs文件中定义了如下外部链接函数：
```rust
unsafe extern "Rust" {
    // These are the magic symbols to call the global allocator.
    #[rustc_allocator]
    #[rustc_nounwind]
    #[rustc_std_internal_symbol]
    fn __rust_alloc(size: usize, align: usize) -> *mut u8;
    #[rustc_deallocator]
    #[rustc_nounwind]
    #[rustc_std_internal_symbol]
    fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
    #[rustc_reallocator]
    #[rustc_nounwind]
    #[rustc_std_internal_symbol]
    fn __rust_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8;
	// ...
}
```
其中`trait GlobalAlloc`包含以下函数声明：
```rust
pub unsafe trait GlobalAlloc {
    #[stable(feature = "global_alloc", since = "1.28.0")]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
	#[stable(feature = "global_alloc", since = "1.28.0")]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
	#[stable(feature = "global_alloc", since = "1.28.0")]
	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8
	// ...
}
```
当代码中使用`#[global_allocator]`修饰了一个static变量(必须是实现了`trait GlobalAlloc`的类型)，Rust编译器会执行一个内置宏展开，自动生成调用以上函数的包装函数`__rust_alloc`、`__rust_dealloc`、`__rust_realloc`等，伪代码例如：
```rust
// 伪代码:

// 外部符号 __rust_alloc 的实现
#[no_mangle] // 确保符号名就是 "__rust_alloc"，不被混淆
pub unsafe extern "C" fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
    // 内部逻辑就是调用你那个 static 变量的 alloc 方法
    MY_ALLOC.alloc(Layout::from_size_align_unchecked(size, align))
}

// 外部符号 __rust_dealloc 的实现
#[no_mangle]
pub unsafe extern "C" fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize) {
    // 内部逻辑就是调用你那个 static 变量的 dealloc 方法
    MY_ALLOC.dealloc(ptr, Layout::from_size_align_unchecked(size, align))
}
// ...
```
这些函数会作为强符号链接到alloc.rs中定义的外部函数

Vec\<T>、String、LinkedList\<T>等集合(或称为容器)类型都使用`Global`作为默认的堆内存分配器，例如：
```rust
pub struct Vec<T, #[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global> {
    buf: RawVec<T, A>,
    len: usize,
}
```
`Global`虽然是一个实现`trait Allocator`的变量，但其底层会调用：
```rust
#[stable(feature = "global_alloc", since = "1.28.0")]
#[must_use = "losing the pointer will leak memory"]
#[inline]
#[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    unsafe {
        // Make sure we don't accidentally allow omitting the allocator shim in
        // stable code until it is actually stabilized.
        core::ptr::read_volatile(&__rust_no_alloc_shim_is_unstable);

        __rust_alloc(layout.size(), layout.align())
    }
}

#[stable(feature = "global_alloc", since = "1.28.0")]
#[inline]
#[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe { __rust_dealloc(ptr, layout.size(), layout.align()) }
}

// ...
```
如前所述，`__rust_alloc`、`__rust_dealloc`这些函数底层都是调用的`GlobalAlloc::alloc`、`GlobalAlloc::dealloc`等函数