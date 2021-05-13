// Copyright © 2021 Keegan Saunders
//
// Permission to use, copy, modify, and/or distribute this software for
// any purpose with or without fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
// AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
// OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

use vtil_parser::{ArchitectureIdentifier, InstructionBuilder, RegisterDesc, Result, Routine, Vip};

fn main() -> Result<()> {
    let mut routine = Routine::new(ArchitectureIdentifier::Virtual);
    routine.header.arch_id = ArchitectureIdentifier::Amd64;
    let basic_block = routine.create_block(Vip(0)).unwrap();
    let mut builder = InstructionBuilder::from(basic_block);
    let tmp1 = RegisterDesc::X86_REG_RAX;

    for i in 0..100 {
        builder
            .add(tmp1, 13u32.into())
            .nop()
            .sub(tmp1, 12u32.into())
            .nop()
            .add(tmp1, 14u32.into())
            .mov(tmp1, tmp1.into())
            .sub(tmp1, tmp1.into())
            .xor(tmp1, (i as u32).into())
            .push(tmp1.into());
    }

    builder.vpinr(tmp1).vexit(0u64.into());

    std::fs::write("built.vtil", routine.into_bytes()?)?;
    Ok(())
}
