"""
Re-export minotaur-mixamo.glb: remove Icosphere, keep only Armature + Object_101 with textures.
"""
import bpy
import os

GLB = r"C:\Users\kglazier\source\projects\Tower Native\assets\models\enemies\minotaur-mixamo.glb"
OUTPUT = GLB  # overwrite

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=GLB)

# Delete Icosphere and any other stray objects
for obj in list(bpy.data.objects):
    if obj.type == 'MESH' and obj.parent is None:
        print(f"Deleting stray mesh: {obj.name}")
        bpy.data.objects.remove(obj, do_unlink=True)

# Verify what remains
print("\nRemaining objects:")
for obj in bpy.data.objects:
    print(f"  {obj.name} ({obj.type})")

# Also clean up orphan mesh data
bpy.ops.outliner.orphans_purge(do_local_ids=True, do_linked_ids=True, do_recursive=True)

bpy.ops.object.select_all(action='SELECT')
bpy.ops.export_scene.gltf(
    filepath=OUTPUT,
    export_format='GLB',
    export_animations=True,
    export_skins=True,
    export_materials='EXPORT',
    export_texcoords=True,
    export_normals=True,
    export_image_format='AUTO',
)

print(f"\nDone! {os.path.getsize(OUTPUT):,} bytes")
