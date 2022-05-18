# Compiled using "xic" by Drew Dunne, Jacob Glueck, Aaron Wisner, and Alex Libman
# Command line: ./xic -libpath ../include/ --target linux mandelbrot.xi

  .text
  .intel_syntax noprefix
  .align  4
_IMdbCalc_init_o7MdbCalco9MdbWidget:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp], r15
  mov r14, rdi
  mov rax, r14
  add rax, -40
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [rax], rsi
  mov rax, r14
  add rax, -16
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_x__offset_i]
  mov QWORD PTR [rax], rcx
  mov rax, r14
  add rax, -8
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_y__offset_i]
  mov QWORD PTR [rax], rcx
  mov rax, r14
  add rax, -32
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [rax], 0
  mov rax, r14
  add rax, -24
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [rax], 0
  call _Ireset__histogram_p
  mov rax, r14
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IMdbCalc_timeout_p:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 24], r12
  mov QWORD PTR [rsp], r13
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp + 16], r15
  .data
_CARR0: .quad 99,117,114,114,101,110,116,32,115,105,122,101,32,61,32
  .text
  mov r12, rdi
  mov r13, 2000
  cmp QWORD PTR [_I_g_done_b], 0
  je _L108
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
_L108:
_L112:
  cmp QWORD PTR [_I_g_done_b], 0
  jne _L110
  cmp r13, 0
  jle _L110
  mov rax, QWORD PTR [r12]
  mov rdi, r12
  call QWORD PTR [rax + 96]
  add r13, -1
  jmp _L112
_L110:
  cmp QWORD PTR [_I_g_done_b], 0
  jne _L114
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
_L114:
  mov rax, r12
  add rax, -40
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov r13, QWORD PTR [rax]
  mov r14, QWORD PTR [r13]
  mov rdi, 0
  mov rsi, 0
  mov rdx, QWORD PTR [_I_g_WINSIZE_i]
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  call _Iqrect_o5QRectiiii
  mov rsi, rax
  mov rdi, r13
  call QWORD PTR [r14 + 216]
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  mov QWORD PTR [_I_g_size_i], rax
  mov rdi, 128
  call _xi_alloc
  mov QWORD PTR [rax], 15
  add rax, 8
  mov rdi, rax
  lea rsi, QWORD PTR [_CARR0]
  mov rcx, 15
  rep movsq 
  mov rdi, rax
  call _Iprint_pai
  mov rdi, QWORD PTR [_I_g_size_i]
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprint_pai
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 10
  call _Iprint_pai
  mov rax, QWORD PTR [_I_g_final__size_i]
  cmp QWORD PTR [_I_g_size_i], rax
  jg _L116
  mov rax, QWORD PTR [r12]
  mov rdi, r12
  call QWORD PTR [rax + 56]
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
_L116:
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_zoom_i]
  cqo 
  idiv rcx
  mov QWORD PTR [_I_g_zoom_i], rax
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  add rax, QWORD PTR [_I_g_x__offset_i]
  mov QWORD PTR [_I_g_x__offset_i], rax
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  add rax, QWORD PTR [_I_g_y__offset_i]
  mov QWORD PTR [_I_g_y__offset_i], rax
  mov rax, QWORD PTR [_I_g_x__offset_i]
  add rax, QWORD PTR [_I_g_size_i]
  mov QWORD PTR [_I_g_x__final_i], rax
  mov rax, QWORD PTR [_I_g_y__offset_i]
  add rax, QWORD PTR [_I_g_size_i]
  mov QWORD PTR [_I_g_y__final_i], rax
  mov rax, r12
  add rax, -16
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_x__offset_i]
  mov QWORD PTR [rax], rcx
  mov rax, r12
  add rax, -8
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_y__offset_i]
  mov QWORD PTR [rax], rcx
  mov rax, r12
  add rax, -32
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [rax], 0
  add r12, -24
  add r12, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [r12], 0
  call _Iupdate__maxiter_p
  call _Ireset__histogram_p
  mov QWORD PTR [_I_g_done_b], 0
  mov rax, 1
  sub rax, QWORD PTR [_I_g_buffer_i]
  mov QWORD PTR [_I_g_buffer_i], rax
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
  .align  4
_IMdbCalc_calc_p:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 24], r12
  mov QWORD PTR [rsp], r13
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp + 16], r15
  mov r12, rdi
  mov r14, QWORD PTR [_I_g_plots_aaai]
  mov r13, QWORD PTR [_I_g_buffer_i]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L117
_L119:
  call _xi_out_of_bounds
_L117:
  cmp r13, 0
  jl _L119
  mov r13, QWORD PTR [r14 + r13*8]
  mov rax, r12
  add rax, -32
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  cmp QWORD PTR [rax], rcx
  jge _L121
  mov rax, r12
  add rax, -24
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  cmp QWORD PTR [rax], rcx
  jge _L123
  mov rax, r12
  add rax, -32
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov r14, QWORD PTR [rax]
  cmp r14, QWORD PTR [r13 - 8]
  jl _L124
_L126:
  call _xi_out_of_bounds
_L124:
  cmp r14, 0
  jl _L126
  mov r14, QWORD PTR [r13 + r14*8]
  mov rax, r12
  add rax, -24
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov r13, QWORD PTR [rax]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L127
_L129:
  call _xi_out_of_bounds
_L127:
  cmp r13, 0
  jl _L129
  mov rax, r12
  add rax, -16
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rdi, QWORD PTR [rax]
  mov rax, r12
  add rax, -8
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rsi, QWORD PTR [rax]
  call _Imandelbrot_iii
  mov QWORD PTR [r14 + r13*8], rax
  mov rcx, r12
  add rcx, -8
  add rcx, QWORD PTR [_I_size_MdbCalc]
  mov rax, r12
  add rax, -8
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rax, QWORD PTR [rax]
  add rax, QWORD PTR [_I_g_zoom_i]
  mov QWORD PTR [rcx], rax
  mov rcx, r12
  add rcx, -24
  add rcx, QWORD PTR [_I_size_MdbCalc]
  mov rax, QWORD PTR [_I_size_MdbCalc]
  sub rax, 24
  add rax, r12
  mov rax, QWORD PTR [rax]
  add rax, 1
  mov QWORD PTR [rcx], rax
