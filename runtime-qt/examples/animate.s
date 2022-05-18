# Compiled using "xic" by Owen Arden, Catalin Dumitru, Wenzel Jakob, and Danfeng Zhang
# Command line: java -jar xic.jar -libpath ../../../../pa5/pa5/runtime/include/ -libpath ../include/ --target linux animate.xi

.file "animate.xi"
.intel_syntax noprefix
.text

.globl _I_AnimationTimer_timeout_po6QTimer
_I_AnimationTimer_timeout_po6QTimer:
	push rbx
	call _IballRect_o5QRect
	mov rbx, rax
	mov rax, qword ptr [rip+_I_g_dx_i]
	add qword ptr [rip+_I_g_x_i], rax
	mov rax, qword ptr [rip+_I_g_dy_i]
	add qword ptr [rip+_I_g_y_i], rax
	mov rdx, qword ptr [rip+_I_g_x_i]
	mov rax, qword ptr [rip+_I_g_R_i]
	lea rcx, qword ptr [rdx+rax]
	cmp rcx, qword ptr [rip+_I_g_DIM_i]
	jl .L6
.L4:
	neg qword ptr [rip+_I_g_dx_i]
.L5:
	mov rdx, qword ptr [rip+_I_g_R_i]
	mov rcx, qword ptr [rip+_I_g_DIM_i]
	mov rax, qword ptr [rip+_I_g_y_i]
	lea rdi, qword ptr [rax+rdx]
	cmp rdi, rcx
	jl .L9
.L7:
	neg qword ptr [rip+_I_g_dy_i]
.L8:
	call _IballRect_o5QRect
	mov rsi, rbx
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+368]
	mov rbx, qword ptr [rip+_I_g_mainWidget_o7QWidget]
	mov r8, 5
	mov rcx, 5
	mov rdx, -5
	mov rsi, -5
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+16]
	mov rsi, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+224]
	jmp .E2
.L6:
	sub rdx, qword ptr [rip+_I_g_R_i]
	cmp rdx, 0
	jle .L4
	jmp .L5
.L9:
	sub rax, rdx
	cmp rax, 0
	jle .L7
	jmp .L8
.E2:
	pop rbx
	ret

.globl _I_BallWidget_paintEvent_po11QPaintEvent
_I_BallWidget_paintEvent_po11QPaintEvent:
	push rbx
	push r12
	push r13
	push r14
	push r15
	mov r12, rdi
	mov rbx, rsi
	mov r14, qword ptr [rip+_I_g_backBuffer_o7QPixmap]
	mov rdx, 192
	mov rsi, 255
	mov rdi, 255
	call _Iqcolor_o6QColoriii
	mov rsi, rax
	mov rdi, r14
	mov rax, qword ptr [r14]
	call qword ptr [rax+48]
	mov rdi, qword ptr [rip+_I_g_backBuffer_o7QPixmap]
	call _Iqpainter_o8QPaintero12QPaintDevice
	mov r14, rax
	mov r13, qword ptr [r14]
	mov rsi, 1
	mov rdi, r14
	call qword ptr [r13+32]
	mov rdx, 255
	xor rsi, rsi
	xor rdi, rdi
	call _Iqcolor_o6QColoriii
	mov rdi, rax
	call _Iqpen_o4QPeno6QColor
	mov r15, rax
	mov rsi, 5
	mov rdi, r15
	mov rax, qword ptr [r15]
	call qword ptr [rax+104]
	mov rsi, r15
	mov rdi, r14
	call qword ptr [r13+112]
	xor rdx, rdx
	xor rsi, rsi
	mov rdi, 255
	call _Iqcolor_o6QColoriii
	mov rdi, rax
	call _Iqbrush_o6QBrusho6QColor
	mov rsi, rax
	mov rdi, r14
	call qword ptr [r13+128]
	call _IballRect_o5QRect
	mov rsi, rax
	mov rdi, r14
	call qword ptr [r13+152]
	mov rdi, r14
	call qword ptr [r13+16]
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+48]
	mov rbx, rax
	mov rdi, r12
	call _Iqpainter_o8QPaintero12QPaintDevice
	mov r14, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+336]
	mov rdx, qword ptr [rip+_I_g_backBuffer_o7QPixmap]
	mov r15, qword ptr [r14]
	mov rcx, rbx
	mov rsi, rax
	mov rdi, r14
	call qword ptr [r15+216]
	mov rdi, r14
	call qword ptr [r15+16]
	pop r15
	pop r14
	pop r13
	pop r12
	pop rbx
	ret

.globl _I_init_AnimationTimer
_I_init_AnimationTimer:
	sub rsp, 8
	cmp qword ptr [rip+_I_size_AnimationTimer], 0
	jne .L10
	call _I_init_TimerListener
	mov rax, qword ptr [rip+_I_size_TimerListener]
	mov qword ptr [rip+_I_size_AnimationTimer], rax
	xor rax, rax
.L11:
	cmp rax, 2
	jge .L12
	lea rcx, qword ptr [rip+_I_vt_TimerListener]
	mov rdx, qword ptr [rcx+rax*8]
	lea rcx, qword ptr [rip+_I_vt_AnimationTimer]
	mov qword ptr [rcx+rax*8], rdx
	inc rax
	jmp .L11
.L12:
	lea rax, qword ptr [rip+_I_AnimationTimer_timeout_po6QTimer]
	mov qword ptr [rip+_I_vt_AnimationTimer+8], rax
.L10:
	add rsp, 8
	ret

