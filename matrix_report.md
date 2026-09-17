# File2File Conversion Matrix Report

| Input | Output | Status | Engine | Duration | Error |
|-------|--------|--------|--------|----------|-------|
| .3gp | .mp4 | ❌ FAIL | FFmpeg | 21ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .webm | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .mkv | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .mov | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .avi | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .mp3 | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .wav | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .3gp | .flac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .7z | .zip | ✅ OK | Rust-Native | 1ms |  |
| .aac | .mp3 | ❌ FAIL | FFmpeg | 5ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .aac | .wav | ❌ FAIL | FFmpeg | 6ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .aac | .flac | ❌ FAIL | FFmpeg | 6ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .aac | .ogg | ❌ FAIL | FFmpeg | 3ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .aac | .m4a | ❌ FAIL | FFmpeg | 7ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .aac | .opus | ❌ FAIL | FFmpeg | 3ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ai | .webp | ❌ FAIL | ImageMagick | 132ms | magick: no images for write '-write' '../matrix_test_output/test_converted.webp' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .avif | ❌ FAIL | ImageMagick | 65ms | magick: no images for write '-write' '../matrix_test_output/test_converted.avif' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .png | ❌ FAIL | ImageMagick | 64ms | magick: no images for write '-write' '../matrix_test_output/test_converted.png' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .jpg | ❌ FAIL | ImageMagick | 63ms | magick: no images found for operation `-flatten' at CLI arg 4 @ error/operation.c/CLIOption/5456. |
| .ai | .ico | ❌ FAIL | ImageMagick | 62ms | magick: no images for write '-write' '../matrix_test_output/test_converted.ico' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .bmp | ❌ FAIL | ImageMagick | 67ms | magick: no images for write '-write' '../matrix_test_output/test_converted.bmp' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .tiff | ❌ FAIL | ImageMagick | 67ms | magick: no images for write '-write' '../matrix_test_output/test_converted.tiff' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .tga | ❌ FAIL | ImageMagick | 66ms | magick: no images for write '-write' '../matrix_test_output/test_converted.tga' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .ai | .pdf | ❌ FAIL | ImageMagick | 64ms | magick: no images for write '-write' '../matrix_test_output/test_converted.pdf' at CLI arg 2 @ error/operation.c/CLINoImageOperator/4952. |
| .aiff | .mp3 | ❌ FAIL | FFmpeg | 8ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .aac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .ogg | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .m4a | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .aiff | .opus | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .mp4 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .webm | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .mkv | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .mov | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .mp3 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .avi | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .avif | .webp | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .png | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .jpg | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .ico | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .bmp | ❌ FAIL | ImageMagick | 17ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .tiff | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .tga | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .avif | .pdf | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .bib | .json | ✅ OK | Rust-Native | 7ms |  |
| .bmp | .webp | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .avif | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .png | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .jpg | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .ico | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .tiff | ❌ FAIL | ImageMagick | 18ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .tga | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .bmp | .pdf | ❌ FAIL | ImageMagick | 16ms | magick: improper image header `../test_files/test.bmp' @ error/bmp.c/ReadBMPImage/660. |
| .csv | .json | ✅ OK | Rust-Native | 3ms |  |
| .csv | .yaml | ✅ OK | Rust-Native | 1ms |  |
| .csv | .toml | ❌ FAIL | Rust-Native | 1ms | TOML serialization error: unsupported rust type |
| .csv | .xml | ✅ OK | Rust-Native | 0ms |  |
| .csv | .xlsx | ✅ OK | Rust-Native | 5ms |  |
| .csv | .sql | ✅ OK | Rust-Native | 0ms |  |
| .db | .json | ✅ OK | Rust-Native | 1ms |  |
| .directory | .zip | ✅ OK | Rust-Native | 0ms |  |
| .doc | .pdf | ❌ FAIL | Pandoc | 278ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .doc | .docx | ✅ OK | Pandoc | 54ms |  |
| .doc | .md | ✅ OK | Pandoc | 23ms |  |
| .doc | .html | ✅ OK | Pandoc | 58ms |  |
| .doc | .txt | ✅ OK | Pandoc | 23ms |  |
| .doc | .rtf | ✅ OK | Pandoc | 23ms |  |
| .doc | .epub | ✅ OK | Pandoc | 45ms |  |
| .doc | .odt | ✅ OK | Pandoc | 35ms |  |
| .docx | .pdf | ❌ FAIL | Pandoc | 56ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .docx | .md | ✅ OK | Pandoc | 23ms |  |
| .docx | .html | ✅ OK | Pandoc | 58ms |  |
| .docx | .txt | ✅ OK | Pandoc | 23ms |  |
| .docx | .rtf | ✅ OK | Pandoc | 23ms |  |
| .docx | .epub | ✅ OK | Pandoc | 44ms |  |
| .docx | .odt | ✅ OK | Pandoc | 46ms |  |
| .eps | .webp | ✅ OK | ImageMagick | 191ms |  |
| .eps | .avif | ✅ OK | ImageMagick | 152ms |  |
| .eps | .png | ✅ OK | ImageMagick | 185ms |  |
| .eps | .jpg | ✅ OK | ImageMagick | 166ms |  |
| .eps | .ico | ❌ FAIL | ImageMagick | 156ms | magick: width or height exceeds limit `../matrix_test_output/test_converted.ico' @ error/icon.c/WriteICONImage/1101. |
| .eps | .bmp | ✅ OK | ImageMagick | 151ms |  |
| .eps | .tiff | ✅ OK | ImageMagick | 158ms |  |
| .eps | .tga | ✅ OK | ImageMagick | 168ms |  |
| .eps | .pdf | ✅ OK | ImageMagick | 162ms |  |
| .epub | .pdf | ❌ FAIL | Pandoc | 58ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .epub | .docx | ✅ OK | Pandoc | 55ms |  |
| .epub | .md | ✅ OK | Pandoc | 24ms |  |
| .epub | .html | ✅ OK | Pandoc | 70ms |  |
| .epub | .txt | ✅ OK | Pandoc | 25ms |  |
| .epub | .rtf | ✅ OK | Pandoc | 23ms |  |
| .epub | .odt | ✅ OK | Pandoc | 37ms |  |
| .flac | .mp3 | ❌ FAIL | FFmpeg | 9ms | This file seems to have no tracks that can be converted to MP3. For example, you cannot extract audio from a silent video. |
| .flac | .wav | ❌ FAIL | FFmpeg | 5ms | This file seems to have no tracks that can be converted to WAV. For example, you cannot extract audio from a silent video. |
| .flac | .aac | ❌ FAIL | FFmpeg | 6ms | This file seems to have no tracks that can be converted to AAC. For example, you cannot extract audio from a silent video. |
| .flac | .ogg | ❌ FAIL | FFmpeg | 6ms | This file seems to have no tracks that can be converted to OGG. For example, you cannot extract audio from a silent video. |
| .flac | .m4a | ❌ FAIL | FFmpeg | 6ms | This file seems to have no tracks that can be converted to M4A. For example, you cannot extract audio from a silent video. |
| .flac | .opus | ❌ FAIL | FFmpeg | 4ms | This file seems to have no tracks that can be converted to OPUS. For example, you cannot extract audio from a silent video. |
| .flv | .mp4 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .webm | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .mkv | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .mov | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .avi | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .gif | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .mp3 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .wav | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .flv | .flac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .gz | .zip | ✅ OK | Rust-Native | 1ms |  |
| .heic | .webp | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .avif | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .png | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .jpg | ❌ FAIL | ImageMagick | 20ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .ico | ❌ FAIL | ImageMagick | 20ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .bmp | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .tiff | ❌ FAIL | ImageMagick | 19ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .tga | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .heic | .pdf | ❌ FAIL | ImageMagick | 18ms | ImageMagick does not support decoding this specific format on your system. |
| .html | .pdf | ❌ FAIL | Pandoc | 67ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .html | .docx | ✅ OK | Pandoc | 55ms |  |
| .html | .md | ✅ OK | Pandoc | 23ms |  |
| .html | .txt | ✅ OK | Pandoc | 24ms |  |
| .html | .rtf | ✅ OK | Pandoc | 23ms |  |
| .html | .epub | ✅ OK | Pandoc | 46ms |  |
| .html | .odt | ✅ OK | Pandoc | 35ms |  |
| .ico | .webp | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .avif | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .png | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .jpg | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .bmp | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .tiff | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .tga | ❌ FAIL | ImageMagick | 16ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ico | .pdf | ❌ FAIL | ImageMagick | 16ms | magick: improper image header `../test_files/test.ico' @ error/icon.c/ReadICONImage/349. |
| .ics | .json | ✅ OK | Rust-Native | 0ms |  |
| .jpeg | .webp | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .avif | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .png | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .jpg | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .ico | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .bmp | ❌ FAIL | ImageMagick | 16ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .tiff | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .tga | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpeg | .pdf | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpeg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .webp | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .avif | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .png | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .ico | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .bmp | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .tiff | ❌ FAIL | ImageMagick | 19ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .tga | ❌ FAIL | ImageMagick | 16ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .jpg | .pdf | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.jpg' @ error/jpeg.c/ReadOneJPEGImage/1546. |
| .json | .csv | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .json | .yaml | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .json | .toml | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .json | .xml | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .json | .xlsx | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .json | .sql | ❌ FAIL | Rust-Native | 0ms | JSON parse error: expected ident at line 1 column 2 |
| .log | .json | ✅ OK | Rust-Native | 8ms |  |
| .m4a | .mp3 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .m4a | .wav | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .m4a | .flac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .m4a | .aac | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .m4a | .ogg | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .m4a | .opus | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .mp4 | ❌ FAIL | FFmpeg | 513ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .webm | ❌ FAIL | FFmpeg | 509ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .mkv | ❌ FAIL | FFmpeg | 506ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .mov | ❌ FAIL | FFmpeg | 511ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .avi | ❌ FAIL | FFmpeg | 505ms | The input file appears to be corrupted or in an unsupported format. |
| .m4v | .gif | ❌ FAIL | FFmpeg | 6ms | This file seems to have no tracks that can be converted to GIF. For example, you cannot extract audio from a silent video. |
| .m4v | .mp3 | ❌ FAIL | FFmpeg | 6ms | This file seems to have no tracks that can be converted to MP3. For example, you cannot extract audio from a silent video. |
| .m4v | .wav | ❌ FAIL | FFmpeg | 5ms | This file seems to have no tracks that can be converted to WAV. For example, you cannot extract audio from a silent video. |
| .m4v | .flac | ❌ FAIL | FFmpeg | 7ms | This file seems to have no tracks that can be converted to FLAC. For example, you cannot extract audio from a silent video. |
| .md | .pdf | ❌ FAIL | Pandoc | 69ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .md | .docx | ✅ OK | Pandoc | 55ms |  |
| .md | .html | ✅ OK | Pandoc | 57ms |  |
| .md | .txt | ✅ OK | Pandoc | 24ms |  |
| .md | .rtf | ✅ OK | Pandoc | 23ms |  |
| .md | .epub | ✅ OK | Pandoc | 46ms |  |
| .md | .odt | ✅ OK | Pandoc | 45ms |  |
| .mkv | .mp4 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .webm | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .mov | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .avi | ❌ FAIL | FFmpeg | 8ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .wav | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mkv | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .mp4 | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .webm | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .mkv | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .avi | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .gif | ❌ FAIL | FFmpeg | 8ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .mp3 | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mov | .flac | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .flac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .aac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .ogg | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .m4a | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .mp3 | .opus | ❌ FAIL | FFmpeg | 8ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .webm | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .mkv | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .mov | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .avi | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .gif | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .mp3 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .mp4 | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .odt | .pdf | ❌ FAIL | Pandoc | 66ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .odt | .docx | ✅ OK | Pandoc | 56ms |  |
| .odt | .md | ✅ OK | Pandoc | 24ms |  |
| .odt | .html | ✅ OK | Pandoc | 59ms |  |
| .odt | .txt | ✅ OK | Pandoc | 25ms |  |
| .odt | .rtf | ✅ OK | Pandoc | 23ms |  |
| .odt | .epub | ✅ OK | Pandoc | 45ms |  |
| .ogg | .mp3 | ❌ FAIL | FFmpeg | 5ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogg | .wav | ❌ FAIL | FFmpeg | 6ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogg | .flac | ❌ FAIL | FFmpeg | 3ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogg | .aac | ❌ FAIL | FFmpeg | 8ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogg | .m4a | ❌ FAIL | FFmpeg | 3ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogg | .opus | ❌ FAIL | FFmpeg | 5ms | An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible. |
| .ogv | .mp4 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .webm | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .mkv | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .mov | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .avi | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .wav | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .ogv | .flac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .wav | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .flac | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .aac | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .ogg | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .opus | .m4a | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .pdf | .docx | ❌ FAIL | Poppler (Poppler-utils) | 25ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .pdf | .md | ❌ FAIL | Poppler (Poppler-utils) | 23ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .pdf | .html | ❌ FAIL | Poppler (Poppler-utils) | 25ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .pdf | .txt | ❌ FAIL | Poppler (Poppler-utils) | 22ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table |
| .pdf | .rtf | ❌ FAIL | Poppler (Poppler-utils) | 25ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .pdf | .epub | ❌ FAIL | Poppler (Poppler-utils) | 23ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .pdf | .odt | ❌ FAIL | Poppler (Poppler-utils) | 25ms | Syntax Warning: May not be a PDF file (continuing anyway)

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't find trailer dictionary