_L130:
_L131:
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
_L123:
  mov rcx, r12
  add rcx, -16
  add rcx, QWORD PTR [_I_size_MdbCalc]
  mov rax, r12
  add rax, -16
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rax, QWORD PTR [rax]
  add rax, QWORD PTR [_I_g_zoom_i]
  mov QWORD PTR [rcx], rax
  mov rax, r12
  add rax, -8
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov rcx, QWORD PTR [_I_g_y__offset_i]
  mov QWORD PTR [rax], rcx
  mov rax, r12
  add rax, -24
  add rax, QWORD PTR [_I_size_MdbCalc]
  mov QWORD PTR [rax], 0
  mov rcx, r12
  add rcx, -32
  add rcx, QWORD PTR [_I_size_MdbCalc]
  mov rax, QWORD PTR [_I_size_MdbCalc]
  sub rax, 32
  add rax, r12
  mov rax, QWORD PTR [rax]
  add rax, 1
  mov QWORD PTR [rcx], rax
  jmp _L130
_L121:
  mov QWORD PTR [_I_g_done_b], 1
  jmp _L131
  .align  4
_I_init_MdbCalc:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp], r15
  lea r14, _I_size_MdbCalc
  mov rax, QWORD PTR [r14]
  cmp rax, 0
  jne _L133
  call _I_init_QTimer
  lea rax, _I_size_QTimer
  mov rax, QWORD PTR [rax]
  add rax, 40
  mov QWORD PTR [r14], rax
  lea rax, _I_vt_QTimer
  lea rcx, _I_vt_MdbCalc
  mov rsi, 0
_L134:
  cmp rsi, 10
  jge _L135
  mov rdx, QWORD PTR [rax + rsi*8]
  mov QWORD PTR [rcx + rsi*8], rdx
  inc rsi
  jmp _L134
_L135:
  lea rax, _IMdbCalc_init_o7MdbCalco9MdbWidget
  mov QWORD PTR [rcx + 88], rax
  lea rax, _IMdbCalc_timeout_p
  mov QWORD PTR [rcx + 64], rax
  lea rax, _IMdbCalc_calc_p
  mov QWORD PTR [rcx + 96], rax
_L133:
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IMdbWidget_init_o9MdbWidget:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp], r14
  mov QWORD PTR [rsp + 8], r15
  .data
_CARR1: .quad 77,97,110,100,101,108,98,114,111,116
  .text
  mov r13, rdi
  mov r14, QWORD PTR [r13]
  mov rdi, 88
  call _xi_alloc
  mov QWORD PTR [rax], 10
  add rax, 8
  mov rdi, rax
  lea rsi, QWORD PTR [_CARR1]
  mov rcx, 10
  rep movsq 
  mov rdi, rax
  call _Iqs_o7QStringai
  mov rdi, r13
  mov rsi, rax
  call QWORD PTR [r14 + 120]
  mov r14, QWORD PTR [r13]
  mov rdi, QWORD PTR [_I_g_WINSIZE_i]
  mov rsi, QWORD PTR [_I_g_WINSIZE_i]
  call _Iqsize_o5QSizeii
  mov rdi, r13
  mov rsi, rax
  call QWORD PTR [r14 + 264]
  mov rax, r13
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp]
  mov r15, QWORD PTR [rsp + 8]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IMdbWidget_paintEvent_po11QPaintEvent:
  # prologue
  sub rsp, 56
  # prologue: begin function body
  mov QWORD PTR [rsp], rbx
  mov QWORD PTR [rsp + 8], rbp
  mov QWORD PTR [rsp + 16], r12
  mov QWORD PTR [rsp + 24], r13
  mov QWORD PTR [rsp + 32], r14
  mov QWORD PTR [rsp + 40], r15
  mov rbx, rdi
  mov rdi, rsi
  mov rax, QWORD PTR [rdi]
  call QWORD PTR [rax + 16]
  mov rdi, rbx
  call _Iqpainter_o8QPaintero12QPaintDevice
  mov r13, rax
  cmp QWORD PTR [_I_g_done_b], 0
  je _L138
  mov rbx, QWORD PTR [_I_g_buffer_i]
_L139:
  mov rbp, QWORD PTR [_I_g_plots_aaai]
  cmp rbx, QWORD PTR [rbp - 8]
  jl _L140
_L142:
  call _xi_out_of_bounds
_L140:
  cmp rbx, 0
  jl _L142
  mov r14, QWORD PTR [rbp + rbx*8]
  mov rbp, 0
_L154:
  cmp rbp, QWORD PTR [_I_g_WINSIZE_i]
  jge _L144
  mov rbx, 0
_L153:
  cmp rbx, QWORD PTR [_I_g_WINSIZE_i]
  jge _L146
  cmp rbp, QWORD PTR [r14 - 8]
  jl _L147
_L149:
  call _xi_out_of_bounds
_L147:
  cmp rbp, 0
  jl _L149
  mov r12, QWORD PTR [r14 + rbp*8]
  cmp rbx, QWORD PTR [r12 - 8]
  jl _L150
_L152:
  call _xi_out_of_bounds
_L150:
  cmp rbx, 0
  jl _L152
  mov rdi, QWORD PTR [r12 + rbx*8]
  call _Iassign__color_o5Colori
  mov r12, rax
  mov r15, QWORD PTR [r13]
  mov rax, r12
  add rax, -24
  add rax, QWORD PTR [_I_size_Color]
  mov rdi, QWORD PTR [rax]
  mov rax, r12
  add rax, -16
  add rax, QWORD PTR [_I_size_Color]
  mov rsi, QWORD PTR [rax]
  mov rax, r12
  add rax, -8
  add rax, QWORD PTR [_I_size_Color]
  mov rdx, QWORD PTR [rax]
  call _Iqcolor_o6QColoriii
  mov rdi, rax
  call _Iqpen_o4QPeno6QColor
  mov rsi, rax
  mov rdi, r13
  call QWORD PTR [r15 + 112]
  mov r15, QWORD PTR [r13]
  mov rax, r12
  add rax, -24
  add rax, QWORD PTR [_I_size_Color]
  mov rdi, QWORD PTR [rax]
  mov rax, r12
  add rax, -16
  add rax, QWORD PTR [_I_size_Color]
  mov rsi, QWORD PTR [rax]
  add r12, -8
  add r12, QWORD PTR [_I_size_Color]
  mov rdx, QWORD PTR [r12]
  call _Iqcolor_o6QColoriii
  mov rdi, rax
  call _Iqbrush_o6QBrusho6QColor
  mov rsi, rax
  mov rdi, r13
  call QWORD PTR [r15 + 128]
  mov r12, QWORD PTR [r13]
  mov rdi, rbp
  mov rsi, rbx
  mov rdx, 2
  mov rcx, 2
  call _Iqrect_o5QRectiiii
  mov r15, rax
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  call QWORD PTR [rax + 120]
  mov rdi, r13
  mov rsi, r15
  mov rdx, rax
  call QWORD PTR [r12 + 200]
  inc rbx
  jmp _L153
