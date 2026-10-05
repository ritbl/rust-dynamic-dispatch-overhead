# Static dispatch versus vtable dispatch

To run bench:
```sh
cargo run --release
```

To see asm, run:

```sh
cargo asm
```

# M4 max on macos (aarch64)

asm looks:

```asm

=== static_dispatch ===
__RNvCslbpFqcgEfJF_30rust_dynamic_dispatch_overhead15static_dispatch:
        sub     sp, sp, #48
        stp     x20, x19, [sp, #16]
        stp     x29, x30, [sp, #32]
        add     x29, sp, #32
        str     x0, [sp, #8]
        add     x8, sp, #8
        ; InlineAsm Start
        ; InlineAsm End
        cbz     x1, LBB0_3
        mov     x19, x1
        ldr     x20, [sp, #8]
LBB0_2:
        mov     x0, x20
        mov     x1, x2
        ; direct call
        bl      __RNvXCslbpFqcgEfJF_30rust_dynamic_dispatch_overheadNtB2_9TransformNtB2_9Operation5apply 
        mov     x2, x0
        subs    x19, x19, #1
        b.ne    LBB0_2
LBB0_3:
        mov     x0, x2
        ldp     x29, x30, [sp, #32]
        ldp     x20, x19, [sp, #16]
        add     sp, sp, #48
        ret

=== vtable_dispatch ===
__RNvCslbpFqcgEfJF_30rust_dynamic_dispatch_overhead15vtable_dispatch:
        sub     sp, sp, #48
        stp     x20, x19, [sp, #16]
        stp     x29, x30, [sp, #32]
        add     x29, sp, #32
        stp     x0, x1, [sp]
        mov     x8, sp
        ; InlineAsm Start
        ; InlineAsm End
        cbz     x2, LBB1_3
        mov     x19, x2
LBB1_2:
        ldp     x0, x8, [sp]   ; load data pointer + vtable pointer
        ldr     x8, [x8, #24]  ; load apply() function pointer from vtable
        mov     x1, x3         ; prepares the second argument
        blr     x8             ; indirect call
        mov     x3, x0
        subs    x19, x19, #1
        b.ne    LBB1_2
LBB1_3:
        mov     x0, x3
        ldp     x29, x30, [sp, #32]
        ldp     x20, x19, [sp, #16]
        add     sp, sp, #48
        ret
```

# cheap VPS on x86

```asm
=== static_dispatch ===
_RNvCs1bt5ya80skn_30rust_dynamic_dispatch_overhead15static_dispatch:
        pushq   %r15
        pushq   %r14
        pushq   %rbx
        subq    $16, %rsp
        movq    %rdx, %rax
        movq    %rdi, 8(%rsp)
        leaq    8(%rsp), %rcx
        #APP
        #NO_APP
        testq   %rsi, %rsi
        je      .LBB0_3
        movq    %rsi, %rbx
        movq    8(%rsp), %r14
         # Load address of Transform::apply into r15
        movq    _RNvXCs1bt5ya80skn_30rust_dynamic_dispatch_overheadNtB2_9TransformNtB2_9Operation5apply@GOTPCREL(%rip), %r15
.LBB0_2:
        movq    %r14, %rdi
        movq    %rax, %rsi
        callq   *%r15 # Call Transform::apply through fixed function pointer
        decq    %rbx
        jne     .LBB0_2
.LBB0_3:
        addq    $16, %rsp
        popq    %rbx
        popq    %r14
        popq    %r15
        retq
.Lfunc_end0:

=== vtable_dispatch ===
_RNvCs1bt5ya80skn_30rust_dynamic_dispatch_overhead15vtable_dispatch:
        pushq   %rbx
        subq    $16, %rsp
        movq    %rcx, %rax
        movq    %rdi, (%rsp)
        movq    %rsi, 8(%rsp)
        movq    %rsp, %rcx
        #APP
        #NO_APP
        testq   %rdx, %rdx
        je      .LBB1_3
        movq    %rdx, %rbx
.LBB1_2:
        movq    (%rsp), %rdi   # Load trait object's data pointer into %rdi
        movq    8(%rsp), %rcx  # Load trait object's vtable pointer into %rcx
        movq    %rax, %rsi     # Move current value/result into %rsi
        callq   *24(%rcx)      # Indirect call through the vtable
        decq    %rbx
        jne     .LBB1_2
.LBB1_3:
        addq    $16, %rsp
        popq    %rbx
        retq
.Lfunc_end1:
```