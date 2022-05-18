# Compiled using "xic" by Owen Arden, Catalin Dumitru, Wenzel Jakob, and Danfeng Zhang
# Command line: java -jar xic.jar -libpath ../../../../pa5/pa5/runtime/include/ -libpath ../include/ --target linux layouts.xi

.file "layouts.xi"
.intel_syntax noprefix
.text

.globl _Imain_paai
_Imain_paai:
	push rbx
	push rbp
	push r12
	push r13
	push r14
	push r15
	sub rsp, 328
	call _Iqapplication_t2o12QApplicationaaiaai
	mov qword ptr [rsp+72], rax
	mov rax, qword ptr [rsp+72]
	mov qword ptr [rsp+72], rax
	call _Iqdialog_o7QDialog
	mov r12, rax
	call _IqvboxLayout_o11QVBoxLayout
	mov r14, rax
	mov rsi, r14
	mov rdi, r12
	mov rcx, qword ptr [r12]
	mov qword ptr [rsp+152], rcx
	mov rax, qword ptr [rsp+152]
	call qword ptr [rax+88]
	lea rdi, qword ptr [rip+.L0]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqlabel_o6QLabelo7QString
	mov r13, rax
	call _Iqslider_o7QSlider
	mov r15, rax
	mov rsi, r15
	mov rdi, r13
	mov rcx, qword ptr [r13]
	mov qword ptr [rsp+144], rcx
	mov rax, qword ptr [rsp+144]
	call qword ptr [rax+664]
	call _IHorizontal_o11Orientation
	mov rbx, qword ptr [r15]
	mov rsi, rax
	mov rdi, r15
	call qword ptr [rbx+640]
	mov rdx, 100
	xor rsi, rsi
	mov rdi, r15
	call qword ptr [rbx+664]
	mov rsi, 10
	mov rdi, r15
	call qword ptr [rbx+752]
	call _ITicksAbove_o18SliderTickPosition
	mov qword ptr [rsp+80], rax
	mov rax, qword ptr [rsp+80]
	mov qword ptr [rsp+80], rax
	mov rsi, qword ptr [rsp+80]
	mov rdi, r15
	call qword ptr [rbx+760]
	call _IqhboxLayout_o11QHBoxLayout
	mov rbp, rax
	mov rax, qword ptr [rbp]
	mov rbx, qword ptr [rax+56]
	mov rsi, r13
	mov rdi, rbp
	call rbx
	mov rsi, r15
	mov rdi, rbp
	call rbx
	mov rax, qword ptr [r14]
	mov rax, qword ptr [rax+40]
	mov rsi, rbp
	mov rdi, r14
	call rax
	lea rdi, qword ptr [rip+.L1]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqcheckbox_o9QCheckBoxo7QString
	mov qword ptr [rsp+280], rax
	mov rax, qword ptr [rsp+280]
	mov qword ptr [rsp+280], rax
	mov rcx, qword ptr [r14]
	add rcx, 56
	mov rbp, qword ptr [rcx]
	mov rsi, qword ptr [rsp+280]
	mov rdi, r14
	call rbp
	lea rax, qword ptr [rip+.L2]
	mov qword ptr [rsp+64], rax
	mov rdi, qword ptr [rsp+64]
	call strdup
	mov qword ptr [rsp+112], rax
	mov rax, qword ptr [rsp+112]
	mov qword ptr [rsp+112], rax
	mov rdi, qword ptr [rsp+112]
	call _Iqs_o7QStringai
	mov r13, rax
	call _Iqbuttongroup_o12QButtonGroup
	mov qword ptr [rsp+296], rax
	mov rcx, qword ptr [rsp+296]
	mov qword ptr [rsp+296], rcx
	mov rax, qword ptr [rsp+296]
	mov qword ptr [rip+_I_g_bg_o12QButtonGroup], rax
	mov rdi, 1
	call _IqsNum_o7QStringi
	mov qword ptr [rsp+208], rax
	mov rax, qword ptr [rsp+208]
	mov qword ptr [rsp+208], rax
	mov rcx, qword ptr [r13]
	mov qword ptr [rsp+232], rcx
	mov rax, qword ptr [rsp+232]
	mov r15, qword ptr [rax+264]
	mov rsi, qword ptr [rsp+208]
	mov rdi, r13
	call r15
	mov qword ptr [rsp+200], rax
	mov rax, qword ptr [rsp+200]
	mov qword ptr [rsp+200], rax
	mov rdi, qword ptr [rsp+200]
	call _Iqradio_o12QRadioButtono7QString
	mov rbx, rax
	mov rsi, rbx
	mov rdi, r14
	call rbp
	mov rax, qword ptr [rip+_I_g_bg_o12QButtonGroup]
	mov qword ptr [rsp+128], rax
	mov rsi, rbx
	mov rdi, qword ptr [rsp+128]
	mov rcx, qword ptr [rsp+128]
	mov rax, qword ptr [rcx]
	mov qword ptr [rsp+224], rax
	mov rcx, qword ptr [rsp+224]
	call qword ptr [rcx+8]
	mov rbx, 1