_L146:
  inc rbp
  jmp _L154
_L144:
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  call QWORD PTR [rax + 16]
  mov rbx, QWORD PTR [rsp]
  mov rbp, QWORD PTR [rsp + 8]
  mov r12, QWORD PTR [rsp + 16]
  mov r13, QWORD PTR [rsp + 24]
  mov r14, QWORD PTR [rsp + 32]
  mov r15, QWORD PTR [rsp + 40]
  # epilog
  add rsp, 56
  ret 
_L138:
  mov rbx, 1
  sub rbx, QWORD PTR [_I_g_buffer_i]
  jmp _L139
  .align  4
_IMdbWidget_mouseDoubleClickEvent_po11QMouseEvent:
  # prologue
  # prologue: begin function body
  # epilog
  ret 
  .align  4
_I_init_MdbWidget:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp], r15
  lea r14, _I_size_MdbWidget
  mov rax, QWORD PTR [r14]
  cmp rax, 0
  jne _L156
  call _I_init_QWidget
  lea rax, _I_size_QWidget
  mov rax, QWORD PTR [rax]
  mov QWORD PTR [r14], rax
  lea rax, _I_vt_QWidget
  lea rcx, _I_vt_MdbWidget
  mov rsi, 0
_L157:
  cmp rsi, 78
  jge _L158
  mov rdx, QWORD PTR [rax + rsi*8]
  mov QWORD PTR [rcx + rsi*8], rdx
  inc rsi
  jmp _L157
_L158:
  lea rax, _IMdbWidget_init_o9MdbWidget
  mov QWORD PTR [rcx + 632], rax
  lea rax, _IMdbWidget_paintEvent_po11QPaintEvent
  mov QWORD PTR [rcx + 496], rax
  lea rax, _IMdbWidget_mouseDoubleClickEvent_po11QMouseEvent
  mov QWORD PTR [rcx + 520], rax
_L156:
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp]
  # epilog
  add rsp, 24
  ret 
  .align  4
_IColor_init_o5Coloriii:
  # prologue
  # prologue: begin function body
  mov rax, rdi
  add rax, -24
  add rax, QWORD PTR [_I_size_Color]
  mov QWORD PTR [rax], rsi
  mov rax, rdi
  add rax, -16
  add rax, QWORD PTR [_I_size_Color]
  mov QWORD PTR [rax], rdx
  mov rax, rdi
  add rax, -8
  add rax, QWORD PTR [_I_size_Color]
  mov QWORD PTR [rax], rcx
  mov rax, rdi
  # epilog
  ret 
  .align  4
_I_init_Color:
  # prologue
  # prologue: begin function body
  # epilog
  ret 
  .align  4
_ImkMatrix_aaii:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 32], rbp
  mov QWORD PTR [rsp], r12
  mov QWORD PTR [rsp + 8], r13
  mov QWORD PTR [rsp + 16], r14
  mov QWORD PTR [rsp + 24], r15
  mov r13, rdi
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], r13
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov r12, rax
  mov r14, 0
_L5:
  cmp r14, r13
  jge _L1
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], r13
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov rbp, rax
  cmp r14, QWORD PTR [r12 - 8]
  jl _L2
_L4:
  call _xi_out_of_bounds
_L2:
  cmp r14, 0
  jl _L4
  mov QWORD PTR [r12 + r14*8], rbp
  inc r14
  jmp _L5
_L1:
  mov rax, r12
  mov rbp, QWORD PTR [rsp + 32]
  mov r12, QWORD PTR [rsp]
  mov r13, QWORD PTR [rsp + 8]
  mov r14, QWORD PTR [rsp + 16]
  mov r15, QWORD PTR [rsp + 24]
  # epilog
  add rsp, 40
  ret 
  .align  4
_IsetupGlobals_p:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 24], r12
  mov QWORD PTR [rsp], r13
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp + 16], r15
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  imul rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov QWORD PTR [_I_g_scale_i], rcx
  mov rsi, QWORD PTR [_I_g_scale_i]
  mov rcx, 6
  imul rsi, rcx
  mov QWORD PTR [_I_g_size_i], rsi
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  mov QWORD PTR [_I_g_zoom_i], rax
  mov QWORD PTR [_I_g_final__size_i], 200000
  mov QWORD PTR [_I_g_buffer_i], 0
  mov QWORD PTR [_I_g_done_b], 0
  mov r13, 0
_L11:
  cmp r13, 2
  jge _L7
  mov rdi, QWORD PTR [_I_g_WINSIZE_i]
  add rdi, 1
  call _ImkMatrix_aaii
  mov r12, rax
  mov r14, QWORD PTR [_I_g_plots_aaai]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L8
_L10:
  call _xi_out_of_bounds
_L8:
  cmp r13, 0
  jl _L10
  mov QWORD PTR [r14 + r13*8], r12
  inc r13
  jmp _L11
_L7:
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
  .globl _Imain_paai
  .align  4
_Imain_paai:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 24], r12
  mov QWORD PTR [rsp], r13
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp + 16], r15
  mov r12, rdi
  call _IsetupGlobals_p
  mov rdi, r12
  call _Iqapplication_t2o12QApplicationaaiaai
  mov r13, rax
  mov r12, rdx
  mov rax, 0
  mov rcx, QWORD PTR [r12 - 8]
  cmp rcx, 2
  jl _L13
  mov rax, 1
  cmp rax, rcx
  jl _L14
  call _xi_out_of_bounds
_L14:
  mov rdi, QWORD PTR [r12 + 8]
  call _IparseInt_t2ibai
