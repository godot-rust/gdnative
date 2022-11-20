use gdnative::api::{
    AnimationPlayer, AnimationTree, AudioStreamPlayer, Camera, InputEventMouseMotion,
    KinematicBody, Particles, Position3D, ProjectSettings, TextureRect,
};
use gdnative::prelude::*;

const CAMERA_MOUSE_ROTATION_SPEED: f32 = 0.001;
const CAMERA_CONTROLLER_ROTATION_SPEED: f32 = 3.0;

// A minimum angle lower than or equal to -90 breaks movement if the player is looking upward.
const CAMERA_X_ROT_MIN: f32 = -89.9;
const CAMERA_X_ROT_MAX: f32 = 70.0;

// Release aiming if the mouse/gamepad button was held for longer than 0.4 seconds.
// This works well for trackpads and is more accessible by not making long presses a requirement.
// If the aiming button was held for less than 0.4 seconds, keep aiming until the aiming button is pressed again.
const AIM_HOLD_THRESHOLD: f32 = 0.4;

const MOTION_INTERPOLATE_SPEED: f32 = 10.0;
const ROTATION_INTERPOLATE_SPEED: f32 = 10.0;

const MIN_AIRBORNE_TIME: f32 = 0.1;
const JUMP_SPEED: f32 = 5.0;

// Animation constants
const STRAFE: i32 = 0;
const WALK: i32 = 1;
const JUMP_UP: i32 = 2;
const JUMP_DOWN: i32 = 3;

// Always present sub-nodes
#[derive(Debug, Clone)]
struct SubNodes {
    animation_tree: Ref<AnimationTree>,
    player_model: Ref<Spatial>,
    shoot_from: Ref<Position3D>,
    color_rect: Ref<ColorRect>,
    crosshair: Ref<TextureRect>,
    fire_cooldown: Ref<Timer>,

    // Handles Y rotation (yaw)
    camera_base: Ref<Spatial>,

    camera_animation: Ref<AnimationPlayer>,

    // Handles x rotation (pitch)
    camera_rot: Ref<Spatial>,

    camera_camera: Ref<Camera>,

    // Not used in this example, but would be useful to have access to.
    _sound_effects: Ref<Node>,

    sound_effect_jump: Ref<AudioStreamPlayer>,
    sound_effect_land: Ref<AudioStreamPlayer>,
    sound_effect_shoot: Ref<AudioStreamPlayer>,
    shoot_particle: Ref<Particles>,
    muzzle_particle: Ref<Particles>,
}

// The player "class"
#[derive(NativeClass, Debug)]
#[inherit(KinematicBody)]
pub struct Player {
    // VARIABLES
    airborne_time: f32,
    orientation: Transform,
    root_motion: Transform,
    motion: Vector2,
    velocity: Vector3,
    aiming: bool,

    // If `true`, the aim button was toggled on by a short press (instead of being held down).
    toggled_aim: bool,

    // The duration the aiming button was held for (in seconds).
    aiming_timer: f32,

    camera_x_rot: f32,

    // ONREADY VARIABLES
    initial_position: Vector3,

    gravity: Vector3,

    // SUB-NODES
    sub_nodes: Option<SubNodes>,
}

impl Player {
    fn transition_to_on_air_state(&self) {
        let sub_nodes = self.sub_nodes.as_ref().unwrap();
        let animation_tree = unsafe { sub_nodes.animation_tree.assume_safe() };

        if self.velocity.y > 0.0 {
            animation_tree.set("parameters/state/current", JUMP_UP);
        } else {
            animation_tree.set("parameters/state/current", JUMP_DOWN);
        }
    }