Syntax Error: Couldn't read xref table

 |
| .png | .webp | ❌ FAIL | ImageMagick | 20ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .avif | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .jpg | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .ico | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .bmp | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .tiff | ❌ FAIL | ImageMagick | 18ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .tga | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .png | .pdf | ❌ FAIL | ImageMagick | 17ms | magick: insufficient image data in file `../test_files/test.png' @ error/png.c/ReadPNGImage/3962. |
| .psd | .webp | ❌ FAIL | ImageMagick | 20ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .avif | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .png | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .jpg | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .ico | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .bmp | ❌ FAIL | ImageMagick | 16ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .tiff | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .tga | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .psd | .pdf | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.psd' @ error/psd.c/ReadPSDImage/2449. |
| .rar | .zip | ✅ OK | Rust-Native | 1ms |  |
| .rtf | .pdf | ❌ FAIL | Pandoc | 57ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .rtf | .docx | ✅ OK | Pandoc | 56ms |  |
| .rtf | .md | ✅ OK | Pandoc | 23ms |  |
| .rtf | .html | ✅ OK | Pandoc | 59ms |  |
| .rtf | .txt | ✅ OK | Pandoc | 24ms |  |
| .rtf | .epub | ✅ OK | Pandoc | 44ms |  |
| .rtf | .odt | ✅ OK | Pandoc | 34ms |  |
| .sql | .json | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to json |
| .sql | .csv | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to csv |
| .sql | .yaml | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to yaml |
| .sql | .toml | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to toml |
| .sql | .xml | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to xml |
| .sql | .xlsx | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: sql to xlsx |
| .sqlite | .json | ✅ OK | Rust-Native | 0ms |  |
| .svg | .webp | ❌ FAIL | ImageMagick | 32ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .avif | ❌ FAIL | ImageMagick | 26ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .png | ❌ FAIL | ImageMagick | 22ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .jpg | ❌ FAIL | ImageMagick | 25ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .ico | ❌ FAIL | ImageMagick | 23ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .bmp | ❌ FAIL | ImageMagick | 24ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .tiff | ❌ FAIL | ImageMagick | 24ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .tga | ❌ FAIL | ImageMagick | 25ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .svg | .pdf | ❌ FAIL | ImageMagick | 24ms | magick: unable to read image data `../test_files/test.svg' @ error/svg.c/RenderRSVGImage/463. |
| .tar | .zip | ✅ OK | Rust-Native | 2ms |  |
| .tga | .webp | ❌ FAIL | ImageMagick | 18ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .avif | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .png | ❌ FAIL | ImageMagick | 18ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .jpg | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .ico | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .bmp | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .tiff | ❌ FAIL | ImageMagick | 17ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tga | .pdf | ❌ FAIL | ImageMagick | 16ms | magick: improper image header `../test_files/test.tga' @ error/tga.c/ReadTGAImage/221. |
| .tiff | .webp | ❌ FAIL | ImageMagick | 20ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .avif | ❌ FAIL | ImageMagick | 18ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .png | ❌ FAIL | ImageMagick | 18ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .jpg | ❌ FAIL | ImageMagick | 18ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .ico | ❌ FAIL | ImageMagick | 20ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .bmp | ❌ FAIL | ImageMagick | 18ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .tga | ❌ FAIL | ImageMagick | 18ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .tiff | .pdf | ❌ FAIL | ImageMagick | 19ms | magick: Cannot read TIFF header. `../test_files/test.tiff' @ error/tiff.c/TIFFErrors/564. |
| .toml | .json | ❌ FAIL | Rust-Native | 2ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .toml | .csv | ❌ FAIL | Rust-Native | 1ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .toml | .yaml | ❌ FAIL | Rust-Native | 1ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .toml | .xml | ❌ FAIL | Rust-Native | 1ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .toml | .xlsx | ❌ FAIL | Rust-Native | 1ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .toml | .sql | ❌ FAIL | Rust-Native | 1ms | TOML parse error: TOML parse error at line 1, column 6
  \|
