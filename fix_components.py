with open("src/components/FormatSelectionPage.tsx", "r") as f:
    content = f.read()

import re

if 'import { useState' not in content:
    content = 'import { useState, useMemo } from "react";\n' + content

content = content.replace("c =>", "(c: string) =>")
content = content.replace('title="Engine missing"', "")

with open("src/components/FormatSelectionPage.tsx", "w") as f:
    f.write(content)

with open("src/components/QuickConverters.tsx", "r") as f:
    content = f.read()

content = content.replace('title="Required engine missing"', "")

with open("src/components/QuickConverters.tsx", "w") as f:
    f.write(content)
