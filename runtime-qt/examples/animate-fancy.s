# Compiled using "xic" by Owen Arden, Catalin Dumitru, Wenzel Jakob, and Danfeng Zhang
# Command line: java -jar xic.jar -libpath ../include/ --target linux animate-fancy.xi

.file "animate-fancy.xi"
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
	mov rcx, qword ptr [rip+_I_g_R_i]
	mov rdx, qword ptr [rip+_I_g_DIM_i]
	mov rax, qword ptr [rip+_I_g_y_i]
	lea rdi, qword ptr [rax+rcx]
	cmp rdi, rdx
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
	jmp .E3
.L6:
	sub rdx, qword ptr [rip+_I_g_R_i]
	cmp rdx, 0
	jle .L4
	jmp .L5
.L9:
	sub rax, rcx
	cmp rax, 0
	jle .L7
	jmp .L8
.E3:
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

.globl _I_PlaybackController_triggered_po7QAction
_I_PlaybackController_triggered_po7QAction:
	push rbx
	mov rdi, rsi
	mov rax, qword ptr [rsi]
	call qword ptr [rax+88]
	mov rbx, rax
	mov rax, qword ptr [rip+_I_g_play_o7QAction]
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+88]
	mov rsi, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+272]
	mov rcx, 1
	xor rcx, rax
	test rcx, rcx
	jnz .L14
	mov rax, qword ptr [rip+_I_g_play_o7QAction]
	xor rsi, rsi
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	mov rax, qword ptr [rip+_I_g_stop_o7QAction]
	mov rsi, 1
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	mov rax, qword ptr [rip+_I_g_timer_o6QTimer]
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
.L15:
	jmp .E1
.L14:
	mov rax, qword ptr [rip+_I_g_play_o7QAction]
	mov rsi, 1
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	mov rax, qword ptr [rip+_I_g_stop_o7QAction]
	xor rsi, rsi
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	mov rax, qword ptr [rip+_I_g_timer_o6QTimer]
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+56]
	jmp .L15
.E1:
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
	xor rcx, rcx
.L11:
	cmp rcx, 2
	jge .L12
	lea rax, qword ptr [rip+_I_vt_TimerListener]
	mov rdx, qword ptr [rax+rcx*8]
	lea rax, qword ptr [rip+_I_vt_AnimationTimer]
	mov qword ptr [rax+rcx*8], rdx
	inc rcx
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
	mov rax, qword ptr [rax+rcx*8]
	lea rdx, qword ptr [rip+_I_vt_BallWidget]
	mov qword ptr [rdx+rcx*8], rax
	inc rcx
	jmp .L2
.L3:
	lea rax, qword ptr [rip+_I_BallWidget_paintEvent_po11QPaintEvent]
	mov qword ptr [rip+_I_vt_BallWidget+496], rax
.L1:
	add rsp, 8
	ret

.globl _I_init_PlaybackController
_I_init_PlaybackController:
	sub rsp, 8
	cmp qword ptr [rip+_I_size_PlaybackController], 0
	jne .L16
	call _I_init_ActionListener
	mov rax, qword ptr [rip+_I_size_ActionListener]
	mov qword ptr [rip+_I_size_PlaybackController], rax
	xor rcx, rcx
.L17:
	cmp rcx, 2
	jge .L18
	lea rax, qword ptr [rip+_I_vt_ActionListener]
	mov rdx, qword ptr [rax+rcx*8]
	lea rax, qword ptr [rip+_I_vt_PlaybackController]
	mov qword ptr [rax+rcx*8], rdx
	inc rcx
	jmp .L17
.L18:
	lea rax, qword ptr [rip+_I_PlaybackController_triggered_po7QAction]
	mov qword ptr [rip+_I_vt_PlaybackController+8], rax
