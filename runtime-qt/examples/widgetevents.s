# Compiled using "xic" by Drew Dunne, Jacob Glueck, Aaron Wisner, and Alex Libman
# Command line: ./xic -libpath ../include/ --target linux widgetevents.xi

  .text
  .intel_syntax noprefix
  .align  4
_IMyButton_keyPressEvent_po9QKeyEvent:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp], r14
  mov QWORD PTR [rsp + 8], r15
  mov r14, rdi
  mov r13, rsi
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  call QWORD PTR [rax + 96]
  mov rdi, rax
  mov rax, QWORD PTR [rdi]
  call QWORD PTR [rax + 8]
  mov rdi, rax
  call _Iprintln_pai
  mov rax, QWORD PTR [r14]
  mov rdi, r14
  mov rsi, r13
  call QWORD PTR [rax + 616]
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp]
  mov r15, QWORD PTR [rsp + 8]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IMyButton_mouseMoveEvent_po11QMouseEvent:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp], r15
  mov rax, QWORD PTR [rsi]
  mov rdi, rsi
  call QWORD PTR [rax + 88]
  mov r14, rax
  mov rax, QWORD PTR [r14]
  mov rdi, r14
  call QWORD PTR [rax + 8]
  mov rdi, rax
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprint_pai
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 44
  call _Iprint_pai
  mov rax, QWORD PTR [r14]
  mov rdi, r14
  call QWORD PTR [rax + 16]
  mov rdi, rax
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprintln_pai
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IMyButton_clicked_p:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 32], rbp
  mov QWORD PTR [rsp], r12
  mov QWORD PTR [rsp + 8], r13
  mov QWORD PTR [rsp + 16], r14
  mov QWORD PTR [rsp + 24], r15
  .data
_CARR0: .quad 81,117,105,116,63
_CARR1: .quad 83,104,111,117,108,100,32,119,101,32,113,117,105,116,63
  .text
  call _INO__WIDGET_o7QWidget
  mov r14, rax
  mov rdi, 48
  call _xi_alloc
  mov QWORD PTR [rax], 5
  add rax, 8
  mov rdi, rax
  lea rsi, QWORD PTR [_CARR0]
  mov rcx, 5
  rep movsq 
  mov rdi, rax
  call _Iqs_o7QStringai
  mov r13, rax
  mov rdi, 128
  call _xi_alloc
  mov QWORD PTR [rax], 15
  add rax, 8
  mov rdi, rax
  lea rsi, QWORD PTR [_CARR1]
  mov rcx, 15
  rep movsq 
  mov rdi, rax
  call _Iqs_o7QStringai
  mov r12, rax
  call _IButtonYes_i
  mov rbp, rax
  call _IButtonNo_i
  add rbp, rax
  mov rdi, r14
  mov rsi, r13
  mov rdx, r12
  mov rcx, rbp
  call _IqmessageBoxQuestion_io7QWidgeto7QStringo7QStringi
  mov rbp, rax
  call _IButtonYes_i
  cmp rbp, rax
  jne _L1
  mov rdi, QWORD PTR [_I_g_qapp_o12QApplication]
  mov rax, QWORD PTR [rdi]
  call QWORD PTR [rax + 16]
_L1:
  mov rbp, QWORD PTR [rsp + 32]
  mov r12, QWORD PTR [rsp]
  mov r13, QWORD PTR [rsp + 8]
  mov r14, QWORD PTR [rsp + 16]
  mov r15, QWORD PTR [rsp + 24]
  # epilog
  add rsp, 40
  ret 
  .align  4
_I_init_MyButton:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp], r15
  lea r14, _I_size_MyButton
  mov rax, QWORD PTR [r14]
  cmp rax, 0
  jne _L3
  call _I_init_QPushButton
  lea rax, _I_size_QPushButton
  mov rax, QWORD PTR [rax]
  mov QWORD PTR [r14], rax
  lea rax, _I_vt_QPushButton
  lea rcx, _I_vt_MyButton
  mov rsi, 0
_L4:
  cmp rsi, 94
  jge _L5
  mov rdx, QWORD PTR [rax + rsi*8]
  mov QWORD PTR [rcx + rsi*8], rdx
  inc rsi
  jmp _L4
_L5:
  lea rax, _IMyButton_keyPressEvent_po9QKeyEvent
  mov QWORD PTR [rcx + 544], rax
  lea rax, _IMyButton_mouseMoveEvent_po11QMouseEvent
  mov QWORD PTR [rcx + 528], rax
  lea rax, _IMyButton_clicked_p
  mov QWORD PTR [rcx + 744], rax
_L3:
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp]
  # epilog
  add rsp, 24
  ret 
  .globl _Imain_paai
  .align  4
_Imain_paai:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp], r14
  mov QWORD PTR [rsp + 8], r15
  call _Iqapplication_t2o12QApplicationaaiaai
  mov QWORD PTR [_I_g_qapp_o12QApplication], rax
  lea rax, _I_size_MyButton
  mov rdi, QWORD PTR [rax]
  lea r14, _I_vt_MyButton
  call _xi_alloc
  mov r13, rax
  mov QWORD PTR [r13], r14
  mov r14, QWORD PTR [r13]
  call _IDialogOkButton_o12StandardIcon
  mov rdi, rax
  call _IqiconStandard_o5QIcono12StandardIcon
  mov rdi, r13
  mov rsi, rax
  call QWORD PTR [r14 + 640]
  mov r14, QWORD PTR [r13]
  call _IClickFocus_o11FocusPolicy
  mov rdi, r13
  mov rsi, rax
  call QWORD PTR [r14 + 408]
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  mov rsi, 1
  call QWORD PTR [rax + 480]
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  call QWORD PTR [rax + 32]
  mov rdi, QWORD PTR [_I_g_qapp_o12QApplication]
  mov rax, QWORD PTR [rdi]
  call QWORD PTR [rax + 8]
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp]
  mov r15, QWORD PTR [rsp + 8]
  # epilog
  add rsp, 24
  ret 
  .p2align 4,,15
  _array_concat_d2lkZ2V0ZXZlbnRzLnhp:
  push	r14
  push	r13
  mov	r13, rdi
  push	r12
  push	rbp
  mov	r12, rsi
  push	rbx
  mov	rbx, QWORD PTR [rdi-8]
  mov	rbp, QWORD PTR [rsi-8]
  lea	r14, [rbx+rbp]
  lea	rdi, [8+r14*8]
  call	_xi_alloc
  xor	edx, edx
  mov	QWORD PTR [rax], r14
  mov	rdi, rax
  add	rax, 8
  test	rbx, rbx
  jle	_L20
  .p2align 4,,10
  .p2align 3