    // Change player state to strafe.
    fn transition_to_aiming_state(&mut self, owner: &KinematicBody, input: &Input, delta: f32) {
        let sub_nodes = self.sub_nodes.as_ref().unwrap();
        let animation_tree = unsafe { sub_nodes.animation_tree.assume_safe() };

        animation_tree.set("parameters/state/current", STRAFE);

        // Change aim according to camera rotation.
        if self.camera_x_rot >= 0.0 {
            /* Aim up. */
            animation_tree.set(
                "parameters/aim/add_amount",
                -self.camera_x_rot / CAMERA_X_ROT_MAX.to_radians(),
            );
        } else {
            /* Aim Down. */
            animation_tree.set(
                "parameters/aim/add_amount",
                self.camera_x_rot / CAMERA_X_ROT_MIN.to_radians(),
            )
        }

        // Convert orientation to quaternions for interpolating rotation.
        let q_from = self.orientation.basis.to_quat();
        let q_to = unsafe {
            sub_nodes
                .camera_base
                .assume_safe()
                .global_transform()
                .basis
                .to_quat()
        };
        // Interpolate current rotation with desired one.
        self.orientation.basis =
            Basis::from_quat(q_from.slerp(q_to, delta * ROTATION_INTERPOLATE_SPEED));

        // The animation's forward/backward axis is reversed.
        animation_tree.set(
            "parameters/strafe/blend_position",
            Vector2::new(self.motion.x, -self.motion.y),
        );

        self.root_motion = animation_tree.get_root_motion_transform();

        let fire_cooldown = unsafe { sub_nodes.fire_cooldown.assume_safe() };

        if input.is_action_pressed("shoot", false) && fire_cooldown.time_left() == 0.0 {
            let shoot_origin =
                unsafe { sub_nodes.shoot_from.assume_safe().global_transform().origin };

            let ch_pos = unsafe {
                sub_nodes.crosshair.assume_safe().get_rect().position
                    + sub_nodes.crosshair.assume_safe().get_rect().size * 0.5
            };

            let camera = unsafe { sub_nodes.camera_camera.assume_safe() };
            let ray_from = camera.project_ray_origin(ch_pos);
            let ray_dir = camera.project_ray_normal(ch_pos);

            let raycast_exclusion_array = VariantArray::new();
            raycast_exclusion_array.push(unsafe { owner.assume_unique() });

            let col = unsafe {
                owner
                    .get_world()
                    .unwrap()
                    .assume_safe()
                    .direct_space_state()
                    .unwrap()
                    .assume_safe()
                    .intersect_ray(
                        ray_from,
                        ray_from + ray_dir * 1000.0,
                        raycast_exclusion_array.into_shared(),
                        0b11,
                        true,
                        false,
                    )
            };

            let shoot_target: Vector3 = if col.is_empty() {
                ray_from + ray_dir * 1000.0
            } else {
                col.values().get(0).to().unwrap()
            };

            let shoot_dir = (shoot_target - shoot_origin).normalized();

            let bullet = unsafe {
                load::<PackedScene>("res://player/bullet/bullet.tscn")
                    .unwrap()
                    .assume_safe()
                    .instance(0)
                    .unwrap()
                    .assume_safe()
                    .cast::<KinematicBody>()
                    .unwrap()
            };
            owner.add_child(bullet, true);

            let mut new_bullet_global_transform = bullet.global_transform();
            new_bullet_global_transform.origin = shoot_origin;

            bullet.set("global_transform", new_bullet_global_transform);

            // If we don't rotate the bullets there is no useful way to control the particles.
            bullet.look_at(shoot_origin + shoot_dir, Vector3::UP);
            bullet.add_collision_exception_with(bullet.get_parent().unwrap());

            let shoot_particle = unsafe { sub_nodes.shoot_particle.assume_safe() };
            let muzzle_particle = unsafe { sub_nodes.muzzle_particle.assume_safe() };

            shoot_particle.restart();
            shoot_particle.set_emitting(true);

            muzzle_particle.restart();
            muzzle_particle.set_emitting(true);
            fire_cooldown.start(-1.0);
            unsafe { sub_nodes.sound_effect_shoot.assume_safe().play(0.0) };
            unsafe { self.add_camera_shake_trauma(sub_nodes.camera_camera, 0.35) };
        }
    }

