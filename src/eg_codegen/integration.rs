use crate::eg_codegen::parser;

#[test]
fn basic_traversal() {
    let byte_stream = [
        // ### Files 1 ###
        0x19, 0x46, 0x69, 0x6C, // Files element ID
        0xDA, // Files length = 90
        //
        // --- File 1 ---
        0x61, 0x46, // File element ID
        0xAB, // File length = 43
        0x61, 0x4E, // FileName element ID
        0x8A, // FileName length = 10
        0x66, 0x69, 0x6c, 0x65, 0x33, 0x2e, 0x68, 0x74, 0x6d,
        0x6c, // FileName data = "file3.html"
        0x46, 0x4D, // MimeType element ID
        0x89, // MimeType length = 9
        0x74, 0x65, 0x78, 0x74, 0x2f, 0x68, 0x74, 0x6d, 0x6c, // MimeType data = "text/html"
        0x46, 0x54, // ModificationTimestamp element ID
        0x88, // ModificationTimestamp length
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ModificationTimestamp data = 0
        0x46, 0x64, // Data element ID
        0x84, // Data length = 4
        0x01, 0x02, 0x03, 0x04, // Data data
        //
        // --- File 2 ---
        0x61, 0x46, // File element ID
        0xA9, // File length = 41
        0x46, 0x54, // ModificationTimestamp element ID
        0x88, // ModificationTimestamp length
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ModificationTimestamp data = 0
        0x46, 0x64, // Data element ID
        0x84, // Data length = 4
        0x01, 0x02, 0x03, 0x04, // Data data
        0x46, 0x4D, // MimeType element ID
        0x88, // MimeType length = 8
        0x74, 0x65, 0x78, 0x74, 0x2f, 0x63, 0x73, 0x76, // MimeType data = "text/csv"
        0x61, 0x4E, // FileName element ID
        0x89, // FileName length = 9
        0x66, 0x69, 0x6c, 0x65, 0x31, 0x2e, 0x63, 0x73, 0x76, // FileName data = "file2.csv"
        //
        // ### Files 2 ###
        0x19, 0x46, 0x69, 0x6C, // Files element ID
        0xAE, // Files length = 46
        //
        // --- File 1 ---
        0x61, 0x46, // File element ID
        0xAB, // File length = 43
        0x61, 0x4E, // FileName element ID
        0x89, // FileName length = 9
        0x66, 0x69, 0x6c, 0x65, 0x31, 0x2e, 0x74, 0x78, 0x74, // FileName data = "file1.txt"
        0x46, 0x4D, // MimeType element ID
        0x8A, // MimeType length = 10
        0x74, 0x65, 0x78, 0x74, 0x2f, 0x70, 0x6c, 0x61, 0x69,
        0x6e, // MimeType data = "text/plain"
        0x46, 0x54, // ModificationTimestamp element ID
        0x88, // ModificationTimestamp length
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ModificationTimestamp data = 0
        0x46, 0x64, // Data element ID
        0x84, // Data length = 4
        0x01, 0x02, 0x03, 0x04, // Data data
    ];

    let mut reader: parser::Readers<_> = parser::_DocumentReader::new(
        &byte_stream[..],
        parser::_DocumentState::new(byte_stream.len()),
    )
    .into();
    let mut result = Vec::new();

    loop {
        match reader {
            parser::Readers::_Document(_) => result.push("(None)"),
            parser::Readers::Files(_) => result.push("Files"),
            parser::Readers::File(_) => result.push("File"),
            parser::Readers::FileName(_) => result.push("FileName"),
            parser::Readers::MimeType(_) => result.push("MimeType"),
            parser::Readers::ModificationTimestamp(_) => result.push("ModTime"),
            parser::Readers::Data(_) => result.push("Data"),
            parser::Readers::None(_) => break,
        }

        reader = match reader {
            parser::Readers::_Document(r) => r.next().unwrap().into(),
            parser::Readers::Files(r) => r.next().unwrap().into(),
            parser::Readers::File(r) => r.next().unwrap().into(),
            parser::Readers::FileName(r) => r.next().unwrap().into(),
            parser::Readers::MimeType(r) => r.next().unwrap().into(),
            parser::Readers::ModificationTimestamp(r) => r.next().unwrap().into(),
            parser::Readers::Data(r) => r.next().unwrap().into(),
            parser::Readers::None(_) => unreachable!(),
        };
    }

    assert_eq!(
        result,
        vec![
            "(None)", "Files", "File", "FileName", "File", "MimeType", "File", "ModTime", "File",
            "Data", "File", "Files", "File", "ModTime", "File", "Data", "File", "MimeType", "File",
            "FileName", "File", "Files", "(None)", "Files", "File", "FileName", "File", "MimeType",
            "File", "ModTime", "File", "Data", "File", "Files", "(None)",
        ]
    );
}
