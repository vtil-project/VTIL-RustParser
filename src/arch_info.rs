// BSD 3-Clause License
//
// Copyright © 2020-2021 Keegan Saunders
// Copyright © 2020-2021 VTIL Project
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//

// Extracted from the capstone source @ d71c95b0.
//
pub(crate) const X86_REGISTER_NAME_MAPPING: &[&str] = &[
    "NULL", "ah", "al", "ax", "bh", "bl", "bp", "bpl", "bx", "ch", "cl", "cs", "cx", "dh", "di",
    "dil", "dl", "ds", "dx", "eax", "ebp", "ebx", "ecx", "edi", "edx", "flags", "eip", "eiz", "es",
    "esi", "esp", "fpsw", "fs", "gs", "ip", "rax", "rbp", "rbx", "rcx", "rdi", "rdx", "rip", "riz",
    "rsi", "rsp", "si", "sil", "sp", "spl", "ss", "cr0", "cr1", "cr2", "cr3", "cr4", "cr5", "cr6",
    "cr7", "cr8", "cr9", "cr10", "cr11", "cr12", "cr13", "cr14", "cr15", "dr0", "dr1", "dr2",
    "dr3", "dr4", "dr5", "dr6", "dr7", "dr8", "dr9", "dr10", "dr11", "dr12", "dr13", "dr14",
    "dr15", "fp0", "fp1", "fp2", "fp3", "fp4", "fp5", "fp6", "fp7", "k0", "k1", "k2", "k3", "k4",
    "k5", "k6", "k7", "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "r8", "r9", "r10",
    "r11", "r12", "r13", "r14", "r15", "st(0)", "st(1)", "st(2)", "st(3)", "st(4)", "st(5)",
    "st(6)", "st(7)", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "xmm8",
    "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15", "xmm16", "xmm17", "xmm18",
    "xmm19", "xmm20", "xmm21", "xmm22", "xmm23", "xmm24", "xmm25", "xmm26", "xmm27", "xmm28",
    "xmm29", "xmm30", "xmm31", "ymm0", "ymm1", "ymm2", "ymm3", "ymm4", "ymm5", "ymm6", "ymm7",
    "ymm8", "ymm9", "ymm10", "ymm11", "ymm12", "ymm13", "ymm14", "ymm15", "ymm16", "ymm17",
    "ymm18", "ymm19", "ymm20", "ymm21", "ymm22", "ymm23", "ymm24", "ymm25", "ymm26", "ymm27",
    "ymm28", "ymm29", "ymm30", "ymm31", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6",
    "zmm7", "zmm8", "zmm9", "zmm10", "zmm11", "zmm12", "zmm13", "zmm14", "zmm15", "zmm16", "zmm17",
    "zmm18", "zmm19", "zmm20", "zmm21", "zmm22", "zmm23", "zmm24", "zmm25", "zmm26", "zmm27",
    "zmm28", "zmm29", "zmm30", "zmm31", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b",
    "r15b", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d", "r8w", "r9w", "r10w",
    "r11w", "r12w", "r13w", "r14w", "r15w",
];

// Extracted from the capstone source @ d71c95b0.
//
pub(crate) const AARCH64_REGISTER_NAME_MAPPING: &[&str] = &[
    "NULL", "x29", "x30", "nzcv", "sp", "wsp", "wzr", "xzr", "b0", "b1", "b2", "b3", "b4", "b5",
    "b6", "b7", "b8", "b9", "b10", "b11", "b12", "b13", "b14", "b15", "b16", "b17", "b18", "b19",
    "b20", "b21", "b22", "b23", "b24", "b25", "b26", "b27", "b28", "b29", "b30", "b31", "d0", "d1",
    "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "d10", "d11", "d12", "d13", "d14", "d15",
    "d16", "d17", "d18", "d19", "d20", "d21", "d22", "d23", "d24", "d25", "d26", "d27", "d28",
    "d29", "d30", "d31", "h0", "h1", "h2", "h3", "h4", "h5", "h6", "h7", "h8", "h9", "h10", "h11",
    "h12", "h13", "h14", "h15", "h16", "h17", "h18", "h19", "h20", "h21", "h22", "h23", "h24",
    "h25", "h26", "h27", "h28", "h29", "h30", "h31", "q0", "q1", "q2", "q3", "q4", "q5", "q6",
    "q7", "q8", "q9", "q10", "q11", "q12", "q13", "q14", "q15", "q16", "q17", "q18", "q19", "q20",
    "q21", "q22", "q23", "q24", "q25", "q26", "q27", "q28", "q29", "q30", "q31", "s0", "s1", "s2",
    "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "s12", "s13", "s14", "s15", "s16",
    "s17", "s18", "s19", "s20", "s21", "s22", "s23", "s24", "s25", "s26", "s27", "s28", "s29",
    "s30", "s31", "w0", "w1", "w2", "w3", "w4", "w5", "w6", "w7", "w8", "w9", "w10", "w11", "w12",
    "w13", "w14", "w15", "w16", "w17", "w18", "w19", "w20", "w21", "w22", "w23", "w24", "w25",
    "w26", "w27", "w28", "w29", "w30", "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9",
    "x10", "x11", "x12", "x13", "x14", "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22",
    "x23", "x24", "x25", "x26", "x27", "x28", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8",
    "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21",
    "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31",
];