    // Not in air or aiming, idle.
    fn transition_to_idle_state(&mut self, camera_x: Vector3, camera_z: Vector3, delta: f32) {
        let sub_nodes = self.sub_nodes.as_ref().unwrap();

        // Convert orientation to quaternions for interpolating rotation.
        let target = camera_x * self.motion.x + camera_z * self.motion.y;
        if target.length() > 0.001 {
            let q_from = self.orientation.basis.to_quat();
            let q_to = Transform::default()
                .looking_at(target, Vector3::UP)
                .basis
                .to_quat();
            // Interpolate current rotation with desired one.
            self.orientation.basis =
                Basis::from_quat(q_from.slerp(q_to, delta * ROTATION_INTERPOLATE_SPEED));
        }

        let animation_tree = unsafe { sub_nodes.animation_tree.assume_safe() };

        // Aim to zero (no aiming while walking).
        animation_tree.set("parameters/aim/add_amount", 0.0);

        // Change state to walk.
        animation_tree.set("parameters/state/current", WALK);

        // Blend position for walk speed based on motion.
        animation_tree.set(
            "parameters/walk/blend_position",
            Vector2::new(self.motion.length(), 0.0),
        );

        self.root_motion = unsafe {
            sub_nodes
                .animation_tree
                .assume_safe()
                .get_root_motion_transform()
        };
    }
}

#[methods]
impl Player {
    fn new(owner: &KinematicBody) -> Self {
        Player {
            // VARIABLES
            airborne_time: 100.0,
            orientation: owner.transform(),
            root_motion: owner.transform(),
            motion: Vector2::default(),
            velocity: Vector3::default(),

            aiming: false,

            toggled_aim: false,

            aiming_timer: 0.0,

            camera_x_rot: 0.0,

            // ONREADY VARIABLES
            initial_position: Vector3::default(),
            gravity: Vector3::default(),
            sub_nodes: None,
        }
    }

    #[method]
    fn _init(&self) {
        Input::godot_singleton().set_mouse_mode(Input::MOUSE_MODE_CAPTURED);
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &KinematicBody) {
        // UPDATING ALL ONREADY VARIABLES
        self.initial_position = owner.transform().origin;
        self.gravity = ProjectSettings::godot_singleton()
            .get_setting("physics/3d/default_gravity_vector")
            .to::<Vector3>()
            .unwrap();
        self.gravity.y *= ProjectSettings::godot_singleton()
            .get_setting("physics/3d/default_gravity")
            .to::<f32>()
            .unwrap();

        self.sub_nodes = Some(unsafe {
            SubNodes {
                animation_tree: owner
                    .get_node_as::<AnimationTree>("AnimationTree")
                    .unwrap()
                    .claim(),
                player_model: owner.get_node_as::<Spatial>("PlayerModel").unwrap().claim(),
                shoot_from: owner
                    .get_node_as::<Position3D>(
                        "PlayerModel/Robot_Skeleton/Skeleton/GunBone/ShootFrom",
                    )
                    .unwrap()
                    .claim(),
                color_rect: owner.get_node_as::<ColorRect>("ColorRect").unwrap().claim(),
                crosshair: owner
                    .get_node_as::<TextureRect>("Crosshair")
                    .unwrap()
                    .claim(),
                fire_cooldown: owner.get_node_as::<Timer>("FireCooldown").unwrap().claim(),
                camera_base: owner.get_node_as::<Spatial>("CameraBase").unwrap().claim(),
                camera_animation: owner
                    .get_node_as::<AnimationPlayer>("CameraBase/Animation")
                    .unwrap()
                    .claim(),
                camera_rot: owner
                    .get_node_as::<Spatial>("CameraBase/CameraRot")
                    .unwrap()
                    .claim(),
                camera_camera: owner
                    .get_node_as::<Camera>("CameraBase/CameraRot/SpringArm/Camera")
                    .unwrap()
                    .claim(),
                _sound_effects: owner.get_node_as::<Node>("SoundEffects").unwrap().claim(),
                sound_effect_jump: owner
                    .get_node_as::<AudioStreamPlayer>("SoundEffects/Jump")
                    .unwrap()
                    .claim(),
                sound_effect_land: owner
                    .get_node_as::<AudioStreamPlayer>("SoundEffects/Land")
                    .unwrap()
                    .claim(),
                sound_effect_shoot: owner
                    .get_node_as::<AudioStreamPlayer>("SoundEffects/Shoot")
                    .unwrap()
                    .claim(),

                shoot_particle: owner
                    .get_node_as::<Particles>(
                        "PlayerModel/Robot_Skeleton/Skeleton/GunBone/ShootFrom/ShootParticle",
                    )
                    .unwrap()
                    .claim(),
                muzzle_particle: owner
                    .get_node_as::<Particles>(
                        "PlayerModel/Robot_Skeleton/Skeleton/GunBone/ShootFrom/MuzzleFlash",
                    )
                    .unwrap()
                    .claim(),
            }
        });

        // PRE-INITIALIZE ORIENTATION TRANSFORM
        let sub_nodes = self.sub_nodes.as_ref().unwrap();

        self.orientation = unsafe { sub_nodes.player_model.assume_safe().global_transform() };
        self.orientation.origin = Vector3::default();
    }

