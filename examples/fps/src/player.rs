use crate::extensions::NodeExt as _;
use gdnative::*;

/// The Player "class"
#[derive(NativeClass)]
#[inherit(KinematicBody)]
#[user_data(user_data::LocalCellData<Player>)]
pub struct Player {
    vel: Vector3,
    dir: Vector3,
    camera: Camera,
    rotation_helper: Spatial,
    mouse_sensitivity: f32,
    flashlight: SpotLight,

    is_sprinting: bool,
}

static GRAVITY: f32 = -24.8;
static MAX_SPEED: f32 = 20.0;
static JUMP_SPEED: f32 = 18.0;
static ACCEL: f32 = 4.5;
static DEACCEL: f32 = 16.0;
static MAX_SLOPE_ANGLE: f32 = 40.0;
static MAX_SPRINT_SPEED: f32 = 30.0;
static SPRINT_ACCEL: f32 = 18.0;

#[methods]
impl Player {
    fn _init(_owner: KinematicBody) -> Self {
        Player {
            vel: Vector3::zero(),
            dir: Vector3::zero(),

            camera: Camera::new(),
            rotation_helper: Spatial::new(),

            mouse_sensitivity: 0.05,
            flashlight: SpotLight::new(),

            is_sprinting: false,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, mut owner: KinematicBody) {
        owner.set_physics_process(true);

        self.camera = owner.get_typed_node("Rotation_Helper/Camera").unwrap();
        self.rotation_helper = owner.get_typed_node("Rotation_Helper").unwrap();
        self.flashlight = owner.get_typed_node("Rotation_Helper/Flashlight").unwrap();

        Input::godot_singleton().set_mouse_mode(Input::MOUSE_MODE_CAPTURED);
    }

    #[export]
    unsafe fn _physics_process(&mut self, mut owner: KinematicBody, delta: f32) {
        self.process_input(owner, delta);
        self.process_movement(owner, delta);
    }

    unsafe fn process_input(&mut self, owner: KinematicBody, delta: f32) {
        // --------------------------
        // Walking
        self.dir = Vector3::zero();

        let cam_xform = self.camera.get_global_transform();

        let mut input_movement_vector = Vector2::zero();

        let mut input = Input::godot_singleton();

        if input.is_action_pressed("movement_forward".into()) {
            input_movement_vector.y += 1.0;
        }
        if input.is_action_pressed("movement_backward".into()) {
            input_movement_vector.y -= 1.0;
        }
        if input.is_action_pressed("movement_left".into()) {
            input_movement_vector.x -= 1.0;
        }
        if input.is_action_pressed("movement_right".into()) {
            input_movement_vector.x += 1.0;
        }
        
        // This check is required because if you normalize a (0.0, 0.0) vector, it returns (NaN, NaN)
        if input_movement_vector != Vector2::zero() {
            input_movement_vector = input_movement_vector.normalize();
        }

        // Basis vectors are already normalized.
        self.dir += -basis_z(cam_xform.basis) * input_movement_vector.y;
        self.dir += basis_x(cam_xform.basis) * input_movement_vector.x;
        // --------------------------

        // --------------------------
        // Jumping
        if owner.is_on_floor() {
            if input.is_action_just_pressed("movement_jump".into()) {
                self.vel.y = JUMP_SPEED;
            }
        }
        // --------------------------

        // --------------------------
        // Capturing/Freeing the cursor
        if input.is_action_just_pressed("ui_cancel".into()) {
            if input.get_mouse_mode() == InputMouseMode::ModeVisible {
                input.set_mouse_mode(Input::MOUSE_MODE_CAPTURED);
            } else {
                input.set_mouse_mode(Input::MOUSE_MODE_VISIBLE);
            }
        }
        // --------------------------

        // --------------------------
        // Sprinting
        if input.is_action_pressed("movement_sprint".into()) {
            self.is_sprinting = true;
        } else {
            self.is_sprinting = false;
        }
        // --------------------------

        // --------------------------
        // Turning the flashlight on/off
        if input.is_action_just_pressed("flashlight".into()) {
            if self.flashlight.is_visible_in_tree() {
                self.flashlight.hide();
            } else {
                self.flashlight.show();
            }
        }
        // --------------------------
    }

    unsafe fn process_movement(&mut self, mut owner: KinematicBody, delta: f32) {
        self.dir.y = 0.0;

        if self.dir != Vector3::zero() {
            self.dir = self.dir.normalize();
        }

        self.vel.y += delta * GRAVITY;

        let mut hvel = self.vel;
        hvel.y = 0.0;

        let mut target = self.dir;
        // target *= MAX_SPEED;
        if self.is_sprinting {
            target *= MAX_SPRINT_SPEED;
        } else {
            target *= MAX_SPEED;
        }

        let accel = if self.dir.dot(hvel) > 0.0 {
            if self.is_sprinting {
                SPRINT_ACCEL
            } else {
                ACCEL
            }
        } else {
            DEACCEL
        };

        hvel = hvel.lerp(target, accel * delta);
        self.vel.x = hvel.x;
        self.vel.z = hvel.z;

        let deg: f64 = 0.05;
        let rad = deg.to_radians();

        self.vel = owner.move_and_slide(self.vel, Vector3::new(0.0,1.0,0.0), false, 4, rad, true)
    }

    #[export]
    unsafe fn _input(&mut self, mut owner: KinematicBody, event: InputEvent) {
        event.cast().map(|e: InputEventMouseMotion| {
            if Input::godot_singleton().get_mouse_mode() == InputMouseMode::ModeCaptured {
                self.rotation_helper.rotate_x((e.get_relative().y * self.mouse_sensitivity).to_radians() as f64);
                owner.rotate_y((e.get_relative().x * self.mouse_sensitivity * -1.0).to_radians() as f64);

                let mut camera_rot = self.rotation_helper.get_rotation_degrees();
                camera_rot.x = clamp(camera_rot.x,-70.0,70.0);
                self.rotation_helper.set_rotation_degrees(camera_rot);
            }
        });
    }
}

// f32 has a nightly flag for clamp, but I am not using nightly
fn clamp(n: f32, min: f32, max: f32) -> f32 {
    if n < min {
        return min;
    }

    if n > max {
        return max;
    }

    n
}

fn basis_x(basis: Basis) -> Vector3 {
    Vector3::new(basis.elements[0].x, basis.elements[1].x, basis.elements[2].x)
}

fn basis_z(basis: Basis) -> Vector3 {
    Vector3::new(basis.elements[0].z, basis.elements[1].z, basis.elements[2].z)
}
// impl Basis {
//     fn x(self) -> Vector3 {
        
//     }

//     fn y(self) -> Vector3 {
//         Vector3::new(self.elements[0].y, self.elements[1].y, self.elements[2].y)
//     }

//     fn z(self) -> Vector3 {
//         Vector3::new(self.elements[0].z, self.elements[1].z, self.elements[2].z)
//     }
// }