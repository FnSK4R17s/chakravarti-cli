---
name: python-scripts
description: Run Python scripts in an isolated virtual environment. Use when you need to execute Python code for image manipulation, data processing, or any scripting task.
---

# Python Scripts Skill

Run Python scripts in an isolated virtual environment that is created and cleaned up automatically.

## When to Use

- Image manipulation (resize, crop, transparency, format conversion)
- Data processing and file manipulation
- Quick scripting tasks that need Python libraries
- Any task requiring PIL/Pillow, numpy, or other common Python packages

## Execution Pattern

Always use this pattern to run Python scripts:

```bash
# Create venv, install dependencies, run script, cleanup
python3 -m venv /apps/chakravarti-cli/.tmp_venv && \
source /apps/chakravarti-cli/.tmp_venv/bin/activate && \
pip install -q <packages> && \
python3 << 'EOF'
# Your Python code here
EOF
rm -rf /apps/chakravarti-cli/.tmp_venv
```

### Example: Image Manipulation

```bash
python3 -m venv /apps/chakravarti-cli/.tmp_venv && \
source /apps/chakravarti-cli/.tmp_venv/bin/activate && \
pip install -q pillow numpy && \
python3 << 'EOF'
from PIL import Image
import numpy as np

# Load image
img = Image.open('/path/to/image.png').convert('RGBA')

# Manipulate...
# img.resize((128, 128), Image.Resampling.LANCZOS)

# Save
img.save('/path/to/output.png')
print('Done!')
EOF
rm -rf /apps/chakravarti-cli/.tmp_venv
```

### Example: Make Corners Transparent

```bash
python3 -m venv /apps/chakravarti-cli/.tmp_venv && \
source /apps/chakravarti-cli/.tmp_venv/bin/activate && \
pip install -q pillow numpy && \
python3 << 'EOF'
from PIL import Image
import numpy as np

img = Image.open('/path/to/logo.png').convert('RGBA')
data = np.array(img)
height, width = data.shape[:2]

# Corner radius as percentage of image size
corner_radius = int(min(width, height) * 0.12)

for y in range(height):
    for x in range(width):
        in_corner = False
        
        # Top-left
        if x < corner_radius and y < corner_radius:
            if ((corner_radius - x)**2 + (corner_radius - y)**2)**0.5 > corner_radius:
                in_corner = True
        # Top-right
        if x >= width - corner_radius and y < corner_radius:
            if ((x - (width - corner_radius - 1))**2 + (corner_radius - y)**2)**0.5 > corner_radius:
                in_corner = True
        # Bottom-left
        if x < corner_radius and y >= height - corner_radius:
            if ((corner_radius - x)**2 + (y - (height - corner_radius - 1))**2)**0.5 > corner_radius:
                in_corner = True
        # Bottom-right
        if x >= width - corner_radius and y >= height - corner_radius:
            if ((x - (width - corner_radius - 1))**2 + (y - (height - corner_radius - 1))**2)**0.5 > corner_radius:
                in_corner = True
        
        if in_corner:
            data[y, x, 3] = 0  # Set alpha to 0

Image.fromarray(data).save('/path/to/output.png')
print('Corners made transparent!')
EOF
rm -rf /apps/chakravarti-cli/.tmp_venv
```

### Example: Resize for App Icons

```bash
python3 -m venv /apps/chakravarti-cli/.tmp_venv && \
source /apps/chakravarti-cli/.tmp_venv/bin/activate && \
pip install -q pillow && \
python3 << 'EOF'
from PIL import Image

logo = Image.open('/apps/chakravarti-cli/logo.png').convert('RGBA')

sizes = [
    (512, 'icon.png'),
    (256, '128x128@2x.png'),
    (128, '128x128.png'),
    (32, '32x32.png'),
]

icons_dir = '/apps/chakravarti-cli/crates/ckrv-tauri/icons/'

for size, filename in sizes:
    resized = logo.resize((size, size), Image.Resampling.LANCZOS)
    resized.save(icons_dir + filename)
    print(f'Created {filename} ({size}x{size})')
EOF
rm -rf /apps/chakravarti-cli/.tmp_venv
```

## Common Packages

- `pillow` - Image manipulation
- `numpy` - Numerical operations, array manipulation
- `requests` - HTTP requests
- `pyyaml` - YAML parsing
- `toml` - TOML parsing

## Important Notes

1. **Always cleanup**: Include `rm -rf /apps/chakravarti-cli/.tmp_venv` at the end
2. **Use quiet install**: Use `pip install -q` to reduce output noise
3. **Heredoc for scripts**: Use `<< 'EOF'` syntax for multi-line Python scripts
4. **RGBA for transparency**: Always use `.convert('RGBA')` when working with transparency
