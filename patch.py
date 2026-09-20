with open("src-tauri/src/lib.rs", "r") as f:
    content = f.read()

import re
content = re.sub(
    r'let dest_path = Path::new\(&dest\);.*?(?=if let Some\(parent\))',
    r'''if dest.contains("..") || dest.contains("\\0") {
            return Err("Path traversal or null bytes not allowed".to_string());
        }
        let dest_path = Path::new(&dest);
        if dest_path.exists() {
            return Err("Destination file already exists".to_string());
        }
        ''',
    content, flags=re.DOTALL
)

with open("src-tauri/src/lib.rs", "w") as f:
    f.write(content)