_L13:
  cmp rax, 8
  jne _L18
  mov QWORD PTR [_I_g_x__offset_i], -2670932
  mov QWORD PTR [_I_g_y__offset_i], 17309052
  mov QWORD PTR [_I_g_final__size_i], 200000
_L37:
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  mov rcx, QWORD PTR [_I_g_x__offset_i]
  sub rcx, rax
  mov QWORD PTR [_I_g_x__offset_i], rcx
  mov rcx, 2
  mov rax, QWORD PTR [_I_g_size_i]
  cqo 
  idiv rcx
  mov rcx, QWORD PTR [_I_g_y__offset_i]
  sub rcx, rax
  mov QWORD PTR [_I_g_y__offset_i], rcx
  mov rax, QWORD PTR [_I_g_x__offset_i]
  add rax, QWORD PTR [_I_g_size_i]
  mov QWORD PTR [_I_g_x__final_i], rax
  mov rax, QWORD PTR [_I_g_y__offset_i]
  add rax, QWORD PTR [_I_g_size_i]
  mov QWORD PTR [_I_g_y__final_i], rax
  lea rax, _I_size_MdbWidget
  mov rdi, QWORD PTR [rax]
  lea r12, _I_vt_MdbWidget
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], r12
  mov rax, QWORD PTR [rdi]
  call QWORD PTR [rax + 632]
  mov r14, rax
  mov rax, QWORD PTR [r14]
  mov rdi, r14
  call QWORD PTR [rax + 32]
  lea rax, _I_size_MdbCalc
  mov rdi, QWORD PTR [rax]
  lea r12, _I_vt_MdbCalc
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], r12
  mov rax, QWORD PTR [rdi]
  mov rsi, r14
  call QWORD PTR [rax + 88]
  mov r12, rax
  mov rax, QWORD PTR [r12]
  mov rdi, r12
  mov rsi, 0
  call QWORD PTR [rax + 40]
  mov rax, QWORD PTR [r12]
  mov rdi, r12
  mov rsi, 5
  call QWORD PTR [rax + 32]
  mov rax, QWORD PTR [r12]
  mov rdi, r12
  call QWORD PTR [rax + 48]
  mov rax, QWORD PTR [r13]
  mov rdi, r13
  call QWORD PTR [rax + 8]
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
_L18:
  cmp rax, 1
  jne _L20
  mov QWORD PTR [_I_g_x__offset_i], -12560912
  mov QWORD PTR [_I_g_y__offset_i], 1677722
  mov QWORD PTR [_I_g_final__size_i], 20000
_L36:
  jmp _L37
_L20:
  cmp rax, 2
  jne _L22
  mov QWORD PTR [_I_g_x__offset_i], 4563275
  mov QWORD PTR [_I_g_y__offset_i], 8073700
  mov rax, QWORD PTR [_I_g_WINSIZE_i]
  sal rax, 1
  mov QWORD PTR [_I_g_final__size_i], rax
_L35:
  jmp _L36
_L22:
  cmp rax, 3
  jne _L24
  mov QWORD PTR [_I_g_x__offset_i], -22960316
  mov QWORD PTR [_I_g_y__offset_i], 83888
  mov QWORD PTR [_I_g_final__size_i], 10000
_L34:
  jmp _L35
_L24:
  cmp rax, 4
  jne _L26
  mov QWORD PTR [_I_g_x__offset_i], -29490000
  mov QWORD PTR [_I_g_y__offset_i], 0
  mov QWORD PTR [_I_g_final__size_i], 400000
_L33:
  jmp _L34
_L26:
  cmp rax, 5
  jne _L28
  mov QWORD PTR [_I_g_x__offset_i], -1476396
  mov QWORD PTR [_I_g_y__offset_i], 10978300
  mov QWORD PTR [_I_g_final__size_i], 20000
_L32:
  jmp _L33
_L28:
  cmp rax, 7
  jne _L30
  mov QWORD PTR [_I_g_x__offset_i], -11884460
  mov QWORD PTR [_I_g_y__offset_i], 5703649
  mov rax, QWORD PTR [_I_g_WINSIZE_i]
  sal rax, 1
  mov QWORD PTR [_I_g_final__size_i], rax
_L31:
  jmp _L32
_L30:
  mov QWORD PTR [_I_g_x__offset_i], 0
  mov QWORD PTR [_I_g_y__offset_i], 0
  mov rax, QWORD PTR [_I_g_size_i]
  mov QWORD PTR [_I_g_final__size_i], rax
  jmp _L31
  .align  4
_Iupdate__maxiter_p:
  # prologue
  sub rsp, 40
  # prologue: begin function body
  mov QWORD PTR [rsp + 24], r12
  mov QWORD PTR [rsp], r13
  mov QWORD PTR [rsp + 8], r14
  mov QWORD PTR [rsp + 16], r15
  .data
_CARR2: .quad 109,97,120,105,116,101,114,32,61,32
  .text
  mov r13, QWORD PTR [_I_g_maxiter_i]
  add r13, -5
  mov r12, 0
_L43:
  cmp r13, QWORD PTR [_I_g_maxiter_i]
  jge _L39
  mov r14, QWORD PTR [_I_g_histogram_ai]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L40
_L42:
  call _xi_out_of_bounds
_L40:
  cmp r13, 0
  jl _L42
  mov rcx, QWORD PTR [r14 + r13*8]
  add rcx, r12
  mov r12, rcx
  inc r13
  jmp _L43
_L39:
  mov rcx, 10000
  imul r12, rcx
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  mov rax, r12
  cqo 
  idiv rcx
  mov rcx, QWORD PTR [_I_g_WINSIZE_i]
  cqo 
  idiv rcx
  cmp rax, QWORD PTR [_I_g_GROW__ITER__PERCENT_i]
  jle _L45
  mov rax, QWORD PTR [_I_g_maxiter_i]
  lea rax, QWORD PTR [rax + rax*2]
  mov rcx, 2
  cqo 
  idiv rcx
  mov QWORD PTR [_I_g_maxiter_i], rax
  mov rax, QWORD PTR [_I_g_MAXITER_i]
  cmp QWORD PTR [_I_g_maxiter_i], rax
  jle _L47
  mov rax, QWORD PTR [_I_g_MAXITER_i]
  mov QWORD PTR [_I_g_maxiter_i], rax