.L3:
	cmp rbx, 5
	jge .L5
	mov qword ptr [rsp+176], rbx
	mov rax, qword ptr [rsp+176]
	add qword ptr [rsp+176], 1
	mov rdi, qword ptr [rsp+176]
	call _IqsNum_o7QStringi
	mov rsi, rax
	mov rdi, r13
	call r15
	mov qword ptr [rsp+160], rax
	mov rax, qword ptr [rsp+160]
	mov qword ptr [rsp+160], rax
	mov rdi, qword ptr [rsp+160]
	call _Iqradio_o12QRadioButtono7QString
	mov qword ptr [rsp+48], rax
	mov rax, qword ptr [rsp+48]
	mov qword ptr [rsp+48], rax
	mov rsi, qword ptr [rsp+48]
	mov rdi, r14
	call rbp
	mov rax, qword ptr [rip+_I_g_bg_o12QButtonGroup]
	mov rsi, qword ptr [rsp+48]
	mov rdi, rax
	mov rax, qword ptr [rax]
	mov qword ptr [rsp+304], rax
	mov rcx, qword ptr [rsp+304]
	call qword ptr [rcx+8]
	inc rbx
	jmp .L3
.L5:
	lea rdi, qword ptr [rip+.L7]
	call strdup
	mov qword ptr [rsp+184], rax
	mov rax, qword ptr [rsp+184]
	mov qword ptr [rsp+184], rax
	mov rdi, qword ptr [rsp+184]
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqgroupbox_o9QGroupBoxo7QString
	mov rbx, rax
	mov rsi, rbx
	mov rdi, r14
	call rbp
	call _IqvboxLayout_o11QVBoxLayout
	mov r13, rax
	mov r15, qword ptr [rbx]
	mov rsi, r13
	mov rdi, rbx
	call qword ptr [r15+88]
	mov rsi, 1
	mov rdi, rbx
	call qword ptr [r15+648]
	lea rax, qword ptr [rip+.L8]
	mov qword ptr [rsp+216], rax
	mov rdi, qword ptr [rsp+216]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rbp, rax
	mov rdi, 1
	call _IqsNum_o7QStringi
	mov qword ptr [rsp+40], rax
	mov rax, qword ptr [rsp+40]
	mov qword ptr [rsp+40], rax
	mov rax, qword ptr [rbp]
	mov qword ptr [rsp+56], rax
	mov rcx, qword ptr [rsp+56]
	mov rbx, qword ptr [rcx+264]
	mov rsi, qword ptr [rsp+40]
	mov rdi, rbp
	call rbx
	mov qword ptr [rsp+16], rax
	mov rax, qword ptr [rsp+16]
	mov qword ptr [rsp+16], rax
	mov rdi, qword ptr [rsp+16]
	call _Iqradio_o12QRadioButtono7QString
	mov qword ptr [rsp+104], rax
	mov rax, qword ptr [rsp+104]
	mov qword ptr [rsp+104], rax
	mov rcx, qword ptr [r13]
	mov qword ptr [rsp+264], rcx
	mov rax, qword ptr [rsp+264]
	mov rax, qword ptr [rax+56]
	mov qword ptr [rsp+8], rax
	mov rsi, qword ptr [rsp+104]
	mov rdi, r13
	call qword ptr [rsp+8]
	mov r15, 1
.L9:
	cmp r15, 5
	jge .L11
	mov qword ptr [rsp+24], r15
	mov rax, qword ptr [rsp+24]
	add qword ptr [rsp+24], 1
	mov rdi, qword ptr [rsp+24]
	call _IqsNum_o7QStringi
	mov rsi, rax
	mov rdi, rbp
	call rbx
	mov qword ptr [rsp+136], rax
	mov rax, qword ptr [rsp+136]
	mov qword ptr [rsp+136], rax
	mov rdi, qword ptr [rsp+136]
	call _Iqradio_o12QRadioButtono7QString
	mov qword ptr [rsp+32], rax
	mov rax, qword ptr [rsp+32]
	mov qword ptr [rsp+32], rax
	mov rsi, qword ptr [rsp+32]
	mov rdi, r13
	call qword ptr [rsp+8]
	inc r15
	jmp .L9
