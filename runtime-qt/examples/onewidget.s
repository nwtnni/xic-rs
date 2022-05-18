# Compiled using "xic" by Owen Arden, Catalin Dumitru, Wenzel Jakob, and Danfeng Zhang
# Command line: java -jar xic.jar -libpath ../../../../pa5/pa5/runtime/include/ -libpath ../include/ --target linux onewidget.xi

.file "onewidget.xi"
.intel_syntax noprefix
.text

.globl _Imain_paai
_Imain_paai:
	push rbx
	push r12
	push r14
	push r15
	sub rsp, 8
	call _Iqapplication_t2o12QApplicationaaiaai
	mov r14, rax
	mov r12, rdx
	mov r15, qword ptr [r12-8]
	cmp r15, 0
	jle .L2
	cmp r15, 0
	jbe call_abort
	mov rdi, qword ptr [r12]
	call _Iprintln_pai
	mov rbx, 1
.L0:
	cmp rbx, r15
	jge .L2
	cmp rbx, r15
	jae call_abort
	mov rdi, qword ptr [r12+rbx*8]
	call _Iprintln_pai
	inc rbx
	jmp .L0
.L2:
	lea rdi, qword ptr [rip+.L4]
	call strdup
	mov rdi, rax
	call _Iqs_o7QStringai
	mov rdi, rax
	call _Iqpushbutton_o11QPushButtono7QString
	mov rdi, rax
	mov rax, qword ptr [rax]
	call qword ptr [rax+32]
	mov rdi, r14
	mov rax, qword ptr [r14]
	call qword ptr [rax+8]
	add rsp, 8
	pop r15
	pop r14
	pop r12
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
.L5:
	mov rcx, qword ptr [r14+rbx*8]
	mov qword ptr [rax+rbx*8], rcx
	dec rbx
	cmp rbx, 0
	jge .L5
	add rax, 8
	add rsp, 8
	pop r14
	pop rbx
	ret

	.section .rodata
	.align 8
.L4:
	.quad 2
	.quad 104
	.quad 105
	.text

call_abort:
	call _xi_out_of_bounds
