import os
from PIL import Image, ImageDraw

def bootstrap_nexus_icons():
    # 1. Establish structural canvas base boundaries
    canvas_size = 512
    img = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 2. Render Base Frame Geometry: Dark slate squircle with metallic teal accents
    # Enforces the unburdened Mercurial structural framework
    padding = 24
    corner_radius = 110
    draw.rounded_rectangle(
        [padding, padding, canvas_size - padding, canvas_size - padding],
        radius=corner_radius,
        fill=(18, 18, 18, 255),
        outline=(0, 77, 77, 255), # Metallic Teal border
        width=16
    )

    # 3. Draw Geometric Object Layout: Dual Emerald Green Candlestick Nodes
    # Left Bearish Adjustment Bar
    draw.rectangle([140, 180, 210, 380], fill=(0, 90, 54, 255)) # Deep Emerald Green
    draw.line([175, 120, 175, 180], fill=(0, 90, 54, 255), width=8)
    draw.line([175, 380, 175, 420], fill=(0, 90, 54, 255), width=8)

    # Right Bullish Adjustment Breakout Bar
    draw.rectangle([302, 100, 372, 300], fill=(0, 255, 157, 255)) # High-visibility Vivid Mint
    draw.line([337, 50, 337, 100], fill=(0, 255, 157, 255), width=8)
    draw.line([337, 300, 337, 360], fill=(0, 255, 157, 255), width=8)

    # 4. Splicing Central Interconnection Vector: Ascending Trend Line Tracker
    draw.line([100, 400, 240, 260, 420, 80], fill=(0, 255, 157, 255), width=12, joint="round")

    # 5. Compile and export matching Tauri resource directory mappings
    target_dir = os.path.join("icons")
    os.makedirs(target_dir, exist_ok=True)

    # Export structured sizes cleanly
    img_32 = img.resize((32, 32), Image.Resampling.LANCZOS)
    img_32.save(os.path.join(target_dir, "32x32.png"))

    img_128 = img.resize((128, 128), Image.Resampling.LANCZOS)
    img_128.save(os.path.join(target_dir, "128x128.png"))

    # Generate multi-layered unified windows executable icon matrix
    img.save(
        os.path.join(target_dir, "icon.ico"),
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )
    print("Nexus Trading Core platform asset icons successfully bootstrapped.")

if __name__ == "__main__":
    bootstrap_nexus_icons()
