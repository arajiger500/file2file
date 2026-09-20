with open(".github/workflows/publish.yml", "r") as f:
    content = f.read()

smoke_test = """
      - name: smoke test sidecars
        shell: bash
        run: |
          for bin in src-tauri/binaries/*; do
            if [ -f "$bin" ]; then
              echo "Testing $bin"
              chmod +x "$bin"
              "$bin" -version || "$bin" --version || echo "Failed to get version for $bin"
            fi
          done

"""

content = content.replace("      - name: build frontend", smoke_test + "      - name: build frontend")

with open(".github/workflows/publish.yml", "w") as f:
    f.write(content)
