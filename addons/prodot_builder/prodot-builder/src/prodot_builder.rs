use gdnative::api::{
    Button,
    Camera,
    Control,
    EditorPlugin,
    EditorSpatialGizmo,
    InputEvent,
    InputEventMouseButton,
    InputEventMouseMotion,
    GlobalConstants,
    Mesh,
    ArrayMesh,
    MeshInstance,
    MeshDataTool,
    Object,
    Texture,
    Script,
    Spatial,
    PackedScene,
    Position3D,
    Viewport,
    World,
};
use gdnative::prelude::*;
use crate::prodot_gizmo::*;
use crate::prodot_mesh::*;
use crate::prodot_utils::*;

#[derive(Copy, Clone, Debug, ToVariant, FromVariant)]
pub enum BuildMode {
    Object = 0,
    Vertex,
    Face,
    Edge
}

impl BuildMode {
    pub fn value(&self) -> i64 {
        match *self {
            BuildMode::Object => 0,
            BuildMode::Vertex => 1,
            BuildMode::Face => 2,
            BuildMode::Edge => 3,
        }
    }

    pub fn set(&self, value: i64) -> BuildMode {
        match value {
            0 => BuildMode::Object,
            1 => BuildMode::Vertex,
            2 => BuildMode::Face,
            3 => BuildMode::Edge,
            _ => {
                godot_print!("[Prodot Builder]: Cannot set BuildMode to an invalid value!");
                BuildMode::Vertex
            },
        }
    }
}

#[derive(NativeClass)]
#[inherit(EditorPlugin)]
pub struct ProdotBuilderPlugin {
    dock: Option<Ref<Control, Shared>>,
    mesh_scene: Option<Ref<PackedScene, Shared>>,
    gizmo_scene: Option<Ref<PackedScene, Shared>>,
    selected_node: Option<Ref<MeshInstance, Shared>>,
    gizmo_node: Option<Ref<Spatial, Shared>>,
    gizmo_positions: Vec<Vector2>,
    active_vertex_index: i32,
    hover_index: i32,
    build_mode: BuildMode,
    object_mode_button: Option<Ref<Button, Shared>>,
    vertex_mode_button: Option<Ref<Button, Shared>>,
    face_mode_button: Option<Ref<Button, Shared>>,
    edge_mode_button: Option<Ref<Button, Shared>>,

    // flags
    is_dragging: bool,
}

#[methods]
impl ProdotBuilderPlugin {
    fn new(_owner: TRef<EditorPlugin>) -> Self {
        ProdotBuilderPlugin {
            dock: None,
            mesh_scene: None,
            gizmo_scene: None,
            selected_node: None,
            gizmo_node: None,
            gizmo_positions: Vec::new(),
            active_vertex_index: -1,
            hover_index: -1,
            build_mode: BuildMode::Vertex,
            object_mode_button: None,
            vertex_mode_button: None,
            face_mode_button: None,
            edge_mode_button: None,

            is_dragging: false,
        }
    }

