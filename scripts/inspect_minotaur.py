"""Inspect minotaur-mixamo.glb bone names and structure."""
import bpy
import os

GLB = r"C:\Users\kglazier\source\projects\Tower Native\assets\models\enemies\minotaur-mixamo.glb"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=GLB)

print("\n=== OBJECTS ===")
for obj in bpy.data.objects:
    print(f"  {obj.name} (type={obj.type}, parent={obj.parent})")
    if obj.type == 'MESH':
        print(f"    Verts: {len(obj.data.vertices)}, Materials: {[s.material.name if s.material else 'None' for s in obj.material_slots]}")
        # Check vertex groups (bone weights)
        print(f"    Vertex groups: {len(obj.vertex_groups)}")
        if obj.vertex_groups:
            for vg in obj.vertex_groups[:10]:
                print(f"      {vg.name}")
            if len(obj.vertex_groups) > 10:
                print(f"      ... and {len(obj.vertex_groups) - 10} more")
    if obj.type == 'ARMATURE':
        print(f"    Bones ({len(obj.data.bones)}):")
        for bone in obj.data.bones:
            print(f"      {bone.name}")

print("\n=== ANIMATIONS ===")
for action in bpy.data.actions:
    print(f"  Action: {action.name} ({len(action.fcurves)} fcurves)")
