import re

with open(".github/workflows/publish.yml", "r") as f:
    content = f.read()

# Fix 'on: push:'
content = re.sub(
    r'on:\n  push:\n    branches:\n      - main',
    r"on:\n  push:\n    tags:\n      - 'v*'",
    content
)

# Use npm ci
content = content.replace("npm install", "npm ci")

# macOS matrix changes
content = re.sub(
    r'- platform: "macos-latest"\n\s+args: "--target universal-apple-darwin"',
    r'- platform: "macos-latest"\n            args: ""\n          - platform: "macos-13"\n            args: ""',
    content
)

# macOS script changes
macos_script = '''
            "macos-latest" | "macos-13")
              ARCH=$(rustc -vV | grep host | awk '{print $2}')
              brew install ffmpeg poppler pandoc imagemagick
              safe_cp "$(brew --prefix ffmpeg)/bin/ffmpeg" "ffmpeg-$ARCH"
              safe_cp "$(brew --prefix ffmpeg)/bin/ffprobe" "ffprobe-$ARCH"
              safe_cp "$(brew --prefix pandoc)/bin/pandoc" "pandoc-$ARCH"
              safe_cp "$(brew --prefix imagemagick)/bin/magick" "magick-$ARCH"
              safe_cp "$(brew --prefix poppler)/bin/pdftotext" "pdftotext-$ARCH"
              safe_cp "$(brew --prefix poppler)/bin/pdftohtml" "pdftohtml-$ARCH"
              ;;
'''
content = re.sub(
    r'"macos-latest"\).*?;;',
    macos_script.strip(),
    content,
    flags=re.DOTALL
)

# Fix validate sidecars
validate_macos = '''
            "macos-latest" | "macos-13")
              ARCH=$(rustc -vV | grep host | awk '{print $2}')
              SUFFIXES=("-$ARCH")
              ;;
'''
content = re.sub(
    r'"macos-latest"\).*?;;',
    validate_macos.strip(),
    content,
    flags=re.DOTALL
)

with open(".github/workflows/publish.yml", "w") as f:
    f.write(content)
