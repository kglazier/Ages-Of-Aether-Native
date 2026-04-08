"""Check bone names in external idle anim."""
import bpy
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=r"C:\Users\kglazier\source\projects\Tower Native\assets\models\enemies\anims\idle.glb")
for obj in bpy.data.objects:
    if obj.type == 'ARMATURE':
        print(f"Bones ({len(obj.data.bones)}):")
        for bone in obj.data.bones:
            print(f"  {bone.name}")