_L47:
  mov rdi, 88
  call _xi_alloc
  mov QWORD PTR [rax], 10
  add rax, 8
  mov rdi, rax
  lea rsi, QWORD PTR [_CARR2]
  mov rcx, 10
  rep movsq 
  mov rdi, rax
  call _Iprint_pai
  mov rdi, QWORD PTR [_I_g_maxiter_i]
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprint_pai
  mov rdi, 16
  call _xi_alloc
  mov QWORD PTR [rax], 1
  add rax, 8
  mov QWORD PTR [rax], 10
  mov rdi, rax
  call _Iprint_pai
_L45:
  mov r12, QWORD PTR [rsp + 24]
  mov r13, QWORD PTR [rsp]
  mov r14, QWORD PTR [rsp + 8]
  mov r15, QWORD PTR [rsp + 16]
  # epilog
  add rsp, 40
  ret 
  .align  4
_Ireset__histogram_p:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp], r14
  mov QWORD PTR [rsp + 8], r15
  mov r14, 0
_L53:
  cmp r14, QWORD PTR [_I_g_maxiter_i]
  jg _L49
  mov r13, QWORD PTR [_I_g_histogram_ai]
  cmp r14, QWORD PTR [r13 - 8]
  jl _L50
_L52:
  call _xi_out_of_bounds
_L50:
  cmp r14, 0
  jl _L52
  mov QWORD PTR [r13 + r14*8], 0
  inc r14
  jmp _L53
_L49:
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp]
  mov r15, QWORD PTR [rsp + 8]
  # epilog
  add rsp, 24
  ret 
  .align  4
_Iprint__histogram_p:
  # prologue
  sub rsp, 24
  # prologue: begin function body
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp], r14
  mov QWORD PTR [rsp + 8], r15
  mov r13, 0
_L64:
  cmp r13, QWORD PTR [_I_g_maxiter_i]
  jg _L55
  mov r14, QWORD PTR [_I_g_histogram_ai]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L58
_L60:
  call _xi_out_of_bounds
_L58:
  cmp r13, 0
  jl _L60
  cmp QWORD PTR [r14 + r13*8], 0
  je _L57
  mov rdi, r13
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprint_pai
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 58
  call _Iprint_pai
  mov r14, QWORD PTR [_I_g_histogram_ai]
  cmp r13, QWORD PTR [r14 - 8]
  jl _L61
_L63:
  call _xi_out_of_bounds
_L61:
  cmp r13, 0
  jl _L63
  mov rdi, QWORD PTR [r14 + r13*8]
  call _IunparseInt_aii
  mov rdi, rax
  call _Iprint_pai
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 32
  call _Iprint_pai
  inc r13
_L57:
  jmp _L64
_L55:
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 10
  call _Iprint_pai
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp]
  mov r15, QWORD PTR [rsp + 8]
  # epilog
  add rsp, 24
  ret 
  .align  4
_Iwheel_ii:
  # prologue
  # prologue: begin function body
  mov rcx, 12
  mov rax, rdi
  cqo 
  idiv rcx
  cmp rdx, 5
  je _L65
  cmp rdx, 11
  jne _L66
_L65:
  mov rax, 128
  # epilog
  ret 
_L66:
  cmp rdx, 5
  jge _L69
  mov rax, 255
  # epilog
  ret 
_L69:
  mov rax, 0
  # epilog
  ret 
  .align  4
_Ibase__color_o5Colori:
  # prologue
  sub rsp, 56
  # prologue: begin function body
  mov QWORD PTR [rsp + 40], rbx
  mov QWORD PTR [rsp], rbp
  mov QWORD PTR [rsp + 8], r12
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp + 24], r14
  mov QWORD PTR [rsp + 32], r15
  mov r13, rdi
  lea rax, _I_size_Color
  mov rdi, QWORD PTR [rax]
  lea rbx, _I_vt_Color
  call _xi_alloc
  mov r12, rax
  mov QWORD PTR [r12], rbx
  mov r14, QWORD PTR [r12]
  mov rdi, r13
  add rdi, 6
  call _Iwheel_ii
  mov rbp, rax
  mov rdi, r13
  add rdi, 10
  call _Iwheel_ii
  mov rbx, rax
  mov rdi, r13
  add rdi, 2
  call _Iwheel_ii
  mov rdi, r12
  mov rsi, rbp
  mov rdx, rbx
  mov rcx, rax
  call QWORD PTR [r14 + 8]
  mov rbx, QWORD PTR [rsp + 40]
  mov rbp, QWORD PTR [rsp]
  mov r12, QWORD PTR [rsp + 8]
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp + 24]
  mov r15, QWORD PTR [rsp + 32]
  # epilog
  add rsp, 56
  ret 
  .align  4
_Iassign__color_o5Colori:
  # prologue
  sub rsp, 56
  # prologue: begin function body
  mov QWORD PTR [rsp], rbx
  mov QWORD PTR [rsp + 8], rbp
  mov QWORD PTR [rsp + 16], r12
  mov QWORD PTR [rsp + 24], r13
  mov QWORD PTR [rsp + 32], r14
  mov QWORD PTR [rsp + 40], r15
  mov r13, rdi
  mov rbx, QWORD PTR [_I_g_colors__init_ab]
  cmp r13, QWORD PTR [rbx - 8]
  jl _L74
_L76:
  call _xi_out_of_bounds
_L74:
  mov rax, 0
  cmp r13, rax
  setge al
  movzx r14, al
  cmp r14, 0
  je _L76
  cmp QWORD PTR [rbx + r13*8], 0
  je _L73
  mov rbx, QWORD PTR [_I_g_colors_ao5Color]
  cmp r13, QWORD PTR [rbx - 8]
  jl _L77
_L79:
  call _xi_out_of_bounds
_L77:
  cmp r14, 0
  je _L79
  mov rax, QWORD PTR [rbx + r13*8]
  mov rbx, QWORD PTR [rsp]
  mov rbp, QWORD PTR [rsp + 8]
  mov r12, QWORD PTR [rsp + 16]
  mov r13, QWORD PTR [rsp + 24]
  mov r14, QWORD PTR [rsp + 32]
  mov r15, QWORD PTR [rsp + 40]
  # epilog
  add rsp, 56
  ret 