    #[export]
    fn _enter_tree(&mut self, owner: TRef<EditorPlugin>) {
        // Initialization of the plugin
        owner.set_force_draw_over_forwarding_enabled();
        owner.set_input_event_forwarding_always_enabled();
        
        godot_print!("[Prodot Builder]: Enabled");
        
    
        self.mesh_scene = unsafe {
            Some(
                load_scene("addons/prodot_builder/prodot_mesh.tscn")
                .unwrap()
            )
        };

        self.gizmo_scene = unsafe {
            Some(
                load_scene("addons/prodot_builder/gizmos/prodot_gizmo.tscn")
                .unwrap()
            )
        };

        self.dock = unsafe {
            Some(
                load_scene("addons/prodot_builder/prodot_dock.tscn")
                    .unwrap()
                    .assume_safe()
                    .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                    .unwrap()
                    .assume_safe()
                    .cast::<Control>()
                    .unwrap()
                    .claim(),
            )
        };

        owner.add_control_to_dock(EditorPlugin::DOCK_SLOT_RIGHT_BL, self.dock.unwrap());

        let mesh_script = unsafe {
            load_resource::<Script>("res://addons/prodot_builder/prodot_mesh.gdns", "Script")
            .unwrap()
        };
        /*let gizmo_script = unsafe {
            load_resource::<Script>("res://addons/prodot_builder/gizmos/prodot_gizmo.gdns", "Script")
            .unwrap()
        };*/
        let mesh_texture = unsafe { load_resource::<Texture>("res://addons/prodot_builder/textures/mesh_icon_v5.png", "Texture").unwrap() };
        //let gizmo_texture = mesh_texture.clone();

        owner.add_custom_type("ProdotMesh", "MeshInstance", mesh_script, mesh_texture);
        //owner.add_custom_type("ProdotGizmo", "Spatial", gizmo_script, gizmo_texture);

        let create_cube_button = unsafe {
            self.dock
                .unwrap()
                .assume_safe()
                .get_node("./DockVC/CreateCube")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };

        let object_mode_button = unsafe {
            self.dock
                .unwrap()
                .assume_safe()
                .get_node("./DockVC/ModeVC/RowOne/Object")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };

        object_mode_button.set_pressed(false);

        let vertex_mode_button = unsafe {
            self.dock
                .unwrap()
                .assume_safe()
                .get_node("./DockVC/ModeVC/RowOne/Vertex")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };

        vertex_mode_button.set_pressed(true);

        let face_mode_button = unsafe {
            self.dock
                .unwrap()
                .assume_safe()
                .get_node("./DockVC/ModeVC/RowTwo/Face")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };

        face_mode_button.set_pressed(false);

        let edge_mode_button = unsafe {
            self.dock
                .unwrap()
                .assume_safe()
                .get_node("./DockVC/ModeVC/RowTwo/Edge")
                .unwrap()
                .assume_safe()
                .cast::<Button>()
                .unwrap()
        };

        edge_mode_button.set_pressed(false);


        // Signals
        create_cube_button.connect(
            "create_cube",
            owner,
            "create_cube",
            VariantArray::new_shared(),
            0,
        ).expect("[Prodot Builder]: Error when connecting the create cube button!");

        object_mode_button.connect(
            "object_mode",
            owner,
            "change_build_mode",
            VariantArray::new_shared(),
            0,
        ).expect("[Prodot Builder]: Error when connecting the object mode button!");

        vertex_mode_button.connect(
            "vertex_mode",
            owner,
            "change_build_mode",
            VariantArray::new_shared(),
            0,
        ).expect("[Prodot Builder]: Error when connecting the vertex mode button!");

        face_mode_button.connect(
            "face_mode",
            owner,
            "change_build_mode",
            VariantArray::new_shared(),
            0,
        ).expect("[Prodot Builder]: Error when connecting the face mode button!");

        edge_mode_button.connect(
            "edge_mode",
            owner,
            "change_build_mode",
            VariantArray::new_shared(),
            0,
        ).expect("[Prodot Builder]: Error when connecting the edge mode button!");


        // Cache
        self.object_mode_button = Some ( object_mode_button.claim() );
        self.vertex_mode_button = Some ( vertex_mode_button.claim() );
        self.face_mode_button = Some ( face_mode_button.claim() );
        self.edge_mode_button = Some ( edge_mode_button.claim() );
        
    }

    #[export]
    fn _exit_tree(&mut self, owner: TRef<EditorPlugin>) {
        
        //Remove it from the engine
        owner.remove_custom_type("ProdotMesh");
        //owner.remove_custom_type("ProdotGizmo");

        // Remove the dock
        owner.remove_control_from_docks(self.dock.unwrap());
        
        // Free the stored instanciated nodes
        unsafe { self.dock.unwrap().assume_safe().queue_free() };
        if let Some(gizmo_ref) = self.gizmo_node {
            unsafe { gizmo_ref.assume_safe().queue_free() };
        }
        //unsafe { self.object_mode_button.unwrap().assume_safe().queue_free() };
        //unsafe { self.vertex_mode_button.unwrap().assume_safe().queue_free() };
        //unsafe { self.face_mode_button.unwrap().assume_safe().queue_free() };
        //unsafe { self.edge_mode_button.unwrap().assume_safe().queue_free() };

        // Clean up
        self.selected_node = None;
        self.mesh_scene = None;
        self.gizmo_scene = None;
        self.gizmo_node = None;
        self.object_mode_button = None;
        self.vertex_mode_button = None;
        self.face_mode_button = None;
        self.edge_mode_button = None;

    }

