use chip8_emulator_rs::assembler::{assemble_text, AssemblerError};

#[test]
fn assemble_basic_program_with_label_jump() {
    let source = "
        ORG 0x200
    start:
        LD V0, 1
        ADD V0, 2
        JP start
    ";

    let rom = assemble_text(source, 0x200).unwrap();

    assert_eq!(rom, vec![0x60, 0x01, 0x70, 0x02, 0x12, 0x00]);
}

#[test]
fn assemble_data_directives_db_and_dw() {
    let source = "
        ORG 0x200
        DB 0x12, 34, 'A'
        DB \"BC\"
        DW 0xABCD
    ";

    let rom = assemble_text(source, 0x200).unwrap();

    assert_eq!(rom, vec![0x12, 34, 0x41, 0x42, 0x43, 0xAB, 0xCD]);
}

#[test]
fn assemble_ld_variants_and_draw() {
    let source = "
        LD I, sprite
        LD V1, DT
        LD DT, V1
        LD ST, V1
        LD F, V1
        LD B, V1
        LD [I], V1
        LD V1, [I]
        DRW V1, V2, 5
    sprite:
        DB 0xFF
    ";

    let rom = assemble_text(source, 0x200).unwrap();
    let expected = vec![
        0xA2, 0x12, 0xF1, 0x07, 0xF1, 0x15, 0xF1, 0x18, 0xF1, 0x29, 0xF1, 0x33, 0xF1, 0x55, 0xF1,
        0x65, 0xD1, 0x25, 0xFF,
    ];

    assert_eq!(rom, expected);
}

#[test]
fn assemble_org_pads_rom() {
    let source = "
        ORG 0x200
        JP 0x206
        ORG 0x206
        RET
    ";

    let rom = assemble_text(source, 0x200).unwrap();

    assert_eq!(rom, vec![0x12, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xEE]);
}

#[test]
fn assemble_errors_on_invalid_register() {
    let result = assemble_text("LD V16, 1", 0x200);
    assert!(matches!(result, Err(AssemblerError { .. })));
}
