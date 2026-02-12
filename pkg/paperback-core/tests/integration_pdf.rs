use std::{fs, io::Cursor, path::Path};

use paperback_core::latest as paperback;
use paperback::Backup;
use paperback::ToPdf;

#[test]
fn generate_pdf_from_test_ged() -> Result<(), Box<dyn std::error::Error>> {
    // Determine repository root from manifest dir (pkg/paperback-core)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("failed to determine repository root")?;

    let test_file = repo_root.join("test.ged.7z");
    let secret = fs::read(&test_file).map_err(|e| format!("failed to read {:?}: {}", test_file, e))?;

    // Create a backup from the file bytes.
    let backup = Backup::new(3, &secret)?;
    let main_document = backup.main_document().clone();

    // Render to PDF in-memory via a BufWriter (required by printpdf::PdfDocument::save).
    let cursor = Cursor::new(Vec::new());
    let mut writer = std::io::BufWriter::new(cursor);
    main_document.to_pdf()?.save(&mut writer)?;
    // Extract inner Vec<u8>.
    let cursor = writer.into_inner().map_err(|e| format!("failed to into_inner BufWriter: {}", e))?;
    let bytes = cursor.into_inner();

    // Parse PDF and ensure it has more than one page (multi-page support).
    let doc = lopdf::Document::load_mem(&bytes)?;
    let pages = doc.get_pages().len();

    assert!(pages >= 2, "expected multi-page PDF, got {} page(s)", pages);

    Ok(())
}
