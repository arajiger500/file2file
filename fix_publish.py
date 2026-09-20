import re

with open(".github/workflows/publish.yml", "r") as f:
    content = f.read()

# Fix validate sidecars
validate_macos = '''
            "macos-latest" | "macos-13")
              ARCH=$(rustc -vV | grep host | awk '{print $2}')
              SUFFIXES=("-$ARCH")
              ;;
'''
content = re.sub(
    r'"macos-latest" \| "macos-13"\).*?;;',
    validate_macos.strip(),
    content,
    flags=re.DOTALL
)

with open(".github/workflows/publish.yml", "w") as f:
    f.write(content)
