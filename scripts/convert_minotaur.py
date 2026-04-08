"""
Blender script: Convert Mixamo minotaur FBX to GLB with textures from old minotaur.
Run headless: blender --background --python scripts/convert_minotaur.py
"""
import bpy
import os
import sys

PROJECT = r"C:\Users\kglazier\source\projects\Tower Native"
OLD_GLB = os.path.join(PROJECT, "assets", "models", "enemies", "minotaur.glb")
NEW_FBX = os.path.join(PROJECT, "assets", "models", "enemies", "minotaur-for-mixamo.fbx")
# Use the one from Downloads if it's different/newer
DOWNLOADS_FBX = os.path.join(os.path.expanduser("~"), "Downloads", "minotaur-for-mixamo.fbx")
OUTPUT_GLB = os.path.join(PROJECT, "assets", "models", "enemies", "minotaur-mixamo.glb")

# Use Downloads FBX if it exists, otherwise use the one already in assets
fbx_path = DOWNLOADS_FBX if os.path.exists(DOWNLOADS_FBX) else NEW_FBX
print(f"Using FBX: {fbx_path}")

# ── Step 1: Clear scene ──
bpy.ops.wm.read_factory_settings(use_empty=True)

# ── Step 2: Import old minotaur GLB to extract materials/textures ──
print("Importing old minotaur GLB for textures...")
bpy.ops.import_scene.gltf(filepath=OLD_GLB)

# Collect materials from old model
old_materials = {}
old_meshes = []
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        old_meshes.append(obj.name)
        for slot in obj.material_slots:
            if slot.material:
                mat = slot.material
                old_materials[mat.name] = mat
                print(f"  Found material: {mat.name}")
                # Inspect texture nodes
                if mat.use_nodes:
                    for node in mat.node_tree.nodes:
                        if node.type == 'TEX_IMAGE' and node.image:
                            print(f"    Texture: {node.image.name} ({node.image.filepath})")

print(f"Collected {len(old_materials)} materials from old model")
print(f"Old mesh objects: {old_meshes}")

# Store old materials separately before clearing
# We need to keep them alive by incrementing users
for mat in old_materials.values():
    mat.use_fake_user = True

# Also preserve all images
for img in bpy.data.images:
    img.use_fake_user = True

# ── Step 3: Delete old mesh objects (keep materials/textures in memory) ──
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete()

# ── Step 4: Import Mixamo FBX ──
print(f"\nImporting Mixamo FBX: {fbx_path}")
bpy.ops.import_scene.fbx(filepath=fbx_path)

# List what we imported
print("\nImported objects:")
new_meshes = []
for obj in bpy.data.objects:
    print(f"  {obj.name} ({obj.type})")
    if obj.type == 'MESH':
        new_meshes.append(obj)
        print(f"    Materials: {[s.material.name if s.material else 'None' for s in obj.material_slots]}")

# ── Step 5: Transfer materials ──
print("\nTransferring materials...")

# Strategy: Try to match by name first, then assign the first available old material
for obj in new_meshes:
    for i, slot in enumerate(obj.material_slots):
        current_mat_name = slot.material.name if slot.material else "None"

        # Try exact name match
        matched = False
        for old_name, old_mat in old_materials.items():
            if old_name.lower() in current_mat_name.lower() or current_mat_name.lower() in old_name.lower():
                print(f"  Matched '{current_mat_name}' -> '{old_name}' (name match)")
                slot.material = old_mat
                matched = True
                break

        if not matched and old_materials:
            # If no name match, try to find a material with textures
            for old_name, old_mat in old_materials.items():
                has_texture = False
                if old_mat.use_nodes:
                    for node in old_mat.node_tree.nodes:
                        if node.type == 'TEX_IMAGE' and node.image:
                            has_texture = True
                            break
                if has_texture:
                    print(f"  Assigned '{old_name}' to slot {i} (has textures)")
                    slot.material = old_mat
                    matched = True
                    break

            if not matched:
                # Just use the first old material
                first_mat = list(old_materials.values())[0]
                print(f"  Fallback: assigned '{first_mat.name}' to slot {i}")
                slot.material = first_mat

# ── Step 6: Export as GLB ──
print(f"\nExporting to: {OUTPUT_GLB}")

# Select all objects for export
bpy.ops.object.select_all(action='SELECT')

bpy.ops.export_scene.gltf(
    filepath=OUTPUT_GLB,
    export_format='GLB',
    export_animations=True,
    export_skins=True,
    export_materials='EXPORT',
    export_texcoords=True,
    export_normals=True,
    export_image_format='AUTO',
)

final_size = os.path.getsize(OUTPUT_GLB)
print(f"\nDone! Output: {OUTPUT_GLB} ({final_size:,} bytes)")
print("Materials in output:")
for mat in bpy.data.materials:
    if mat.users > 0:
        print(f"  {mat.name} (users: {mat.users})")