_L21:
  mov	rcx, QWORD PTR [r13+0+rdx*8]
  mov	QWORD PTR [rdi+8+rdx*8], rcx
  add	rdx, 1
  cmp	rbx, rdx
  jne	_L21
_L20:
  xor	edx, edx
  test	rbp, rbp
  lea	rsi, [rdi+rbx*8]
  jle	_L22
  .p2align 4,,10
  .p2align 3
_L23:
  mov	rcx, QWORD PTR [r12+rdx*8]
  mov	QWORD PTR [rsi+8+rdx*8], rcx
  add	rdx, 1
  cmp	rbp, rdx
  jne	_L23
_L22:
  pop	rbx
  pop	rbp
  pop	r12
  pop	r13
  pop	r14
  ret
  .p2align 4,,15
  .local	_FUN_1_func1
  .type	_FUN_1_func1, @function
  _FUN_1_func1:
  push	r15
  push	r14
  push	r13
  push	r12
  push	rbp
  push	rbx
  sub	rsp, 24
  mov	r13, QWORD PTR [rdi]
  cmp	rdx, 1
  mov	QWORD PTR [rcx], r13
  jle	_L24
  test	r13, r13
  jle	_L24
  lea	rax, [rdx-1]
  mov	r14, QWORD PTR [rsi]
  lea	r15, [rsi+8]
  lea	rbp, [rcx+16+r13*8]
  mov	r12, rcx
  xor	ebx, ebx
  mov	QWORD PTR [rsp], rax
  lea	rax, [rdi+8]
  sal	r14, 3
  mov	QWORD PTR [rsp+8], rax
  .p2align 4,,10
  .p2align 3
_L25:
  mov	rdx, QWORD PTR [rsp]
  mov	rdi, QWORD PTR [rsp+8]
  lea	rcx, [rbp-8]
  mov	QWORD PTR [r12+8+rbx*8], rbp
  mov	rsi, r15
  add	rbx, 1
  add	rbp, r14
  call	_FUN_1_func1
  cmp	r13, rbx
  jg	_L25
_L24:
  add	rsp, 24
  pop	rbx
  pop	rbp
  pop	r12
  pop	r13
  pop	r14
  pop	r15
  ret
  
  .p2align 4,,15
  .local	_array_alloc_d2lkZ2V0ZXZlbnRzLnhp
  .type	_array_alloc_d2lkZ2V0ZXZlbnRzLnhp, @function
  _array_alloc_d2lkZ2V0ZXZlbnRzLnhp:
  push	rbp
  lea	rdx, [0+rsi*8]
  mov	rbp, rsp
  lea	rax, [rdx+22]
  push	r13
  push	r12
  push	rbx
  mov	r12, rdi
  and	rax, -16
  mov	r13, rsi
  sub	rsp, 8
  sub	rsp, rax
  mov	rax, rsi
  sub	rax, 1
  mov	rbx, rsp
  js	_L26
  mov	rcx, QWORD PTR [rdi-8+rdx]
  xor	edi, edi
  test	rcx, rcx
  jns	_L27
  jmp	_L28
  .p2align 4,,10
  .p2align 3
_L29:
  mov	rcx, QWORD PTR [r12+rax*8]
  test	rcx, rcx
  js	_L28
_L27:
  imul	rdi, rcx
  lea	rdi, [rcx+1+rdi]
  mov	QWORD PTR [rbx+rax*8], rdi
  sub	rax, 1
  cmp	rax, -1
  jne	_L29
  sal	rdi, 3
_L30:
  call	_xi_alloc
  lea	rsi, [rbx+8]
  mov	rcx, rax
  mov	rdx, r13
  mov	rdi, r12
  mov	r8, rax
  call	_FUN_1_func1
  lea	rsp, [rbp-24]
  lea	rax, [r8+8]
  pop	rbx
  pop	r12
  pop	r13
  pop	rbp
  ret
  .p2align 4,,10
  .p2align 3
_L28:
  xor	eax, eax
  call	_xi_out_of_bounds
  lea	rsp, [rbp-24]
  xor	eax, eax
  pop	rbx
  pop	r12
  pop	r13
  pop	rbp
  ret
_L26:
  xor	edi, edi
  jmp	_L30
  .align  4
_I_global_init:
  # prologue
  # prologue: begin function body
  # epilog
  ret 
  .section .ctors
  .align 8
  .quad _I_init_MyButton
  .quad _I_global_init
  .section .rodata
  .bss
  .align 8
  .globl _I_size_MyButton
_I_size_MyButton:
  .zero 8
  .align 8
  .globl _I_vt_MyButton
_I_vt_MyButton:
  .zero 760
  .align 8
  .globl _I_g_qapp_o12QApplication
_I_g_qapp_o12QApplication:
  .zero 8
  .text