.L16:
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
	push rbp
	push r12
	push r13
	push r14
	push r15
	sub rsp, 8
	call _Iqapplication_t2o12QApplicationaaiaai
	mov r15, rax
	call _Iqmainwindow_o11QMainWindow
	mov r12, rax
	call _Iqtoolbar_o8QToolBar
	mov rbx, rax
	call _IToolButtonTextBesideIcon_o15ToolButtonStyle
	mov rcx, qword ptr [rbx]
	mov rsi, rax
	mov rdi, rbx
	call qword ptr [rcx+648]
	mov r14, qword ptr [r12]
	mov rsi, rbx
	mov rdi, r12
	call qword ptr [r14+632]
	lea rdi, qword ptr [rip+.L19]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqmenu_o5QMenuo7QString
	mov r13, rax
	mov rdi, r12
	call qword ptr [r14+640]
	mov rsi, r13
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+632]
	lea rdi, qword ptr [rip+.L20]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqaction_o7QActiono7QString
	mov qword ptr [rip+_I_g_play_o7QAction], rax
	mov rbp, qword ptr [rip+_I_g_play_o7QAction]
	call _IMediaPlay_o12StandardIcon
	mov rdi, rax
	call _IqiconStandard_o5QIcono12StandardIcon
	mov rsi, rax
	mov rdi, rbp
	mov rax, qword ptr [rbp]
	call qword ptr [rax+80]
	mov rax, qword ptr [rip+_I_g_play_o7QAction]
	xor rsi, rsi
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	lea rdi, qword ptr [rip+.L21]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqaction_o7QActiono7QString
	mov qword ptr [rip+_I_g_stop_o7QAction], rax
	mov rbp, qword ptr [rip+_I_g_stop_o7QAction]
	call _IMediaStop_o12StandardIcon
	mov rdi, rax
	call _IqiconStandard_o5QIcono12StandardIcon
	mov rsi, rax
	mov rdi, rbp
	mov rax, qword ptr [rbp]
	call qword ptr [rax+80]
	mov rdi, qword ptr [rip+_I_size_PlaybackController]
	call _xi_alloc
	mov rbp, rax
	lea rax, qword ptr [rip+_I_vt_PlaybackController]
	mov qword ptr [rbp], rax
	mov rax, qword ptr [rip+_I_g_play_o7QAction]
	mov rsi, rbp
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+144]
	mov rax, qword ptr [rip+_I_g_stop_o7QAction]
	mov rsi, rbp
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+144]
	mov rax, qword ptr [rbx]
	add rax, 488
	mov rbp, qword ptr [rax]
	mov rsi, qword ptr [rip+_I_g_play_o7QAction]
	mov rdi, rbx
	call rbp
	mov rsi, qword ptr [rip+_I_g_stop_o7QAction]
	mov rdi, rbx
	call rbp
	mov rax, qword ptr [r13]
	mov rbx, qword ptr [rax+488]
	mov rsi, qword ptr [rip+_I_g_play_o7QAction]
	mov rdi, r13
	call rbx
	mov rsi, qword ptr [rip+_I_g_stop_o7QAction]
	mov rdi, r13
	call rbx
	mov rdi, qword ptr [rip+_I_size_BallWidget]
	call _xi_alloc
	lea rcx, qword ptr [rip+_I_vt_BallWidget]
	mov qword ptr [rax], rcx
	mov qword ptr [rip+_I_g_mainWidget_o7QWidget], rax
	mov rbx, qword ptr [rip+_I_g_mainWidget_o7QWidget]
	mov rdi, qword ptr [rip+_I_g_DIM_i]
	mov rsi, qword ptr [rip+_I_g_DIM_i]
	call _Iqsize_o5QSizeii
	mov rsi, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+264]
	mov rax, qword ptr [rip+_I_g_DIM_i]
	mov rsi, rax
	mov rdi, rax
	call _Iqpixmap_o7QPixmapii
	mov qword ptr [rip+_I_g_backBuffer_o7QPixmap], rax
	call _Iqtimer_o6QTimer
	mov qword ptr [rip+_I_g_timer_o6QTimer], rax
	mov rbx, qword ptr [rip+_I_g_timer_o6QTimer]
	mov rdi, qword ptr [rip+_I_size_AnimationTimer]
	call _xi_alloc
	lea rcx, qword ptr [rip+_I_vt_AnimationTimer]
	mov qword ptr [rax], rcx
	mov rsi, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+72]
	mov rax, qword ptr [rip+_I_g_timer_o6QTimer]
	xor rsi, rsi
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+40]
	mov rax, qword ptr [rip+_I_g_timer_o6QTimer]
	mov rsi, 40
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+32]
	mov rax, qword ptr [rip+_I_g_timer_o6QTimer]
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+48]
	mov rsi, qword ptr [rip+_I_g_mainWidget_o7QWidget]
	mov rdi, r12
	call qword ptr [r14+648]
	mov rdi, r12
	call qword ptr [r14+32]
	mov rdi, r15
	mov rax, qword ptr [r15]
	call qword ptr [rax+8]
	add rsp, 8
	pop r15
	pop r14
	pop r13
	pop r12
	pop rbp
	pop rbx
	ret

strdup:
	push rbx
	push r14
	sub rsp, 8
	mov r14, rdi
	mov rbx, qword ptr [r14]
	lea rdi, qword ptr [rbx*8+8]
	call _xi_alloc
.L22:
	mov rcx, qword ptr [r14+rbx*8]
	mov qword ptr [rax+rbx*8], rcx
	dec rbx
	cmp rbx, 0
	jge .L22
	add rax, 8
	add rsp, 8
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

	.bss
	.align 8
.globl _I_size_PlaybackController
_I_size_PlaybackController:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_vt_PlaybackController
_I_vt_PlaybackController:
	.zero 24
	.text

.section .ctors
	.align 8
	.quad _I_init_PlaybackController
	.text

	.section .rodata
	.align 8
.L20:
	.quad 4
	.quad 80
	.quad 108
	.quad 97
	.quad 121
	.text

	.section .rodata
	.align 8
.L19:
	.quad 8
	.quad 80
	.quad 108
	.quad 97
	.quad 121
	.quad 98
	.quad 97
	.quad 99
	.quad 107
	.text

	.section .rodata
	.align 8
.L21:
	.quad 4
	.quad 83
	.quad 116
	.quad 111
	.quad 112
	.text

	.section .data
	.align 8
.globl _I_g_DIM_i
_I_g_DIM_i:
	.quad 556
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

	.bss
	.align 8
.globl _I_g_timer_o6QTimer
_I_g_timer_o6QTimer:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_g_play_o7QAction
_I_g_play_o7QAction:
	.zero 8
	.text

	.bss
	.align 8
.globl _I_g_stop_o7QAction
_I_g_stop_o7QAction:
	.zero 8
	.text