    #[export]
    fn _process(&mut self, owner: TRef<EditorPlugin>, _delta: f64) {
        match self.selected_node {
            Some(_x) => {
                let editor_instance = unsafe { EditorPlugin::get_editor_interface(&owner).unwrap().assume_safe() };
                let selection = unsafe { editor_instance.get_selection().unwrap().assume_safe() };
                if selection.get_selected_nodes().len() == 0 {
                    self.selected_node = None;
                    self.reset(owner);
                    owner.update_overlays();
                } else {

                }
            },
            None => (),
        }
    }

    /// Requests the editor to edit the given object
    /// Sends the plugin the object that is being edited.
    ///
    #[export]
    fn edit(&mut self, owner: TRef<EditorPlugin>, object: Ref<Object>) {
        match unsafe { object.assume_safe().cast::<MeshInstance>() } {
            Some(node) => {
                self.selected_node = Some(node.claim());
                match self.build_mode {
                    BuildMode::Object => {

                    },
                    BuildMode::Vertex => {
                        self.refresh_gizmos(owner);
                    },
                    BuildMode::Face => {

                    },
                    BuildMode::Edge => {

                    },
                }
                owner.update_overlays();
            }
            None => self.selected_node = None,
        }
    }

    #[export]
    fn handles(&mut self, owner: TRef<EditorPlugin>, object: Ref<Object>) -> bool {
        match unsafe { object.assume_safe().cast::<MeshInstance>() } {
            Some(_node) => {
                return true;
            }
            None => {
                self.selected_node = None;
                
            },
        }
        self.reset(owner);
        self.gizmo_positions.clear();
        owner.update_overlays();
        return false;
    }

    #[export]
    fn forward_spatial_force_draw_over_viewport(
        &mut self, _owner: TRef<EditorPlugin>, overlay: Ref<Control>,
    ) {

        if let Some(_node) = self.selected_node {
        
            let overlay_ref = unsafe { overlay.assume_safe() };
            let alpha  = 0.7;
            let peach_color = Color::rgba(0.98431, 0.39216, 0.47451, alpha);
            let white_color = Color::rgba(1.0, 1.0, 1.0, alpha);
            let plum_color = Color::rgba(0.16078, 0.19608, 0.32157, alpha);
            for gizmo_index in 0..self.gizmo_positions.len() {  
                let gizmo_position = self.gizmo_positions[gizmo_index];
                // If it has been clicked on
                if self.active_vertex_index != -1 && gizmo_index == self.active_vertex_index as usize {
                    overlay_ref.draw_circle(
                        gizmo_position,
                        9.0,
                        white_color,
                    );

                    overlay_ref.draw_circle(
                        gizmo_position,
                        8.0,
                        plum_color,
                    );
                } else if self.hover_index != -1 && gizmo_index == self.hover_index as usize {
                // If the index is being hovered over
                    overlay_ref.draw_circle(
                        gizmo_position,
                        9.0,
                        peach_color,
                    );

                    overlay_ref.draw_circle(
                        gizmo_position,
                        8.0,
                        white_color,
                    );
                } else { 
                // If the state is normal
                    overlay_ref.draw_circle(
                        gizmo_position,
                        9.0,
                        white_color,
                    );

                    overlay_ref.draw_circle(
                        gizmo_position,
                        8.0,
                        peach_color,
                    );
                }
            }

        }

    }

