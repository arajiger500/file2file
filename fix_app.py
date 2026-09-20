with open("src/App.tsx", "r") as f:
    content = f.read()

import re
content = re.sub(
    r'isTauri\(\)',
    r'isTauri()',
    content
)

# Wait, the error was: src/App.tsx:235:46 - error TS2304: Cannot find name 'isTauri'.
# We need to import isTauri in src/App.tsx
content = 'import { isTauri } from "./services/api";\n' + content

with open("src/App.tsx", "w") as f:
    f.write(content)

with open("src/components/FormatSelectionPage.tsx", "r") as f:
    content = f.read()

if 'import { useState' not in content:
    content = content.replace('import React', 'import React, { useState, useMemo }')

with open("src/components/FormatSelectionPage.tsx", "w") as f:
    f.write(content)
