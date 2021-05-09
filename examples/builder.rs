use vtil_parser::{
    ArchitectureIdentifier, ImmediateDesc, InstructionBuilder, Result, Routine, Vip,
};

fn main() -> Result<()> {
    let mut routine = Routine::new(ArchitectureIdentifier::Virtual);
    let basic_block = routine.create_block(Vip(0)).unwrap();
    let tmp0 = basic_block.tmp(64);
    let mut builder = InstructionBuilder::from(basic_block);
    builder.mov(tmp0, ImmediateDesc::new(0xA57E6F0335298D0u64, 64).into());
    std::fs::write("built.vtil", routine.into_bytes()?)?;
    Ok(())
}