_L73:
  mov rbx, QWORD PTR [_I_g_colors__init_ab]
  cmp r13, QWORD PTR [rbx - 8]
  jl _L80
_L82:
  call _xi_out_of_bounds
_L80:
  cmp r14, 0
  je _L82
  mov QWORD PTR [rbx + r13*8], 1
  cmp r13, QWORD PTR [_I_g_maxiter_i]
  jne _L84
  lea rax, _I_size_Color
  mov rdi, QWORD PTR [rax]
  lea rbx, _I_vt_Color
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], rbx
  mov rax, QWORD PTR [rdi]
  mov rsi, 0
  mov rdx, 0
  mov rcx, 0
  call QWORD PTR [rax + 8]
  mov rbx, rax
  mov r12, QWORD PTR [_I_g_colors_ao5Color]
  mov rbp, QWORD PTR [_I_g_maxiter_i]
  cmp rbp, QWORD PTR [r12 - 8]
  jl _L85
_L87:
  call _xi_out_of_bounds
_L85:
  cmp rbp, 0
  jl _L87
  mov QWORD PTR [r12 + rbp*8], rbx
  mov rax, rbx
  mov rbx, QWORD PTR [rsp]
  mov rbp, QWORD PTR [rsp + 8]
  mov r12, QWORD PTR [rsp + 16]
  mov r13, QWORD PTR [rsp + 24]
  mov r14, QWORD PTR [rsp + 32]
  mov r15, QWORD PTR [rsp + 40]
  # epilog
  add rsp, 56
  ret 
_L84:
  mov rbx, QWORD PTR [_I_g_RAMPSIZE_i]
  mov rax, r13
  cqo 
  idiv rbx
  mov rbx, QWORD PTR [_I_g_RAMPS_i]
  cqo 
  idiv rbx
  mov rdi, rdx
  mov rbx, QWORD PTR [_I_g_RAMPSIZE_i]
  mov rax, r13
  cqo 
  idiv rbx
  mov rbx, rdx
  mov rcx, QWORD PTR [_I_g_RAMPSIZE_i]
  mov rax, r13
  cqo 
  idiv rcx
  mov rcx, QWORD PTR [_I_g_RAMPS_i]
  cqo 
  idiv rcx
  mov rcx, QWORD PTR [_I_g_RAMPS_i]
  cqo 
  idiv rcx
  mov rbp, rdx
  call _Ibase__color_o5Colori
  mov r15, rax
  cmp rbp, 0
  jne _L89
  lea rax, _I_size_Color
  mov rdi, QWORD PTR [rax]
  lea rbp, _I_vt_Color
  call _xi_alloc
  mov QWORD PTR [rax], rbp
  mov rsi, rax
_L90:
  mov rcx, rsi
  add rcx, -24
  add rcx, QWORD PTR [_I_size_Color]
  mov rax, rsi
  add rax, -24
  add rax, QWORD PTR [_I_size_Color]
  mov rbp, 2
  mov rax, QWORD PTR [rax]
  cqo 
  idiv rbp
  mov QWORD PTR [rcx], rax
  mov rcx, rsi
  add rcx, -16
  add rcx, QWORD PTR [_I_size_Color]
  mov rax, rsi
  add rax, -16
  add rax, QWORD PTR [_I_size_Color]
  mov rbp, 2
  mov rax, QWORD PTR [rax]
  cqo 
  idiv rbp
  mov QWORD PTR [rcx], rax
  mov rbp, rsi
  add rbp, -8
  add rbp, QWORD PTR [_I_size_Color]
  mov rax, rsi
  add rax, -8
  add rax, QWORD PTR [_I_size_Color]
  mov rcx, 2
  mov rax, QWORD PTR [rax]
  cqo 
  idiv rcx
  mov rcx, rax
  mov QWORD PTR [rbp], rcx
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 24
  add rcx, r15
  mov rdx, QWORD PTR [_I_size_Color]
  sub rdx, 24
  add rdx, rsi
  mov rcx, QWORD PTR [rcx]
  sub rcx, QWORD PTR [rdx]
  imul rcx, rbx
  mov rbp, QWORD PTR [_I_g_RAMPSIZE_i]
  add rbp, -1
  mov rax, rcx
  cqo 
  idiv rbp
  mov r12, rax
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 24
  add rcx, rsi
  add r12, QWORD PTR [rcx]
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 16
  add rcx, r15
  mov rdx, QWORD PTR [_I_size_Color]
  sub rdx, 16
  add rdx, rsi
  mov rcx, QWORD PTR [rcx]
  sub rcx, QWORD PTR [rdx]
  imul rcx, rbx
  mov rbp, QWORD PTR [_I_g_RAMPSIZE_i]
  add rbp, -1
  mov rax, rcx
  cqo 
  idiv rbp
  mov rbp, rax
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 16
  add rcx, rsi
  add rbp, QWORD PTR [rcx]
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 8
  add rcx, r15
  mov rdx, QWORD PTR [_I_size_Color]
  sub rdx, 8
  add rdx, rsi
  mov rcx, QWORD PTR [rcx]
  sub rcx, QWORD PTR [rdx]
  imul rcx, rbx
  mov rbx, QWORD PTR [_I_g_RAMPSIZE_i]
  add rbx, -1
  mov rax, rcx
  cqo 
  idiv rbx
  mov rbx, rax
  mov rcx, QWORD PTR [_I_size_Color]
  sub rcx, 8
  add rcx, rsi
  add rbx, QWORD PTR [rcx]
  mov rcx, 255
  sub rcx, r12
  mov rdx, 255
  sub rdx, r12
  imul rcx, rdx
  mov rsi, 255
  mov rax, rcx
  cqo 
  idiv rsi
  mov rcx, rax
  mov r12, 255
  sub r12, rcx
  mov rcx, 255
  sub rcx, rbp
  mov rdx, 255
  sub rdx, rbp
  imul rcx, rdx
  mov rbp, 255
  mov rax, rcx
  cqo 
  idiv rbp
  mov rcx, rax
  mov rbp, 255
  sub rbp, rcx
  mov rcx, 255
  sub rcx, rbx
  mov rdx, 255
  sub rdx, rbx
  imul rcx, rdx
  mov rbx, 255
  mov rax, rcx
  cqo 
  idiv rbx
  mov rbx, 255
  sub rbx, rax
  mov rax, QWORD PTR [_I_g_colors_ao5Color]
  mov QWORD PTR [rsp + 48], rax
  mov rax, QWORD PTR [rsp + 48]
  cmp r13, QWORD PTR [rax - 8]
  jl _L91