.globl _I_init_BallWidget
_I_init_BallWidget:
	sub rsp, 8
	cmp qword ptr [rip+_I_size_BallWidget], 0
	jne .L1
	call _I_init_QWidget
	mov rax, qword ptr [rip+_I_size_QWidget]
	mov qword ptr [rip+_I_size_BallWidget], rax
	xor rcx, rcx
.L2:
	cmp rcx, 78
	jge .L3
	lea rax, qword ptr [rip+_I_vt_QWidget]
	mov rdx, qword ptr [rax+rcx*8]
	lea rax, qword ptr [rip+_I_vt_BallWidget]
	mov qword ptr [rax+rcx*8], rdx
	inc rcx
	jmp .L2
.L3:
	lea rax, qword ptr [rip+_I_BallWidget_paintEvent_po11QPaintEvent]
	mov qword ptr [rip+_I_vt_BallWidget+496], rax
.L1:
	add rsp, 8
	ret

.globl _IballRect_o5QRect
_IballRect_o5QRect:
	sub rsp, 8
	mov rax, qword ptr [rip+_I_g_R_i]
	mov rdi, qword ptr [rip+_I_g_x_i]
	sub rdi, rax
	mov rsi, qword ptr [rip+_I_g_y_i]
	sub rsi, rax
	mov rax, qword ptr [rip+_I_g_R_i]
	lea rdx, qword ptr [rax*2]
	mov rax, qword ptr [rip+_I_g_R_i]
	lea rcx, qword ptr [rax*2]
	call _Iqrect_o5QRectiiii
	mov rax, qword ptr [rip+_I_g_x_i]
	mov rcx, qword ptr [rip+_I_g_R_i]
	sub rax, rcx
	mov rdi, qword ptr [rip+_I_g_y_i]
	sub rdi, rcx
	mov rcx, qword ptr [rip+_I_g_R_i]
	lea rsi, qword ptr [rcx*2]
	mov rcx, rsi
	mov rdx, rsi
	mov rsi, rdi
	mov rdi, rax
	call _Iqrect_o5QRectiiii
	add rsp, 8
	ret

.globl _Imain_paai
_Imain_paai:
	push rbx
	push r14
	push r15
	call _Iqapplication_t2o12QApplicationaaiaai
	mov rbx, rax
	mov rdi, qword ptr [rip+_I_size_BallWidget]
	call _xi_alloc
	lea rcx, qword ptr [rip+_I_vt_BallWidget]
	mov qword ptr [rax], rcx
	mov qword ptr [rip+_I_g_mainWidget_o7QWidget], rax
	mov rax, qword ptr [rip+_I_g_DIM_i]
	mov rsi, qword ptr [rip+_I_g_DIM_i]
	mov rdi, rax
	call _Iqpixmap_o7QPixmapii
	mov qword ptr [rip+_I_g_backBuffer_o7QPixmap], rax
	call _Iqtimer_o6QTimer
	mov r14, rax
	mov rdi, qword ptr [rip+_I_size_AnimationTimer]
	call _xi_alloc
	lea rcx, qword ptr [rip+_I_vt_AnimationTimer]
	mov qword ptr [rax], rcx
	mov r15, qword ptr [r14]
	mov rsi, rax
	mov rdi, r14
	call qword ptr [r15+72]
	xor rsi, rsi
	mov rdi, r14
	call qword ptr [r15+40]
	mov rsi, 40
	mov rdi, r14
	call qword ptr [r15+32]
	mov rdi, r14
	call qword ptr [r15+48]
	mov r14, qword ptr [rip+_I_g_mainWidget_o7QWidget]
	mov rdi, qword ptr [rip+_I_g_DIM_i]
	mov rsi, rdi
	call _Iqsize_o5QSizeii
	mov rsi, rax
	mov rdi, r14
	mov rax, qword ptr [r14]
	call qword ptr [rax+264]
	mov rax, qword ptr [rip+_I_g_mainWidget_o7QWidget]
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+32]
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+8]
	pop r15
	pop r14
	pop rbx
	ret

	.bss
	.align 8
.globl _I_size_BallWidget
_I_size_BallWidget:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_vt_BallWidget
_I_vt_BallWidget:
	.zero 632
	.text

.section .ctors
	.align 8
	.quad _I_init_BallWidget
	.text

	.bss
	.align 8
.globl _I_size_AnimationTimer
_I_size_AnimationTimer:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_vt_AnimationTimer
_I_vt_AnimationTimer:
	.zero 24
	.text

.section .ctors
	.align 8
	.quad _I_init_AnimationTimer
	.text

	.section .data
	.align 8
.globl _I_g_DIM_i
_I_g_DIM_i:
	.quad 256
	.text

	.section .data
	.align 8
.globl _I_g_R_i
_I_g_R_i:
	.quad 16
	.text

	.section .data
	.align 8
.globl _I_g_x_i
_I_g_x_i:
	.quad 100
	.text

	.section .data
	.align 8
.globl _I_g_y_i
_I_g_y_i:
	.quad 50
	.text

	.section .data
	.align 8
.globl _I_g_dx_i
_I_g_dx_i:
	.quad 2
	.text

	.section .data
	.align 8
.globl _I_g_dy_i
_I_g_dy_i:
	.quad 1
	.text

	.bss
	.align 8
.globl _I_g_mainWidget_o7QWidget
_I_g_mainWidget_o7QWidget:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_g_backBuffer_o7QPixmap
_I_g_backBuffer_o7QPixmap:
	.zero 8
	.text

