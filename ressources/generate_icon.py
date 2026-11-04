import math
from PIL import Image, ImageDraw, ImageFilter, ImageFont
import os

def create_texteur_icon():
    # Render at high resolution 1024x1024 for pristine downsampling
    size = 1024
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    
    # 1. Background Squircle (Rounded Square)
    margin = 48
    radius = 210
    
    # Create mask for squircle
    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(
        [margin, margin, size - margin, size - margin],
        radius=radius,
        fill=255
    )
    
    # Background gradient: deep midnight blue / slate (#0B1120 to #1E293B)
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg)
    for y in range(margin, size - margin):
        ratio = (y - margin) / float(size - 2 * margin)
        # Gradient from (11, 17, 32) to (30, 41, 59)
        r = int(12 + ratio * 22)
        g = int(18 + ratio * 28)
        b = int(32 + ratio * 36)
        bg_draw.line([(margin, y), (size - margin, y)], fill=(r, g, b, 255))
        
    # Top-left radial highlight (soft ambient light)
    highlight = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    hl_draw = ImageDraw.Draw(highlight)
    hl_center = (margin + 200, margin + 200)
    for radius_hl in range(350, 0, -5):
        alpha = int(45 * (1.0 - radius_hl / 350.0))
        hl_draw.ellipse(
            [hl_center[0] - radius_hl, hl_center[1] - radius_hl, hl_center[0] + radius_hl, hl_center[1] + radius_hl],
            fill=(56, 189, 248, alpha)
        )
    bg = Image.alpha_composite(bg, highlight)
    
    # Apply squircle mask to background
    bg.putalpha(mask)
    
    # Subtle emerald glowing border
    border = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    border_draw = ImageDraw.Draw(border)
    border_draw.rounded_rectangle(
        [margin, margin, size - margin, size - margin],
        radius=radius,
        outline=(16, 185, 129, 220),
        width=14
    )
    # Inner border line for depth
    border_draw.rounded_rectangle(
        [margin + 7, margin + 7, size - margin - 7, size - margin - 7],
        radius=radius - 5,
        outline=(52, 211, 153, 120),
        width=4
    )
    bg = Image.alpha_composite(bg, border)
    
    # 2. Main Motif: Elegant Quill / Feather + Bold Stylized 'T'
    # Draw on a separate transparent layer so we can apply drop shadow
    motif = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    m_draw = ImageDraw.Draw(motif)
    
    # --- The Letter 'T' ---
    # Top bar of T: x from 220 to 600, y from 300 to 390
    # Stem of T: x from 360 to 460, y from 390 to 760
    # With beveled rounded corners
    t_color = (248, 250, 252, 255) # Clean snow white
    
    # Horizontal bar of T
    m_draw.rounded_rectangle([230, 310, 620, 400], radius=24, fill=t_color)
    # Vertical stem of T
    m_draw.rounded_rectangle([375, 370, 475, 750], radius=24, fill=t_color)
    
    # Top-right subtle cyan cut/accent on T
    m_draw.rounded_rectangle([240, 320, 610, 350], radius=8, fill=(255, 255, 255, 180))

    # --- The Feather / Plume (Curved Quill) ---
    # Diagonal angle ~ -45 degrees, sweeping from top-right (720, 240) down to (520, 780)
    # Nib touching down near (520, 790) with golden tip
    
    # Path points for outer feather vane (emerald)
    feather_outer = [
        (760, 230), # Tip of the feather
        (795, 320),
        (780, 440),
        (720, 560),
        (650, 670),
        (580, 750), # Feather shaft near nib
        (540, 780), # Nib base
        (560, 730),
        (600, 640),
        (640, 530),
        (665, 410),
        (670, 300),
    ]
    
    # Draw feather body with gradient
    feather_img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    f_draw = ImageDraw.Draw(feather_img)
    f_draw.polygon(feather_outer, fill=(16, 185, 129, 250))
    
    # Inner vane highlight (mint green #34D399)
    feather_inner = [
        (750, 250),
        (775, 330),
        (755, 430),
        (700, 540),
        (640, 630),
        (620, 610),
        (645, 500),
        (660, 390),
        (670, 310),
    ]
    f_draw.polygon(feather_inner, fill=(52, 211, 153, 230))
    
    # Feather cuts / barbs (fine styling lines)
    barbs = [
        ((720, 340), (660, 370)),
        ((740, 410), (670, 440)),
        ((715, 480), (650, 510)),
        ((680, 550), (620, 580)),
        ((640, 620), (590, 650)),
    ]
    for p1, p2 in barbs:
        f_draw.line([p1, p2], fill=(6, 78, 59, 140), width=6)
        
    # Feather Central Shaft (quill spine)
    spine_points = [
        (760, 230),
        (720, 360),
        (670, 490),
        (610, 620),
        (550, 740),
        (525, 800), # tip of nib
    ]
    for i in range(len(spine_points) - 1):
        f_draw.line([spine_points[i], spine_points[i+1]], fill=(254, 240, 138, 240), width=9)
        
    # Golden Calligraphy Nib (La Plume d'Or)
    nib_poly = [
        (555, 735),
        (515, 810), # Nib point
        (505, 805),
        (520, 760),
        (535, 735),
    ]
    f_draw.polygon(nib_poly, fill=(245, 158, 11, 255))
    # Nib slit
    f_draw.line([(515, 810), (535, 755)], fill=(120, 53, 15, 255), width=3)
    
    # Small breather hole in nib
    f_draw.ellipse([532, 752, 538, 758], fill=(120, 53, 15, 255))

    # Sparkle / Precision Star at the tip of the nib
    sparkle_center = (512, 814)
    scx, scy = sparkle_center
    sp_color = (253, 224, 71, 255) # Bright gold
    sp_arm_long = 42
    sp_arm_short = 14
    # 4-pointed star
    star_pts = [
        (scx, scy - sp_arm_long),
        (scx + sp_arm_short, scy - sp_arm_short),
        (scx + sp_arm_long, scy),
        (scx + sp_arm_short, scy + sp_arm_short),
        (scx, scy + sp_arm_long),
        (scx - sp_arm_short, scy + sp_arm_short),
        (scx - sp_arm_long, scy),
        (scx - sp_arm_short, scy - sp_arm_short),
    ]
    f_draw.polygon(star_pts, fill=sp_color)
    f_draw.ellipse([scx - 7, scy - 7, scx + 7, scy + 7], fill=(255, 255, 255, 255))

    # Composite feather onto motif
    motif = Image.alpha_composite(motif, feather_img)
    
    # Drop shadow for the entire motif
    shadow = motif.split()[3].filter(ImageFilter.GaussianBlur(22))
    shadow_img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    shadow_img.paste((0, 0, 0, 160), mask=shadow)
    # Offset shadow down slightly
    shadow_offset = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    shadow_offset.paste(shadow_img, (0, 14))
    
    # Merge background + shadow + motif
    final_img = Image.alpha_composite(bg, shadow_offset)
    final_img = Image.alpha_composite(final_img, motif)
    
    # Export PNG high-res
    out_dir = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur\ressources"
    os.makedirs(out_dir, exist_ok=True)
    
    png_path = os.path.join(out_dir, "texteur.png")
    final_img.resize((256, 256), Image.Resampling.LANCZOS).save(png_path, "PNG")
    print(f"Saved {png_path}")
    
    # Export multi-resolution ICO file (256, 128, 64, 48, 32, 16)
    ico_path = os.path.join(out_dir, "texteur.ico")
    icon_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
    final_img.save(ico_path, format="ICO", sizes=icon_sizes)
    print(f"Saved {ico_path} with sizes {icon_sizes}")
    
    # Also save to parent folder so Desktop shortcut or assets can easily reach it
    parent_ico = r"c:\Users\Bpaquereau\Desktop\Antidote\texteur.ico"
    final_img.save(parent_ico, format="ICO", sizes=icon_sizes)
    print(f"Saved {parent_ico}")

if __name__ == "__main__":
    create_texteur_icon()