    #[method]
    fn _process(&mut self, #[base] owner: &KinematicBody, delta: f32) {
        // Fade out to black if falling out of the map. -17 is lower than
        // the lowest valid position on the map (which is a bit under -16).
        // At 15 units below -17 (so -32), the screen turns fully black.

        let sub_nodes = self.sub_nodes.as_ref().unwrap();

        let color_rect = unsafe { sub_nodes.color_rect.assume_safe() };
        let mut modulate = color_rect.modulate();
        if owner.transform().origin.y < -17.0 {
            modulate.a = std::cmp::min((-17 - owner.transform().origin.y as i32) / 15, 1) as f32;
            color_rect.set_modulate(modulate);

            // If we're below -40, respawn (teleport to the initial position).
            if owner.transform().origin.y < -40.0 {
                let mut new_transform = owner.transform();
                new_transform.origin = self.initial_position;
                owner.set_transform(new_transform);
            }
        } else {
            // Fade out the black ColorRect progressively after being teleported back.
            modulate.a *= 1.0 - delta * 4.0;
            color_rect.set_modulate(modulate);
        }
    }

    #[method]
    fn _physics_process(&mut self, #[base] owner: &KinematicBody, delta: f32) {
        let sub_nodes = self.sub_nodes.clone().unwrap();

        let input = Input::godot_singleton();
        let camera_move = Vector2::new(
            input.get_action_strength("view_right", false) as f32
                - input.get_action_strength("view_left", false) as f32,
            input.get_action_strength("view_up", false) as f32
                - input.get_action_strength("view_down", false) as f32,
        );

        let mut camera_speed_this_frame = delta * CAMERA_CONTROLLER_ROTATION_SPEED;
        if self.aiming {
            camera_speed_this_frame *= 0.5
        }

        self.rotate_camera(camera_move * camera_speed_this_frame);

        let motion_target = Vector2::new(
            input.get_action_strength("move_right", false) as f32
                - input.get_action_strength("move_left", false) as f32,
            input.get_action_strength("move_back", false) as f32
                - input.get_action_strength("move_forward", false) as f32,
        );
        self.motion = self
            .motion
            .linear_interpolate(motion_target, MOTION_INTERPOLATE_SPEED * delta);

        let camera_basis = unsafe { sub_nodes.camera_rot.assume_safe().global_transform().basis };
        let mut camera_z = camera_basis.c();
        let mut camera_x = camera_basis.a();

        camera_z.y = 0.0;
        camera_z = camera_z.normalized();
        camera_x.y = 0.0;
        camera_x = camera_x.normalized();

        let current_aim: bool;

        // Keep aiming if the mouse wasn't held for long enough.
        if input.is_action_just_released("aim", false) && self.aiming_timer <= AIM_HOLD_THRESHOLD {
            current_aim = true;
            self.toggled_aim = true;
        } else {
            current_aim = self.toggled_aim || input.is_action_pressed("aim", false);
            if input.is_action_just_pressed("aim", false) {
                self.toggled_aim = false;
            }
        }

        if current_aim {
            self.aiming_timer += delta;
        } else {
            self.aiming_timer = 0.0;
        }

        if self.aiming != current_aim {
            self.aiming = current_aim;

            let camera_animation = unsafe { sub_nodes.camera_animation.assume_safe() };

            if self.aiming {
                camera_animation.play("shoot", -1.0, 1.0, false)
            } else {
                camera_animation.play("far", -1.0, 1.0, false)
            }
        }

        // Jump/in-air logic.
        self.airborne_time += delta;
        if owner.is_on_floor() {
            if self.airborne_time > 0.5 {
                unsafe { sub_nodes.sound_effect_land.assume_safe().play(0.0) };
            }
            self.airborne_time = 0.0;
        }

        let mut on_air = self.airborne_time > MIN_AIRBORNE_TIME;

        if !on_air && input.is_action_just_pressed("jump", false) {
            self.velocity.y = JUMP_SPEED;
            on_air = true;
            // Increase airborne time so next frame on_air is still true.
            self.airborne_time = MIN_AIRBORNE_TIME;
            unsafe {
                sub_nodes
                    .animation_tree
                    .assume_safe()
                    .set("parameters/state/current", JUMP_UP)
            };
            unsafe { sub_nodes.sound_effect_jump.assume_safe().play(0.0) };
        }

        if on_air {
            self.transition_to_on_air_state();
        } else if self.aiming {
            // Change player state to strafe.
            self.transition_to_aiming_state(owner, input, delta);
        } else {
            // Not in air or aiming, idle.
            self.transition_to_idle_state(camera_x, camera_z, delta);
        }

        // Apply root motion to orientation.
        self.orientation *= self.root_motion;

        let h_velocity = self.orientation.origin / delta;
        self.velocity.x = h_velocity.x;
        self.velocity.z = h_velocity.z;
        self.velocity += self.gravity * delta;
        self.velocity = owner.move_and_slide(
            self.velocity,
            Vector3::UP,
            false,
            4,
            std::f64::consts::FRAC_PI_4,
            true,
        );

        self.orientation.origin = Vector3::default(); // Clear accumulated root motion displacement (was applied to speed).
        self.orientation.basis = self.orientation.basis.orthonormalized(); // Orthonormalize orientation.

        let mut new_global_transform =
            unsafe { sub_nodes.player_model.assume_safe().global_transform() };
        new_global_transform.basis = self.orientation.basis;

        unsafe {
            sub_nodes
                .player_model
                .assume_safe()
                .set("global_transform", new_global_transform)
        };
    }

