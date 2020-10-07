use intertrait::cast_to;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_graphics::camera::camera_2d::Camera2D;
use tearchan_graphics::camera::camera_3d::Camera3D;
use tearchan_graphics::camera::Camera;

pub trait CameraObject: GameObjectBase {
    fn label(&self) -> &str;

    fn camera(&self) -> &Camera;
}

pub trait Camera2DObject: GameObjectBase {
    fn label(&self) -> &str;

    fn camera2d(&self) -> &Camera2D;
}

pub trait Camera3DObject: GameObjectBase {
    fn label(&self) -> &str;

    fn camera3d(&self) -> &Camera3D;
}

pub struct Camera2DDefaultObject {
    pub camera: Camera2D,
    label: String,
}

impl Camera2DDefaultObject {
    pub fn new(camera: Camera2D, label: String) -> Self {
        Camera2DDefaultObject { camera, label }
    }
}

#[cast_to]
impl GameObjectBase for Camera2DDefaultObject {}

#[cast_to]
impl Camera2DObject for Camera2DDefaultObject {
    fn label(&self) -> &str {
        &self.label
    }

    fn camera2d(&self) -> &Camera2D {
        &self.camera
    }
}

#[cast_to]
impl CameraObject for Camera2DDefaultObject {
    fn label(&self) -> &str {
        &self.label
    }

    fn camera(&self) -> &Camera {
        self.camera.base()
    }
}

pub struct Camera3DDefaultObject {
    pub camera: Camera3D,
    label: String,
}

impl Camera3DDefaultObject {
    pub fn new(camera: Camera3D, label: String) -> Self {
        Camera3DDefaultObject { camera, label }
    }
}

#[cast_to]
impl GameObjectBase for Camera3DDefaultObject {}

#[cast_to]
impl Camera3DObject for Camera3DDefaultObject {
    fn label(&self) -> &str {
        &self.label
    }

    fn camera3d(&self) -> &Camera3D {
        &self.camera
    }
}

#[cast_to]
impl CameraObject for Camera3DDefaultObject {
    fn label(&self) -> &str {
        &self.label
    }

    fn camera(&self) -> &Camera {
        self.camera.base()
    }
}
