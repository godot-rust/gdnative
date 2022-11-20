# Godot Rust Third Person Example ![Godot 3.5](https://img.shields.io/badge/godot-v3.5-%23478cbf)

Rust port of the character controller from the official Godot Third-Person-Shooter demo. 

### Important notes!:
- Due to file size, the player model asset is not included in the repository and therefore is unincluded with this example.
- To get it, follow the five-minute guide below.
- When opening Player.tcsn, the console may flood with the following error messages:

```
ERROR: Index p_point = 5 is out of bounds (blend_points_used = 5).
   at: get_blend_point_node (scene/animation/animation_blend_space_2d.cpp:110)
ERROR: Index p_point = 6 is out of bounds (blend_points_used = 5).
   at: get_blend_point_node (scene/animation/animation_blend_space_2d.cpp:110)
ERROR: Index p_point = 7 is out of bounds (blend_points_used = 5).
   at: get_blend_point_node (scene/animation/animation_blend_space_2d.cpp:110)
...
```

- This also occurs in the original official project and can be safely ignored.

### Guide on bringing the player model asset into the Godot-Rust TPS example:

- _Ensure you complete steps 1-3 BEFORE opening the project to avoid dependency issues)._

- _You do NOT need to open the regular (GDScript scripted) TPS demo you download from GitHub. You are merely getting the player model folder from it and moving it to the Godot-Rust TPS example, as it is unincluded automatically._

#### Step 1: 
- Go to the Godot official GitHub page and select the TPS demo
- Link here: https://github.com/godotengine/tps-demo

#### Step 2: 
- Press the "Code" button and, in the resulting dropdown, press "Download Zip."

#### Step 3: 
- Extract the downloaded zip folder and move the unzipped "player" folder into the Godot-Rust TPS example project folder. 

#### Step 4: 
- Open the project. The player character should have a GDScript script attached. Replace that with the Nativescript Player.gdns. You should be good to go to build the project!

- Godot will complain about missing dependencies if you open the project before completing steps 1-3. If you do this, complete steps 1-3 and then tell Godot where the files are in the popup.

### Includes:
- 3D Character Controller
  - Smooth Movement
  - Smooth Camera Rotation
  - Aiming
  - Root Motion and Animation Controls
- Example 3D Scene

### Project Controls:
- Move with `W` `A` `S` `D` , `↑` `←` `↓` `→` or controller input.
- Close Game with `Esc`
- Standard FPS controls for aiming and shooting: e.g. `Right Click` or `L2` for aiming

### Credits:
#### Grid Texture/s
- By [KenneyNL](https://www.kenney.nl/assets/prototype-textures)
- Licensed under the terms of the Creative Commons Public Domain Dedication License (CCO 1.0)
#### Level Template
- By [Whimfoome](https://github.com/Whimfoome/godot-FirstPersonStarter)
- Licensed under the terms of the MIT License
#### Player-Character Assets
- [Copyright (c) 2018 Juan Linietsky, Fernando Miguel Calabró](https://github.com/godotengine/tps-demo)
- Licensed under the terms of the Creative Commons Attribution License version 3.0 (CC-BY 3.0)