    #[export]
    fn forward_spatial_gui_input(
        &mut self, owner: TRef<EditorPlugin>, camera: Ref<Camera>, event: Ref<InputEvent>,
    ) -> bool {

        let mut consume_input = false;
        
        if let Some(node) = self.selected_node {
            
            self.refresh_gizmos_camera(owner, camera);

            let input = unsafe { event.assume_safe() };

    // --------------- Mouse Motion Input ------------------------ //
    //
            if let Some(motion) = input.cast::<InputEventMouseMotion>() {
                // This is in viewport coords
                let mouse = motion.position();
                // Cache if the mouse if hovering over a gizmo
                let mut hover_index_found = false;
                
                /*if self.is_dragging && self.active_vertex_index != -1{
                    self.gizmo_positions[self.active_vertex_index as usize] = mouse;
                    self.edit_mesh(owner);
                   
                } else { */
                /*
                for vertex_index in 0..self.gizmo_positions.len() {
                    let vertex_pos = self.gizmo_positions[vertex_index];
                    
                    if mouse.x > vertex_pos.x - 10.0 &&
                        mouse.x < vertex_pos.x + 10.0 &&
                        mouse.y > vertex_pos.y - 10.0 &&
                        mouse.y < vertex_pos.y + 10.0 { 
                        self.hover_index = vertex_index as i32;
                        hover_index_found = true;
                    }
                }*/
                let cam = unsafe { camera.assume_safe() } ;
                let space_state = unsafe { 
                    cam
                        .get_world()
                        .unwrap()
                        .assume_safe()
                        .direct_space_state()
                        .unwrap()
                        .assume_safe()
                };
                
                let mut plane: Plane = Plane::new( Vector3::new( 0.0, 0.0, 1.0), 0.0);
                let origin = cam.project_ray_origin(mouse);
                let normal: Vector3 = cam.project_ray_normal(mouse);
                
                let mesh = unsafe { node.assume_safe() };
                let mesh_script = 
                    mesh
                        .cast_instance::<ProdotMesh>()
                        .unwrap();
                let vertices = 
                    mesh_script
                        .map_mut(|mesh, owner: TRef<MeshInstance>| {
                            mesh.get_vertices(owner)
                        })
                        .ok()
                        .unwrap();

                let box_size: f32 = 0.05;

                for i in 0..vertices.len() {
                    let vertex_pos = vertices.get(i);
                    plane.d = vertex_pos.z;
                    
                    if let Some(proj_pos) = plane.intersects_ray(origin, normal) {
                        if proj_pos.x > vertex_pos.x - box_size &&
                            proj_pos.x < vertex_pos.x + box_size &&
                            proj_pos.y > vertex_pos.y - box_size &&
                            proj_pos.y < vertex_pos.y + box_size &&
                            proj_pos.z > vertex_pos.z - box_size &&
                            proj_pos.z < vertex_pos.z + box_size {
                            godot_print!("Oyyy");
                        }
                    }
                }
                
                
                if !hover_index_found {
                    self.hover_index = -1;
                }
                //}
            }

    // --------------- Mouse Button Input ------------------------ //
    //
            if let Some(button) = input.cast::<InputEventMouseButton>() {
                match button.button_index() { 
                    GlobalConstants::BUTTON_LEFT => {
                        // If we're hovering over a handle
                        if self.hover_index != -1 {
                            // Was it a press or release event?
                            if button.is_pressed() { 
                                self.is_dragging = true;
                                self.active_vertex_index = self.hover_index;
                                consume_input = true;

                                self.set_gizmo_position(owner);

                            } else {
                                self.is_dragging = false;
                            }
                        } else {
                            if self.is_dragging {
                                self.is_dragging = false;
                            }
                        }
                    },
                    _ => (),
                }
            }
    
    // --------------- Keyboard Input ------------------------ //
    //
            if input.is_action_pressed("ui_cancel", false) {
                self.reset(owner);
                consume_input = true;
            }
            
            owner.update_overlays();
        }
        return consume_input;
    }

    /// Creates a default 1x1x1 cube and attaches it to the current
    /// scenes root node
    ///
    #[export]
    pub fn create_cube(&mut self, owner: TRef<EditorPlugin>) {
        godot_print!("Created cube!");

        let editor = EditorPlugin::get_editor_interface(owner.as_ref()).unwrap();
        let root_node = unsafe { editor.assume_safe().get_edited_scene_root().unwrap().assume_safe() };

        
        let arrays = VariantArray::new();
        let mut normal_array = TypedArray::<Vector3>::new();
        let mut uv_array = TypedArray::<Vector2>::new();
        let mut vertex_array = TypedArray::<Vector3>::new();
        //let mut index_array = TypedArray::<i32>::new();

        arrays.resize(Mesh::ARRAY_MAX as i32);
        //normal_array.resize(3);
        //uv_array.resize(3);
        //vertex_array.resize(3);
        //index_array.resize(6);

        // Point one
        normal_array.push(Vector3::new(0.0, 0.0, 1.0));
        uv_array.push(Vector2::new(0.0, 0.0));
        vertex_array.push(Vector3::new(-1.0, -1.0, 0.0));

        // Point two
        normal_array.push(Vector3::new(0.0, 0.0, 1.0));
        uv_array.push(Vector2::new(0.0, 1.0));
        vertex_array.push(Vector3::new(-1.0, 1.0, 0.0));

        // Point three
        normal_array.push(Vector3::new(0.0, 0.0, 1.0));
        uv_array.push(Vector2::new(1.0, 1.0));
        vertex_array.push(Vector3::new(1.0, 1.0, 0.0));

        //index_array.push(0);
        //index_array.push(1);
        //index_array.push(2);

        arrays.set(Mesh::ARRAY_VERTEX as i32, vertex_array.clone());
        arrays.set(Mesh::ARRAY_NORMAL as i32, normal_array);
        arrays.set(Mesh::ARRAY_TEX_UV as i32, uv_array);
        //arrays.set(Mesh::ARRAY_INDEX as i32, index_array);

        let blend_shapes = VariantArray::new_shared();
        //blend_shapes.resize(Mesh::ARRAY_MAX as i32);
        let end_mesh = ArrayMesh::new();
        end_mesh.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, arrays.into_shared(), blend_shapes, 97280);
        