_L93:
  call _xi_out_of_bounds
_L91:
  cmp r14, 0
  je _L93
  lea rax, _I_size_Color
  mov rdi, QWORD PTR [rax]
  lea r15, _I_vt_Color
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], r15
  mov rax, QWORD PTR [rdi]
  mov rsi, r12
  mov rdx, rbp
  mov rcx, rbx
  call QWORD PTR [rax + 8]
  mov rbx, QWORD PTR [rsp + 48]
  mov QWORD PTR [rbx + r13*8], rax
  mov rbx, QWORD PTR [_I_g_colors_ao5Color]
  cmp r13, QWORD PTR [rbx - 8]
  jl _L94
_L96:
  call _xi_out_of_bounds
_L94:
  cmp r14, 0
  je _L96
  mov rax, QWORD PTR [rbx + r13*8]
  mov rbx, QWORD PTR [rsp]
  mov rbp, QWORD PTR [rsp + 8]
  mov r12, QWORD PTR [rsp + 16]
  mov r13, QWORD PTR [rsp + 24]
  mov r14, QWORD PTR [rsp + 32]
  mov r15, QWORD PTR [rsp + 40]
  # epilog
  add rsp, 56
  ret 
_L89:
  mov rdi, rbp
  call _Ibase__color_o5Colori
  mov rsi, rax
  jmp _L90
  .align  4
_Imandelbrot_iii:
  # prologue
  sub rsp, 56
  # prologue: begin function body
  mov QWORD PTR [rsp + 40], rbx
  mov QWORD PTR [rsp], rbp
  mov QWORD PTR [rsp + 8], r12
  mov QWORD PTR [rsp + 16], r13
  mov QWORD PTR [rsp + 24], r14
  mov QWORD PTR [rsp + 32], r15
  mov r12, 0
  mov rbp, 0
  mov r13, 0
  mov r8, 0
  mov rbx, 0
_L100:
  add r13, r8
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  sal rcx, 2
  imul rcx, QWORD PTR [_I_g_SCALEROOT_i]
  cmp r13, rcx
  jge _L98
  cmp rbx, QWORD PTR [_I_g_maxiter_i]
  jge _L98
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, r12
  cqo 
  idiv rcx
  mov r8, rax
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, r12
  cqo 
  idiv rcx
  mov r14, rdx
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, rbp
  cqo 
  idiv rcx
  mov r11, rax
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, rbp
  cqo 
  idiv rcx
  mov rbp, rdx
  mov r10, r8
  sal r10, 1
  mov rcx, r10
  imul rcx, r14
  mov r9, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, rcx
  cqo 
  idiv r9
  mov r13, rax
  mov rcx, r8
  imul rcx, r8
  add r13, rcx
  mov r9, r11
  sal r9, 1
  mov rcx, r9
  imul rcx, rbp
  mov r8, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, rcx
  cqo 
  idiv r8
  mov r8, rax
  mov rcx, r11
  imul rcx, r11
  add r8, rcx
  mov r12, r13
  add r12, rdi
  sub r12, r8
  mov rcx, r10
  imul rcx, rbp
  mov rbp, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, rcx
  cqo 
  idiv rbp
  mov rbp, rax
  imul r10, r11
  add rbp, r10
  add rbp, rsi
  imul r9, r14
  mov rcx, QWORD PTR [_I_g_SCALEROOT_i]
  mov rax, r9
  cqo 
  idiv rcx
  mov rcx, rax
  add rbp, rcx
  inc rbx
  jmp _L100
_L98:
  mov r12, QWORD PTR [_I_g_histogram_ai]
  cmp rbx, QWORD PTR [r12 - 8]
  jl _L104
_L106:
  call _xi_out_of_bounds
_L104:
  cmp rbx, 0
  jl _L106
  mov rbp, QWORD PTR [_I_g_histogram_ai]
  cmp rbx, QWORD PTR [rbp - 8]
  jl _L101
_L103:
  call _xi_out_of_bounds
_L101:
  cmp rbx, 0
  jl _L103
  mov rax, QWORD PTR [rbp + rbx*8]
  add rax, 1
  mov QWORD PTR [r12 + rbx*8], rax
  mov rax, rbx
  mov rbx, QWORD PTR [rsp + 40]
  mov rbp, QWORD PTR [rsp]
  mov r12, QWORD PTR [rsp + 8]
  mov r13, QWORD PTR [rsp + 16]
  mov r14, QWORD PTR [rsp + 24]
  mov r15, QWORD PTR [rsp + 32]
  # epilog
  add rsp, 56
  ret 
  .p2align 4,,15
  _array_concat_bWFuZGVsYnJvdC54aQ:
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
  jle	_L173
  .p2align 4,,10
  .p2align 3
_L174:
  mov	rcx, QWORD PTR [r13+0+rdx*8]
  mov	QWORD PTR [rdi+8+rdx*8], rcx
  add	rdx, 1
  cmp	rbx, rdx
  jne	_L174
_L173:
  xor	edx, edx
  test	rbp, rbp
  lea	rsi, [rdi+rbx*8]
  jle	_L175
  .p2align 4,,10
  .p2align 3
_L176:
  mov	rcx, QWORD PTR [r12+rdx*8]
  mov	QWORD PTR [rsi+8+rdx*8], rcx
  add	rdx, 1
  cmp	rbp, rdx
  jne	_L176
_L175:
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
  jle	_L177
  test	r13, r13
  jle	_L177
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
_L178:
  mov	rdx, QWORD PTR [rsp]
  mov	rdi, QWORD PTR [rsp+8]
  lea	rcx, [rbp-8]
  mov	QWORD PTR [r12+8+rbx*8], rbp
  mov	rsi, r15
  add	rbx, 1
  add	rbp, r14
  call	_FUN_1_func1
  cmp	r13, rbx
  jg	_L178
