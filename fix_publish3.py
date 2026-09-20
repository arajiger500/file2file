import re

with open(".github/workflows/publish.yml", "r") as f:
    content = f.read()

content = content.replace('''
            "macos-latest" | "macos-13")
              ARCH=$(rustc -vV | grep host | awk '{print $2}')
              SUFFIXES=("-$ARCH")
              ;;'''.strip(), '''
            "macos-latest" | "macos-13")
              ARCH=$(rustc -vV | grep host | awk '{print $2}')
              brew install ffmpeg poppler pandoc imagemagick
              safe_cp "$(brew --prefix ffmpeg)/bin/ffmpeg" "ffmpeg-$ARCH"
              safe_cp "$(brew --prefix ffmpeg)/bin/ffprobe" "ffprobe-$ARCH"
              safe_cp "$(brew --prefix pandoc)/bin/pandoc" "pandoc-$ARCH"
              safe_cp "$(brew --prefix imagemagick)/bin/magick" "magick-$ARCH"
              safe_cp "$(brew --prefix poppler)/bin/pdftotext" "pdftotext-$ARCH"
              safe_cp "$(brew --prefix poppler)/bin/pdftohtml" "pdftohtml-$ARCH"
              ;;'''.strip(), 1)

with open(".github/workflows/publish.yml", "w") as f:
    f.write(content)