    #[method]
    fn _input(&mut self, event: Ref<InputEvent>) {
        if let Ok(event) = event.try_cast::<InputEventMouseMotion>() {
            let mut camera_speed_this_frame = CAMERA_MOUSE_ROTATION_SPEED;
            if self.aiming {
                camera_speed_this_frame *= 0.75
            }
            self.rotate_camera(unsafe { event.assume_safe().relative() * camera_speed_this_frame })
        }
    }

    #[method]
    fn rotate_camera(&mut self, camera_move_value: Vector2) {
        let sub_nodes = self.sub_nodes.as_ref().unwrap();

        unsafe {
            sub_nodes
                .camera_base
                .assume_safe()
                .rotate_y((-camera_move_value.x).into());

            // After relative transforms, camera needs to be renormalized.
            sub_nodes.camera_base.assume_safe().orthonormalize();
        }

        self.camera_x_rot += camera_move_value.y;

        self.camera_x_rot = self
            .camera_x_rot
            .clamp(CAMERA_X_ROT_MIN.to_radians(), CAMERA_X_ROT_MAX.to_radians());

        unsafe {
            sub_nodes
                .camera_rot
                .assume_safe()
                .set_rotation(Vector3::new(self.camera_x_rot, 0.0, 0.0))
        };
    }

    #[method]
    // Demonstrating how a Rust + GDScript workflow would look in contrast to the pure Nativescript
    // used for the rest of the player controller logic
    // Call only if camera_camera is safe to use
    unsafe fn add_camera_shake_trauma(&mut self, camera_camera: Ref<Camera>, amount: f32) {
        camera_camera
            .assume_safe()
            .call("add_trauma", &[amount.to_variant()]);
    }
}