_L177:
  add	rsp, 24
  pop	rbx
  pop	rbp
  pop	r12
  pop	r13
  pop	r14
  pop	r15
  ret
  
  .p2align 4,,15
  .local	_array_alloc_bWFuZGVsYnJvdC54aQ
  .type	_array_alloc_bWFuZGVsYnJvdC54aQ, @function
  _array_alloc_bWFuZGVsYnJvdC54aQ:
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
  js	_L179
  mov	rcx, QWORD PTR [rdi-8+rdx]
  xor	edi, edi
  test	rcx, rcx
  jns	_L180
  jmp	_L181
  .p2align 4,,10
  .p2align 3
_L182:
  mov	rcx, QWORD PTR [r12+rax*8]
  test	rcx, rcx
  js	_L181
_L180:
  imul	rdi, rcx
  lea	rdi, [rcx+1+rdi]
  mov	QWORD PTR [rbx+rax*8], rdi
  sub	rax, 1
  cmp	rax, -1
  jne	_L182
  sal	rdi, 3
_L183:
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
_L181:
  xor	eax, eax
  call	_xi_out_of_bounds
  lea	rsp, [rbp-24]
  xor	eax, eax
  pop	rbx
  pop	r12
  pop	r13
  pop	rbp
  ret
_L179:
  xor	edi, edi
  jmp	_L183
  .align  4
_I_global_init:
  # prologue
  sub rsp, 8
  # prologue: begin function body
  mov QWORD PTR [_I_g_WINSIZE_i], 256
  mov QWORD PTR [_I_g_MAXITER_i], 2000
  mov QWORD PTR [_I_g_HISTLEN_i], 2001
  mov QWORD PTR [_I_g_maxiter_i], 100
  mov QWORD PTR [_I_g_GROW__ITER__PERCENT_i], 1
  mov QWORD PTR [_I_g_SCALEROOT_i], 4096
  mov QWORD PTR [_I_g_final__size_i], 200000
  mov QWORD PTR [_I_g_buffer_i], 0
  mov QWORD PTR [_I_g_done_b], 0
  mov QWORD PTR [_I_g_RAMPSIZE_i], 8
  mov QWORD PTR [_I_g_RAMPS_i], 12
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov rax, QWORD PTR [_I_g_HISTLEN_i]
  mov QWORD PTR [rdi], rax
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov QWORD PTR [_I_g_histogram_ai], rax
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov rax, QWORD PTR [_I_g_HISTLEN_i]
  mov QWORD PTR [rdi], rax
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov QWORD PTR [_I_g_colors_ao5Color], rax
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov rax, QWORD PTR [_I_g_HISTLEN_i]
  mov QWORD PTR [rdi], rax
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov QWORD PTR [_I_g_colors__init_ab], rax
  mov rdi, 16
  call _xi_alloc
  mov rdi, rax
  mov QWORD PTR [rdi], 1
  add rdi, 8
  mov QWORD PTR [rdi], 2
  mov rsi, 1
  call _array_alloc_bWFuZGVsYnJvdC54aQ
  mov QWORD PTR [_I_g_plots_aaai], rax
  # epilog
  add rsp, 8
  ret 
  .section .ctors
  .align 8
  .quad _I_init_MdbCalc
  .quad _I_init_MdbWidget
  .quad _I_init_Color
  .quad _I_global_init
  .section .rodata
  .align 8
  .globl _I_size_Color
_I_size_Color:
  .quad 32
  .align 8
  .globl _I_vt_Color
_I_vt_Color:
  .quad 0
  .quad _IColor_init_o5Coloriii
  .bss
  .align 8
  .globl _I_size_MdbCalc
_I_size_MdbCalc:
  .zero 8
  .align 8
  .globl _I_vt_MdbCalc
_I_vt_MdbCalc:
  .zero 104
  .align 8
  .globl _I_size_MdbWidget
_I_size_MdbWidget:
  .zero 8
  .align 8
  .globl _I_vt_MdbWidget
_I_vt_MdbWidget:
  .zero 640
  .align 8
  .globl _I_g_WINSIZE_i
_I_g_WINSIZE_i:
  .zero 8
  .align 8
  .globl _I_g_MAXITER_i
_I_g_MAXITER_i:
  .zero 8
  .align 8
  .globl _I_g_HISTLEN_i
_I_g_HISTLEN_i:
  .zero 8
  .align 8
  .globl _I_g_maxiter_i
_I_g_maxiter_i:
  .zero 8
  .align 8
  .globl _I_g_GROW__ITER__PERCENT_i
_I_g_GROW__ITER__PERCENT_i:
  .zero 8
  .align 8
  .globl _I_g_SCALEROOT_i
_I_g_SCALEROOT_i:
  .zero 8
  .align 8
  .globl _I_g_scale_i
_I_g_scale_i:
  .zero 8
  .align 8
  .globl _I_g_final__size_i
_I_g_final__size_i:
  .zero 8
  .align 8
  .globl _I_g_buffer_i
_I_g_buffer_i:
  .zero 8
  .align 8
  .globl _I_g_done_b
_I_g_done_b:
  .zero 8
  .align 8
  .globl _I_g_RAMPSIZE_i
_I_g_RAMPSIZE_i:
  .zero 8
  .align 8
  .globl _I_g_RAMPS_i
_I_g_RAMPS_i:
  .zero 8
  .align 8
  .globl _I_g_size_i
_I_g_size_i:
  .zero 8
  .align 8
  .globl _I_g_zoom_i
_I_g_zoom_i:
  .zero 8
  .align 8
  .globl _I_g_x__offset_i
_I_g_x__offset_i:
  .zero 8
  .align 8
  .globl _I_g_y__offset_i
_I_g_y__offset_i:
  .zero 8
  .align 8
  .globl _I_g_x__final_i
_I_g_x__final_i:
  .zero 8
  .align 8
  .globl _I_g_y__final_i
_I_g_y__final_i:
  .zero 8
  .align 8
  .globl _I_g_histogram_ai
_I_g_histogram_ai:
  .zero 8
  .align 8
  .globl _I_g_colors_ao5Color
_I_g_colors_ao5Color:
  .zero 8
  .align 8
  .globl _I_g_colors__init_ab
_I_g_colors__init_ab:
  .zero 8
  .align 8
  .globl _I_g_plots_aaai
_I_g_plots_aaai:
  .zero 8
  .text