1 \| test data
  \|      ^
expected `.`, `=`
 |
| .ts | .mp4 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .webm | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .mkv | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .mov | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .avi | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .gif | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .mp3 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .wav | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .ts | .flac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .txt | .pdf | ❌ FAIL | Pandoc | 58ms | PDF generation requires a LaTeX installation (like MiKTeX or TeX Live). |
| .txt | .docx | ✅ OK | Pandoc | 56ms |  |
| .txt | .md | ✅ OK | Pandoc | 23ms |  |
| .txt | .html | ✅ OK | Pandoc | 61ms |  |
| .txt | .rtf | ✅ OK | Pandoc | 24ms |  |
| .txt | .epub | ✅ OK | Pandoc | 45ms |  |
| .txt | .odt | ✅ OK | Pandoc | 35ms |  |
| .vob | .mp4 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .webm | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .mkv | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .mov | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .avi | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .wav | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .vob | .flac | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .aac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .ogg | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .m4a | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .wav | .opus | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .mp4 | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .mkv | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .mov | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .avi | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .gif | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .mp3 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .wav | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .webm | .flac | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .webp | .avif | ❌ FAIL | ImageMagick | 17ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .png | ❌ FAIL | ImageMagick | 17ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .jpg | ❌ FAIL | ImageMagick | 16ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .ico | ❌ FAIL | ImageMagick | 16ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .bmp | ❌ FAIL | ImageMagick | 18ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .tiff | ❌ FAIL | ImageMagick | 20ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .tga | ❌ FAIL | ImageMagick | 20ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .webp | .pdf | ❌ FAIL | ImageMagick | 16ms | magick: no images found for operation `-coalesce' at CLI arg 1 @ error/operation.c/CLIOption/5456. |
| .wma | .mp3 | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .wav | ❌ FAIL | FFmpeg | 3ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .flac | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .aac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .ogg | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .m4a | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wma | .opus | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .mp4 | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .webm | ❌ FAIL | FFmpeg | 7ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .mkv | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .mov | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .avi | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .gif | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .mp3 | ❌ FAIL | FFmpeg | 5ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .wav | ❌ FAIL | FFmpeg | 6ms | The input file appears to be corrupted or in an unsupported format. |
| .wmv | .flac | ❌ FAIL | FFmpeg | 4ms | The input file appears to be corrupted or in an unsupported format. |
| .xls | .json | ❌ FAIL | Rust-Native | 1ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .csv | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .yaml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .toml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .xml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .xlsx | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xls | .sql | ❌ FAIL | Rust-Native | 1ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .json | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .csv | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .yaml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .toml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .xml | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xlsx | .sql | ❌ FAIL | Rust-Native | 0ms | Excel open error: Zip error: invalid Zip archive: Could not find EOCD |
| .xml | .json | ✅ OK | Rust-Native | 1ms |  |
| .xml | .csv | ❌ FAIL | Rust-Native | 0ms | Input must be an array of objects to convert to CSV |
| .xml | .yaml | ✅ OK | Rust-Native | 0ms |  |
| .xml | .toml | ✅ OK | Rust-Native | 0ms |  |
| .xml | .xlsx | ✅ OK | Rust-Native | 6ms |  |
| .xml | .sql | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: xml to sql |
| .yaml | .json | ✅ OK | Rust-Native | 1ms |  |
| .yaml | .csv | ❌ FAIL | Rust-Native | 0ms | Input must be an array of objects to convert to CSV |
| .yaml | .toml | ❌ FAIL | Rust-Native | 0ms | TOML serialization error: unsupported rust type |
| .yaml | .xml | ✅ OK | Rust-Native | 0ms |  |
| .yaml | .xlsx | ✅ OK | Rust-Native | 6ms |  |
| .yaml | .sql | ❌ FAIL | Rust-Native | 0ms | Unsupported data conversion: yaml to sql |
| .zip | .folder | ✅ OK | Rust-Native | 0ms |  |