        let mesh_instance = unsafe {
            self
                .mesh_scene
                .as_ref()
                .unwrap()
                .assume_safe()
                .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                .unwrap()
                .assume_safe()
                .cast::<MeshInstance>()
                .unwrap()
        };
        
        root_node.add_child(mesh_instance, false);
        mesh_instance.set_owner(root_node);
        mesh_instance.set_mesh(end_mesh);

        let mesh_script = 
            mesh_instance
                .cast_instance::<ProdotMesh>()
                .unwrap();
        mesh_script
            .map_mut(|mesh, owner: TRef<MeshInstance>| {
                mesh.set_vertices(owner, vertex_array);
            })
            .ok()
            .unwrap();
            
    }

    /// Calculates the gizmo locations using the cameras found by a recursive search
    /// through the editor tree.
    ///
    ///
    #[export]
    fn refresh_gizmos(&mut self, owner: TRef<EditorPlugin>) {
        if let Some(mesh_inst) = self.selected_node {
            let root_node = unsafe { owner.get_tree().unwrap().assume_safe().root().unwrap().assume_safe().upcast::<Node>() };
            let mesh_node = unsafe { mesh_inst.assume_safe() };
            let mesh_ref = mesh_node.mesh();
            match mesh_ref {
                Some(mesh_ref) => {
                    let mesh = unsafe { mesh_ref.assume_safe() };
                    let mesh_array = mesh.surface_get_arrays(Mesh::ARRAY_VERTEX);
                    let vertex_array = mesh_array.get(0).to_vector3_array();

                    self.gizmo_positions.clear();
                    
                    let mut viewports: Vec<Ref<Viewport>> = Vec::new();
                    self.get_viewports(owner, root_node, viewports.as_mut());
                    for viewport_ref in viewports {
                        let viewport = unsafe { viewport_ref.assume_safe() };
                        if let Some(camera_ref) = viewport.get_camera() {
                            if viewport.size().length() > 0.0 {
                                let camera = unsafe { camera_ref.assume_safe()} ;

                                let mesh_position = mesh_node.global_transform().origin;
                                for i in 0..vertex_array.len() {
                                    self.gizmo_positions.push(camera.unproject_position(mesh_position + vertex_array.get(i)));
                                }
                            }
                        }
                    }
                },
                None => (),
            }
        }
    }
    
    /// Calculates the gizmo locations using the camera supplied.
    ///
    ///
    #[export]
    fn refresh_gizmos_camera(&mut self, _owner: TRef<EditorPlugin>, camera: Ref<Camera>) {
        if let Some(mesh_inst) = self.selected_node {
            let mesh_node = unsafe { mesh_inst.assume_safe() };
            let mesh_ref = mesh_node.mesh().unwrap();
            let mesh = unsafe { mesh_ref.assume_safe() };
            let mesh_array = mesh.surface_get_arrays(Mesh::ARRAY_VERTEX);
            let vertex_array = mesh_array.get(0).to_vector3_array();
            
            self.gizmo_positions.clear();

            let mesh_position = mesh_node.global_transform().origin;
            
            for i in 0..vertex_array.len() {
                unsafe {
                    self.gizmo_positions
                        .push(
                            camera
                            .assume_safe()
                            .unproject_position(
                                mesh_position + 
                                vertex_array.get(i)
                            )
                        )
                };
            }
            
        }
    }
    
    /// Recursively searches for Viewports in the editor tree.
    /// Used for obtaining the spatial editor camera.
    ///
    fn get_viewports(&self, owner: TRef<EditorPlugin>, root_node: TRef<Node>, viewports: &mut Vec<Ref<Viewport>>){

        for child in root_node.get_children().iter() {
            let node = unsafe { child.try_to_object::<Node>().unwrap().assume_safe() };
            if let Some(viewport_ref) = node.cast::<Viewport>()  {
                viewports.push(viewport_ref.claim());
            }
            self.get_viewports(owner, node, viewports);
            
        }

    }
    
    /// Sets the build mode of the plugin
    ///
    /// BuildMode::Vertex
    /// BuildMode::Face
    /// BuildMode::Edge
    ///
    #[export]
    pub fn change_build_mode(&mut self, _owner: TRef<EditorPlugin>, mode: i64) {
        self.build_mode = self.build_mode.set(mode);
        godot_print!("[Prodot Builder]: Switched to {:?} mode", self.build_mode);
        match self.build_mode {
            BuildMode::Object => {
                // Toggle the correct button, and untoggle the other buttons
               
            },
            BuildMode::Vertex => {
                // Toggle the correct button, and untoggle the other buttons
                
            },
            BuildMode::Face => {
                // Toggle the correct button, and untoggle the other buttons
                
            },
            BuildMode::Edge => {
                // Toggle the correct button, and untoggle the other buttons
                
            }
        }
    }
    
    /// Resets the indices of the active index and hover index
    ///
    ///
    #[export]
    fn reset(&mut self, _owner: TRef<EditorPlugin>) {
        self.active_vertex_index = -1;
        self.hover_index = -1;
    }

    #[export]
    fn edit_mesh(&mut self, _owner: TRef<EditorPlugin>) {
        let mesh_node = self.selected_node.unwrap();
        let mesh_pos = unsafe { mesh_node.assume_safe().global_transform().origin };
        let mesh_ref = unsafe { mesh_node.assume_safe().mesh().unwrap() };
        let mesh = unsafe { mesh_ref.assume_safe() };
        let mesh_array =  mesh.cast::<ArrayMesh>().unwrap();
        let mesh_tool = MeshDataTool::new();
        mesh_tool.create_from_surface(mesh_array, 0).expect("[Prodot Builder]: Failed to create mesh from surface!");
        
        let mut active_vertex = mesh_tool.get_vertex(self.active_vertex_index as i64);
        let screen_pos = self.gizmo_positions[self.active_vertex_index as usize];
        active_vertex = Vector3::new(screen_pos.x, screen_pos.y, mesh_pos.z + active_vertex.z);
        
        mesh_tool.set_vertex(self.active_vertex_index as i64, active_vertex);
        mesh_array.surface_remove(0);
        mesh_tool.commit_to_surface(mesh_array).expect("[Prodot Builder]: Failed to commit mesh to surface!");

    }

    #[export]
    fn set_gizmo_position(&mut self, owner: TRef<EditorPlugin>) {
        if self.gizmo_node == None {
            unsafe { 
                let gizzy_scene = self.gizmo_scene.clone();
                let gizzy =   
                       gizzy_scene 
                            .unwrap()
                            .clone()
                            .assume_safe()
                            .instance(PackedScene::GEN_EDIT_STATE_INSTANCE)
                            .unwrap()
                            .assume_safe()
                            .cast::<Spatial>()
                            .unwrap();
                    
                let editor = EditorPlugin::get_editor_interface(owner.as_ref()).unwrap();
                let root_node = editor.assume_safe().get_edited_scene_root().unwrap().assume_safe();

                root_node.add_child(gizzy, false);
                gizzy.set_owner(root_node);
                
                self.gizmo_node = Some( gizzy.claim() );
            }
                    
        }

        if self.selected_node == None {
            return
        }

        if let Some(gizmo_ref) = self.gizmo_node {
            let gizmo = unsafe { gizmo_ref.assume_safe() };
            match self.build_mode {
                BuildMode::Object => {

                },
                BuildMode::Vertex => {
                    if self.active_vertex_index != -1 {
                        let vertex_pos: Vector2 = self.gizmo_positions[self.active_vertex_index as usize];
                        let mesh_pos: Vector3 = unsafe {
                            self.selected_node
                                .unwrap()
                                .assume_safe()
                                .global_transform()
                                .origin
                        };
                        //gizmo.global_translate(mesh_pos + Vector3::new(vertex_pos.x, vertex_pos.y, 0.0));
                        //gizmo.global_translate(mesh_pos + V);
                        
                    }
                },
                BuildMode::Face => {

                },
                BuildMode::Edge => {

                },
            }

            
        }
        
    }

}