.L11:
	lea rax, qword ptr [rip+.L0]
	mov qword ptr [rsp+96], rax
	mov rdi, qword ptr [rsp+96]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqlabel_o6QLabelo7QString
	mov r13, rax
	call _Iqlineedit_o9QLineEdit
	mov rbp, rax
	mov rsi, rbp
	mov rdi, r13
	mov rcx, qword ptr [r13]
	mov qword ptr [rsp+120], rcx
	mov rax, qword ptr [rsp+120]
	call qword ptr [rax+664]
	call _IqhboxLayout_o11QHBoxLayout
	mov rbx, rax
	mov rax, qword ptr [rbx]
	mov r15, qword ptr [rax+56]
	mov rsi, r13
	mov rdi, rbx
	call r15
	mov rsi, rbp
	mov rdi, rbx
	call r15
	mov rcx, qword ptr [r14]
	mov r15, qword ptr [rcx+40]
	mov rsi, rbx
	mov rdi, r14
	call r15
	call _Iqtextedit_o9QTextEdit
	mov qword ptr [rsp+240], rax
	mov rax, qword ptr [rsp+240]
	mov qword ptr [rsp+240], rax
	mov rax, qword ptr [r14]
	add rax, 56
	mov rax, qword ptr [rax]
	mov rsi, qword ptr [rsp+240]
	mov rdi, r14
	call rax
	call _IqhboxLayout_o11QHBoxLayout
	mov rbp, rax
	mov rax, qword ptr [rbp]
	mov rdi, rbp
	call qword ptr [rax+48]
	lea rax, qword ptr [rip+.L13]
	mov qword ptr [rsp+88], rax
	mov rdi, qword ptr [rsp+88]
	call strdup
	mov qword ptr [rsp+168], rax
	mov rax, qword ptr [rsp+168]
	mov qword ptr [rsp+168], rax
	mov rdi, qword ptr [rsp+168]
	call _Iqs_o7QStringai
	mov qword ptr [rsp+192], rax
	mov rax, qword ptr [rsp+192]
	mov qword ptr [rsp+192], rax
	mov rdi, qword ptr [rsp+192]
	call _Iqpushbutton_o11QPushButtono7QString
	mov rbx, rax
	call _IDialogOkButton_o12StandardIcon
	mov rdi, rax
	call _IqiconStandard_o5QIcono12StandardIcon
	mov rsi, rax
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	call qword ptr [rax+640]
	mov rax, qword ptr [rbp]
	add rax, 56
	mov r13, qword ptr [rax]
	mov rsi, rbx
	mov rdi, rbp
	call r13
	lea rax, qword ptr [rip+.L14]
	mov qword ptr [rsp+320], rax
	mov rdi, qword ptr [rsp+320]
	call strdup
	mov qword ptr [rsp+312], rax
	mov rax, qword ptr [rsp+312]
	mov qword ptr [rsp+312], rax
	mov rdi, qword ptr [rsp+312]
	call _Iqs_o7QStringai
	mov qword ptr [rsp+288], rax
	mov rax, qword ptr [rsp+288]
	mov qword ptr [rsp+288], rax
	mov rdi, qword ptr [rsp+288]
	call _Iqpushbutton_o11QPushButtono7QString
	mov rbx, rax
	call _IDialogCancelButton_o12StandardIcon
	mov qword ptr [rsp+272], rax
	mov rax, qword ptr [rsp+272]
	mov qword ptr [rsp+272], rax
	mov rdi, qword ptr [rsp+272]
	call _IqiconStandard_o5QIcono12StandardIcon
	mov qword ptr [rsp+256], rax
	mov rax, qword ptr [rsp+256]
	mov qword ptr [rsp+256], rax
	mov rsi, qword ptr [rsp+256]
	mov rdi, rbx
	mov rax, qword ptr [rbx]
	mov qword ptr [rsp+248], rax
	mov rcx, qword ptr [rsp+248]
	call qword ptr [rcx+640]
	mov rsi, rbx
	mov rdi, rbp
	call r13
	mov rsi, rbp
	mov rdi, r14
	call r15
	mov rdi, r12
	mov rax, qword ptr [r12]
	call qword ptr [rax+32]
	mov rdi, qword ptr [rsp+72]
	mov rcx, qword ptr [rsp+72]
	mov rax, qword ptr [rcx]
	call qword ptr [rax+8]
	add rsp, 328
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
.L15:
	mov rcx, qword ptr [r14+rbx*8]
	mov qword ptr [rax+rbx*8], rcx
	dec rbx
	cmp rbx, 0
	jge .L15
	add rax, 8
	add rsp, 8
	pop r14
	pop rbx
	ret

	.section .rodata
	.align 8
.L2:
	.quad 1
	.quad 65
	.text

	.section .rodata
	.align 8
.L8:
	.quad 1
	.quad 66
	.text

	.section .rodata
	.align 8
.L14:
	.quad 6
	.quad 67
	.quad 97
	.quad 110
	.quad 99
	.quad 101
	.quad 108
	.text

	.section .rodata
	.align 8
.L1:
	.quad 8
	.quad 67
	.quad 104
	.quad 101
	.quad 99
	.quad 107
	.quad 98
	.quad 111
	.quad 120
	.text

	.section .rodata
	.align 8
.L7:
	.quad 5
	.quad 71
	.quad 114
	.quad 111
	.quad 117
	.quad 112
	.text

	.section .rodata
	.align 8
.L13:
	.quad 2
	.quad 79
	.quad 107
	.text

	.section .rodata
	.align 8
.L0:
	.quad 5
	.quad 86
	.quad 97
	.quad 108
	.quad 117
	.quad 101
	.text

	.bss
	.align 8
.globl _I_g_bg_o12QButtonGroup
_I_g_bg_o12QButtonGroup:
	.zero 8
	.text

